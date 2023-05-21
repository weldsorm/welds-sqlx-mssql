use futures_core::future::BoxFuture;

use crate::error::Error;
use crate::net::tls::{self, TlsConfig};
use crate::net::{Socket, SocketIntoBox, WithSocket};

use crate::{MssqlConnectOptions, MssqlSslMode};

pub struct MaybeUpgradeTls<'a>(pub &'a MssqlConnectOptions);

impl<'a> WithSocket for MaybeUpgradeTls<'a> {
    type Output = BoxFuture<'a, crate::Result<Box<dyn Socket>>>;

    fn with_socket<S: Socket>(self, socket: S) -> Self::Output {
        Box::pin(maybe_upgrade(socket, self.0))
    }
}

async fn maybe_upgrade<S: Socket>(
    mut socket: S,
    options: &MssqlConnectOptions,
) -> Result<Box<dyn Socket>, Error> {
    // TODO: make ssl upgrades work
    Ok(Box::new(socket))
}
