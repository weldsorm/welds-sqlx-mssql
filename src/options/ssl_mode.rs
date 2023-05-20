use crate::error::Error;
use std::str::FromStr;

/// Options for controlling the level of protection provided for Mssql SSL connections.
///
/// It is used by the [`ssl_mode`](super::MssqlConnectOptions::ssl_mode) method.
#[derive(Debug, Clone, Copy)]
pub enum MssqlSslMode {
    /// Only try a non-SSL connection.
    Disable,

    /// First try a non-SSL connection; if that fails, try an SSL connection.
    Allow,

    /// First try an SSL connection; if that fails, try a non-SSL connection.
    Prefer,

    /// Only try an SSL connection. If a root CA file is present, verify the connection
    /// in the same way as if `VerifyCa` was specified.
    Require,

    /// Only try an SSL connection, and verify that the server certificate is issued by a
    /// trusted certificate authority (CA).
    VerifyCa,

    /// Only try an SSL connection; verify that the server certificate is issued by a trusted
    /// CA and that the requested server host name matches that in the certificate.
    VerifyFull,
}

impl Default for MssqlSslMode {
    fn default() -> Self {
        MssqlSslMode::Prefer
    }
}

impl FromStr for MssqlSslMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Ok(match &*s.to_ascii_lowercase() {
            "disable" => MssqlSslMode::Disable,
            "allow" => MssqlSslMode::Allow,
            "prefer" => MssqlSslMode::Prefer,
            "require" => MssqlSslMode::Require,
            "verify-ca" => MssqlSslMode::VerifyCa,
            "verify-full" => MssqlSslMode::VerifyFull,

            _ => {
                return Err(Error::Configuration(
                    format!("unknown value {:?} for `ssl_mode`", s).into(),
                ));
            }
        })
    }
}
