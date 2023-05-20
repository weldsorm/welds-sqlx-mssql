use crate::common::StatementCache;
use crate::connection::stream::MssqlStream;
use crate::error::Error;
use crate::executor::Executor;
use crate::statement::MssqlStatementMetadata;
use crate::{Mssql, MssqlConnectOptions};
use futures_core::future::BoxFuture;
use futures_util::{FutureExt, TryFutureExt};
use sqlx_core::connection::{Connection, LogSettings};
use sqlx_core::transaction::Transaction;
use std::fmt::{self, Debug, Formatter};
use std::sync::Arc;

mod establish;
mod executor;
mod prepare;
mod stream;
mod tls;

pub struct MssqlConnection {
    pub(crate) stream: MssqlStream,
    pub(crate) cache_statement: StatementCache<Arc<MssqlStatementMetadata>>,
    log_settings: LogSettings,
}

impl Debug for MssqlConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MssqlConnection").finish()
    }
}

impl Connection for MssqlConnection {
    type Database = Mssql;

    type Options = MssqlConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.stream.shutdown().await?;
            Ok(())
        })
    }

    fn close_hard(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.stream.shutdown().await?;
            Ok(())
        })
    }

    //#[allow(unused_mut)]
    //fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
    //    // NOTE: there does not seem to be a clean shutdown packet to send to MSSQL

    //    #[cfg(feature = "_rt-async-std")]
    //    {
    //        use std::future::ready;
    //        use std::net::Shutdown;

    //        ready(self.stream.shutdown(Shutdown::Both).map_err(Into::into)).boxed()
    //    }

    //    #[cfg(feature = "_rt-tokio")]
    //    {
    //        use sqlx_rt::AsyncWriteExt;

    //        // FIXME: This is equivalent to Shutdown::Write, not Shutdown::Both like above
    //        // https://docs.rs/tokio/1.0.1/tokio/io/trait.AsyncWriteExt.html#method.shutdown
    //        async move { self.stream.shutdown().await.map_err(Into::into) }.boxed()
    //    }
    //}

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // NOTE: we do not use `SELECT 1` as that *could* interact with any ongoing transactions
        self.execute("/* SQLx ping */").map_ok(|_| ()).boxed()
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self)
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.stream.wait_until_ready().boxed()
    }

    fn shrink_buffers(&mut self) {
        self.stream.shrink_buffers();
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        todo!("FIX");
        // !self.stream.wbuf.is_empty()
    }
}
