use std::ops::{Deref, DerefMut};

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::protocol::col_meta_data::ColMetaData;
use crate::protocol::done::{Done, Status as DoneStatus};
use crate::protocol::env_change::EnvChange;
use crate::protocol::error::Error as ProtocolError;
use crate::protocol::info::Info;
use crate::protocol::login_ack::LoginAck;
use crate::protocol::message::{Message, MessageType};
use crate::protocol::order::Order;
use crate::protocol::packet::{PacketHeader, PacketType, Status};
use crate::protocol::return_status::ReturnStatus;
use crate::protocol::return_value::ReturnValue;
use crate::protocol::row::Row;
use crate::HashMap;
use crate::{MssqlColumn, MssqlConnectOptions, MssqlDatabaseError};
use bytes::{Bytes, BytesMut};
use sqlx_core::io::Encode;
use std::sync::Arc;

use crate::connection::tls::MaybeUpgradeTls;
use crate::net::{self, BufferedSocket, Socket};

pub(crate) struct MssqlStream {
    // A trait object is okay here as the buffering amortizes the overhead of both the dynamic
    // function call as well as the syscall.
    inner: BufferedSocket<Box<dyn Socket>>,

    // how many Done (or Error) we are currently waiting for
    pub(crate) pending_done_count: usize,

    // current transaction descriptor
    // set from ENVCHANGE on `BEGIN` and reset to `0` on a ROLLBACK
    pub(crate) transaction_descriptor: u64,
    pub(crate) transaction_depth: usize,

    // current TabularResult from the server that we are iterating over
    response: Option<(PacketHeader, Bytes)>,

    // most recent column data from ColMetaData
    // we need to store this as its needed when decoding <Row>
    pub(crate) columns: Arc<Vec<MssqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl MssqlStream {
    pub(super) async fn connect(options: &MssqlConnectOptions) -> Result<Self, Error> {
        let socket_future =
            net::connect_tcp(&options.host, options.port, MaybeUpgradeTls(options)).await?;

        let socket = socket_future.await?;

        Ok(Self {
            inner: BufferedSocket::new(socket),
            columns: Default::default(),
            column_names: Default::default(),
            response: None,
            pending_done_count: 0,
            transaction_descriptor: 0,
            transaction_depth: 0,
        })
    }

    // writes the packet out to the write buffer
    // will (eventually) handle packet chunking
    pub(crate) fn write_packet<'en, T: Encode<'en>>(&mut self, ty: PacketType, payload: T) {
        // TODO: Support packet chunking for large packet sizes
        //       We likely need to double-buffer the writes so we know to chunk

        // write out the packet header, leaving room for setting the packet length later

        //find the length of the payload
        let mut buf = Vec::default();
        payload.encode_with(&mut buf, ());
        let data_size = buf.len();
        let header_size = 8;
        let len = (data_size + header_size) as u16;

        let header_packet = PacketHeader {
            r#type: ty,
            status: Status::END_OF_MESSAGE,
            length: len,
            server_process_id: 0,
            packet_id: 1,
        };

        //write the header
        let mut len_offset = 0;
        self.inner.write_with(header_packet, &mut len_offset);
        //write the packet
        self.inner.write(payload);
    }

    // receive the next packet from the database
    // blocks until a packet is available
    pub(super) async fn recv_packet(&mut self) -> Result<(PacketHeader, Bytes), Error> {
        let mut header: PacketHeader = self.inner.read(8).await?;

        // NOTE: From what I can tell, the response type from the server should ~always~
        //       be TabularResult. Here we expect that and die otherwise.
        if !matches!(header.r#type, PacketType::TabularResult) {
            return Err(err_protocol!(
                "received unexpected packet: {:?}",
                header.r#type
            ));
        }

        let mut payload: BytesMut;

        loop {
            let len = (header.length - 8) as usize;
            payload = self.inner.read_buffered(len).await?;

            if header.status.contains(Status::END_OF_MESSAGE) {
                break;
            }

            header = self.inner.read(8).await?;
        }

        Ok((header, payload.freeze()))
    }

    // receive the next ~message~
    // TDS communicates in streams of packets that are themselves streams of messages
    pub(super) async fn recv_message(&mut self) -> Result<Message, Error> {
        loop {
            while self.response.as_ref().map_or(false, |r| !r.1.is_empty()) {
                let buf = if let Some((_, buf)) = self.response.as_mut() {
                    buf
                } else {
                    // this shouldn't be reachable but just nope out
                    // and head to refill our buffer
                    break;
                };

                let ty = MessageType::get(buf)?;

                let message = match ty {
                    MessageType::EnvChange => {
                        match EnvChange::get(buf)? {
                            EnvChange::BeginTransaction(desc) => {
                                self.transaction_descriptor = desc;
                            }

                            EnvChange::CommitTransaction(_) | EnvChange::RollbackTransaction(_) => {
                                self.transaction_descriptor = 0;
                            }

                            _ => {}
                        }

                        continue;
                    }

                    MessageType::Info => {
                        let _ = Info::get(buf)?;
                        continue;
                    }

                    MessageType::Row => Message::Row(Row::get(buf, false, &self.columns)?),
                    MessageType::NbcRow => Message::Row(Row::get(buf, true, &self.columns)?),
                    MessageType::LoginAck => Message::LoginAck(LoginAck::get(buf)?),
                    MessageType::ReturnStatus => Message::ReturnStatus(ReturnStatus::get(buf)?),
                    MessageType::ReturnValue => Message::ReturnValue(ReturnValue::get(buf)?),
                    MessageType::Done => Message::Done(Done::get(buf)?),
                    MessageType::DoneInProc => Message::DoneInProc(Done::get(buf)?),
                    MessageType::DoneProc => Message::DoneProc(Done::get(buf)?),
                    MessageType::Order => Message::Order(Order::get(buf)?),

                    MessageType::Error => {
                        let error = ProtocolError::get(buf)?;
                        return self.handle_error(error);
                    }

                    MessageType::ColMetaData => {
                        // NOTE: there isn't anything to return as the data gets
                        //       consumed by the stream for use in subsequent Row decoding
                        ColMetaData::get(
                            buf,
                            Arc::make_mut(&mut self.columns),
                            Arc::make_mut(&mut self.column_names),
                        )?;
                        continue;
                    }
                };

                return Ok(message);
            }

            // no packet from the server to iterate (or its empty); fill our buffer
            self.response = Some(self.recv_packet().await?);
        }
    }

    pub(crate) fn handle_done(&mut self, _done: &Done) {
        self.pending_done_count -= 1;
    }

    pub(crate) fn handle_error<T>(&mut self, error: ProtocolError) -> Result<T, Error> {
        // NOTE: [error] is sent IN ADDITION TO [done]
        Err(MssqlDatabaseError(error).into())
    }

    pub(crate) async fn wait_until_ready(&mut self) -> Result<(), Error> {
        self.inner.flush().await?;

        while self.pending_done_count > 0 {
            let message = self.recv_message().await?;

            if let Message::DoneProc(done) | Message::Done(done) = message {
                if !done.status.contains(DoneStatus::DONE_MORE) {
                    // finished RPC procedure *OR* SQL batch
                    self.handle_done(&done);
                }
            }
        }

        Ok(())
    }
}

impl Deref for MssqlStream {
    type Target = BufferedSocket<Box<dyn Socket>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MssqlStream {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
