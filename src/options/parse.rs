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
    let url = "mysql://user@hostname:password@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!("user@hostname", &opts.username);
}

#[test]
fn it_parses_password_with_non_ascii_chars_correctly() {
    let url = "mysql://username:p@ssw0rd@hostname:5432/database";
    let opts = MssqlConnectOptions::from_str(url).unwrap();

    assert_eq!(Some("p@ssw0rd".into()), opts.password);
}
