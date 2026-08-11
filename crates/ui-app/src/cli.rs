//! Command-line parsing for the ui-app binary.
//!
//! The client accepts the same core connection flags as the workspace's
//! headless `rdpio` binary (`--host`, `--port`, `--user`, `--password`,
//! `--insecure`) plus the experimental `--udp` side-band toggle. Parsing is
//! pure logic — no window, no network — so it is fully unit-testable.

use std::fmt;

/// Options parsed from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    /// Remote host name or IP literal (`--host`). Required.
    pub host: String,
    /// Remote TCP port (`--port`, default 3389).
    pub port: u16,
    /// Logon user name (`--user`). Optional.
    pub user: Option<String>,
    /// Logon password (`--password`). Optional.
    pub password: Option<String>,
    /// Honor `--insecure`: use Standard RDP Security (RC4) instead of TLS.
    pub insecure: bool,
    /// Also open the RDP-UDP side-band socket (`--udp`).
    pub udp: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 3389,
            user: None,
            password: None,
            insecure: false,
            udp: false,
        }
    }
}

/// Errors produced while parsing the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// A flag that takes a value was given none.
    MissingValue(String),
    /// An unknown flag was passed.
    UnknownFlag(String),
    /// `--port` was not a valid port number.
    BadPort(String),
    /// No `--host` was given.
    MissingHost,
    /// `--help`/`-h` was requested.
    Help,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingValue(flag) => write!(f, "flag `{flag}` requires a value"),
            CliError::UnknownFlag(flag) => write!(f, "unknown flag `{flag}`"),
            CliError::BadPort(v) => write!(f, "invalid port `{v}`"),
            CliError::MissingHost => write!(f, "missing required `--host`"),
            CliError::Help => write!(f, "help requested"),
        }
    }
}

impl std::error::Error for CliError {}

/// Parse command-line arguments (excluding argv[0]).
///
/// Both `--flag value` and `--flag=value` spellings are accepted for flags
/// that take a value. Boolean flags (`--insecure`, `--udp`, `--help`) take no
/// value.
pub fn parse<I>(args: I) -> Result<CliOptions, CliError>
where
    I: IntoIterator<Item = String>,
{
    let mut opts = CliOptions::default();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        // Split `--flag=value` into the flag and its inline value.
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f.to_string(), Some(v.to_string())),
            None => (arg.clone(), None),
        };

        let mut take_value = |iter: &mut dyn Iterator<Item = String>| -> Result<String, CliError> {
            if let Some(v) = inline.clone() {
                return Ok(v);
            }
            iter.next()
                .ok_or_else(|| CliError::MissingValue(flag.clone()))
        };

        match flag.as_str() {
            "--host" => opts.host = take_value(&mut iter)?,
            "--port" => {
                let v = take_value(&mut iter)?;
                opts.port = v.parse().map_err(|_| CliError::BadPort(v))?;
            }
            "--user" => opts.user = Some(take_value(&mut iter)?),
            "--password" => opts.password = Some(take_value(&mut iter)?),
            "--insecure" => opts.insecure = true,
            "--udp" => opts.udp = true,
            "--help" | "-h" => return Err(CliError::Help),
            other => return Err(CliError::UnknownFlag(other.to_string())),
        }
    }

    if opts.host.is_empty() {
        return Err(CliError::MissingHost);
    }
    Ok(opts)
}

/// The usage banner printed for `--help`.
pub const USAGE: &str = "\
RDPiO ui-app — GPU-accelerated RDP client

USAGE:
  ui-app --host HOST [OPTIONS]

OPTIONS:
  --host HOST         Remote host name or IP literal (required)
  --port PORT         Remote TCP port [default: 3389]
  --user USER         Logon user name
  --password PASS     Logon password (prefer interactive entry)
  --insecure          Use Standard RDP Security (RC4) instead of TLS
  --udp               Also open the RDP-UDP side-band socket
  -h, --help          Print this help and exit
";

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<CliOptions, CliError> {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_core_flags() {
        let opts = parse_strs(&[
            "--host",
            "rdp.example.com",
            "--user",
            "alice",
            "--password",
            "s3cret",
            "--insecure",
            "--udp",
        ])
        .unwrap();
        assert_eq!(opts.host, "rdp.example.com");
        assert_eq!(opts.port, 3389);
        assert_eq!(opts.user.as_deref(), Some("alice"));
        assert_eq!(opts.password.as_deref(), Some("s3cret"));
        assert!(opts.insecure);
        assert!(opts.udp);
    }

    #[test]
    fn defaults_are_sane() {
        let opts = parse_strs(&["--host", "10.0.0.1"]).unwrap();
        assert_eq!(opts.port, 3389);
        assert!(!opts.insecure);
        assert!(!opts.udp);
        assert_eq!(opts.user, None);
        assert_eq!(opts.password, None);
    }

    #[test]
    fn equals_spelling_works() {
        let opts = parse_strs(&["--host=example.com", "--port=3390", "--insecure"]).unwrap();
        assert_eq!(opts.host, "example.com");
        assert_eq!(opts.port, 3390);
        assert!(opts.insecure);
    }

    #[test]
    fn missing_host_rejected() {
        assert_eq!(parse_strs(&["--insecure"]), Err(CliError::MissingHost));
        assert_eq!(parse_strs(&[]), Err(CliError::MissingHost));
    }

    #[test]
    fn missing_value_rejected() {
        assert!(matches!(
            parse_strs(&["--host"]),
            Err(CliError::MissingValue(_))
        ));
        assert!(matches!(
            parse_strs(&["--host", "h", "--port"]),
            Err(CliError::MissingValue(_))
        ));
    }

    #[test]
    fn bad_port_rejected() {
        assert!(matches!(
            parse_strs(&["--host", "h", "--port", "notaport"]),
            Err(CliError::BadPort(_))
        ));
    }

    #[test]
    fn unknown_flag_rejected() {
        assert!(matches!(
            parse_strs(&["--host", "h", "--bogus"]),
            Err(CliError::UnknownFlag(_))
        ));
    }

    #[test]
    fn help_requested() {
        assert_eq!(parse_strs(&["--help"]), Err(CliError::Help));
        assert_eq!(parse_strs(&["-h"]), Err(CliError::Help));
    }
}
