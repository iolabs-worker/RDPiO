//! Platform-agnostic application controller.
//!
//! The controller owns the parsed [`CliOptions`](crate::cli::CliOptions) and
//! the connected [`WireTransport`]. Both platform front-ends (the Win32 +
//! D3D11 window on Windows, the headless fallback elsewhere) drive this same
//! type, so connection setup, PDU I/O, and teardown behave identically and can
//! be unit-tested without a window or a GPU.
//!
//! The golden rule enforced here: the session is only opened *after*
//! connection setup succeeds. [`AppController::connect`] returns `Ok` only
//! once the TCP connection is established, TPKT framing is armed, and the
//! optional UDP side-band is bound. The render loop must not start before
//! that.

use crate::cli::CliOptions;
use wire_main::{TransportOptions, WireError, WireResult, WireTransport};

/// The application controller: owns options + the live transport.
pub struct AppController {
    opts: CliOptions,
    transport: Option<WireTransport>,
}

impl AppController {
    /// Create a controller from parsed options. The connection is *not*
    /// opened until [`Self::connect`] is called.
    pub fn new(opts: CliOptions) -> Self {
        Self {
            opts,
            transport: None,
        }
    }

    /// The parsed options.
    pub fn opts(&self) -> &CliOptions {
        &self.opts
    }

    /// Whether a transport is currently connected.
    pub fn is_connected(&self) -> bool {
        self.transport.is_some()
    }

    /// Open the connection. Returns `Ok` only after the transport is fully
    /// set up (TCP connected, TPKT framing ready, UDP side-band bound when
    /// requested). Idempotent: a second call while connected is a no-op.
    pub fn connect(&mut self) -> WireResult<()> {
        if self.transport.is_some() {
            return Ok(());
        }
        let transport = WireTransport::connect(TransportOptions {
            host: self.opts.host.clone(),
            port: self.opts.port,
            insecure: self.opts.insecure,
            udp: self.opts.udp,
            ..Default::default()
        })?;
        tracing::info!(
            peer = %transport.peer_addr(),
            insecure = transport.insecure(),
            "transport connected"
        );
        self.transport = Some(transport);
        Ok(())
    }

    /// Access the connected transport, if any.
    pub fn transport(&mut self) -> Option<&mut WireTransport> {
        self.transport.as_mut()
    }

    /// Send one PDU body wrapped in a TPKT frame. Errors if not connected.
    pub fn send_pdu(&mut self, body: &[u8]) -> WireResult<()> {
        match self.transport.as_mut() {
            Some(t) => t.send(body),
            None => Err(WireError::Sequence(
                "cannot send: session is not connected".into(),
            )),
        }
    }

    /// Receive one TPKT-framed PDU body. Errors if not connected.
    pub fn recv_pdu(&mut self) -> WireResult<Vec<u8>> {
        match self.transport.as_mut() {
            Some(t) => t.recv(),
            None => Err(WireError::Sequence(
                "cannot recv: session is not connected".into(),
            )),
        }
    }

    /// Tear down the transport gracefully. Safe to call multiple times.
    pub fn close(&mut self) {
        if let Some(t) = self.transport.as_mut() {
            t.close();
        }
        self.transport = None;
    }
}

impl Drop for AppController {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_starts_disconnected() {
        let c = AppController::new(CliOptions {
            host: "127.0.0.1".into(),
            ..Default::default()
        });
        assert!(!c.is_connected());
        assert!(c.transport().is_none());
    }

    #[test]
    fn send_without_connect_errors() {
        let mut c = AppController::new(CliOptions {
            host: "127.0.0.1".into(),
            ..Default::default()
        });
        assert!(matches!(
            c.send_pdu(&[0x01, 0x02, 0x03]),
            Err(WireError::Sequence(_))
        ));
        assert!(matches!(c.recv_pdu(), Err(WireError::Sequence(_))));
    }

    #[test]
    fn insecure_disabled_connection_rejected() {
        // `--insecure` off means the wire-main transport refuses to connect
        // (no TLS stack is vendored in this crate); the controller must
        // surface that failure instead of pretending the session is up.
        let mut c = AppController::new(CliOptions {
            host: "127.0.0.1".into(),
            port: 1,
            insecure: false,
            ..Default::default()
        });
        assert!(matches!(c.connect(), Err(WireError::Unsupported(_))));
        assert!(!c.is_connected());
    }

    #[test]
    fn close_is_idempotent() {
        let mut c = AppController::new(CliOptions {
            host: "127.0.0.1".into(),
            ..Default::default()
        });
        c.close();
        c.close();
        assert!(!c.is_connected());
    }
}
