use crate::error::Error;
use crate::{MssqlConnectOptions, MssqlConnection};
use futures_core::future::BoxFuture;
use log::LevelFilter;
use percent_encoding::percent_decode_str;
use sqlx_core::connection::ConnectOptions;
use std::time::Duration;
use url::Url;

impl ConnectOptions for MssqlConnectOptions {
    type Connection = MssqlConnection;

    fn from_url(url: &Url) -> Result<Self, Error> {
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let username = url.username();
        if !username.is_empty() {
            options = options.username(
                &*percent_decode_str(username)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &*percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(path);
        }

        Ok(options)
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(MssqlConnection::establish(self))
    }

    fn log_statements(mut self, level: LevelFilter) -> Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(mut self, level: LevelFilter, duration: Duration) -> Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
