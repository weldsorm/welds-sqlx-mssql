use crate::error::Error;
use crate::MssqlConnectOptions;
use sqlx_core::connection::ConnectOptions;
use std::str::FromStr;
use url::Url;

impl FromStr for MssqlConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: Url = s.parse().map_err(Error::config)?;
        MssqlConnectOptions::from_url(&url)
    }
}

#[test]
fn it_parses_username_with_at_sign_correctly() {
    let url = "mssql://user@hostname:password@hostname:12345/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!("user@hostname", &opts.username);
}

#[test]
fn it_parses_password_with_non_ascii_chars_correctly() {
    let url = "mssql://username:p@ssw0rd@hostname:12345/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!(Some("p@ssw0rd".into()), opts.password);
}

#[test]
fn it_parses_hostname() {
    let url = "mssql://username:p@ssw0rd@hostname:12345/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!("hostname", &opts.host);
}

#[test]
fn it_parses_port() {
    let url = "mssql://username:p@ssw0rd@hostname:12345/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!(12345, opts.port);
}

#[test]
fn it_parses_database_name() {
    let url = "mssql://username:p@ssw0rd@hostname:12345/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();
    assert_eq!("database", &opts.database);
}
