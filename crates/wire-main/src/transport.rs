//! TCP transport with TPKT framing plus the optional RDP-UDP side-band socket.
//!
//! Every RDP PDU after (and including) the X.224 layer travels inside a TPKT
//! frame: a 4-byte header (`03 00 <len-be>`) whose length covers the whole
//! frame including the header. [`WireTransport`] owns the socket and turns
//! length-prefixed TPKT frames into plain byte slices for the codec layers.
//!
//! The `--insecure` flag is honored here: [`TransportOptions::insecure`]
//! selects Standard RDP Security (RC4, no TLS). When it is `false` the
//! transport refuses to connect, because this crate deliberately implements
//! the RDP protocol from scratch and does not vendor a TLS stack — the
//! workspace's TLS-capable path lives in `rdp-client`.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use crate::error::{WireError, WireResult};

/// Size of the TPKT header: version, reserved, length (big-endian).
pub const TPKT_HEADER_LEN: usize = 4;
/// TPKT version byte, always 3 for RDP.
pub const TPKT_VERSION: u8 = 3;

/// Options controlling how [`WireTransport::connect`] opens the connection.
#[derive(Debug, Clone)]
pub struct TransportOptions {
    /// Remote host name or IP literal.
    pub host: String,
    /// Remote TCP port (default 3389).
    pub port: u16,
    /// Honor `--insecure`: use Standard RDP Security instead of TLS.
    /// `false` is rejected by the transport with a clear error.
    pub insecure: bool,
    /// Also bind a UDP socket for the low-latency side-band (RDP-UDP
    /// multitransport). The side-band is managed but data still flows over
    /// TCP until the server negotiates multitransport.
    pub udp: bool,
    /// Timeout for the initial TCP connect.
    pub connect_timeout: Duration,
    /// Read timeout applied to the TCP stream (None = blocking).
    pub recv_timeout: Option<Duration>,
}

impl Default for TransportOptions {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 3389,
            insecure: true,
            udp: false,
            connect_timeout: Duration::from_secs(10),
            recv_timeout: Some(Duration::from_secs(30)),
        }
    }
}

/// Low-latency UDP side-band socket (RDP-UDP / multitransport).
///
/// The RDP server moves the real-time graphics channel onto this socket once
/// multitransport is negotiated; the TCP stream keeps carrying everything else.
/// This type manages the socket lifecycle and exposes plain datagram I/O; the
/// RDP-UDP framing on top of it is a later concern.
#[derive(Debug)]
pub struct UdpSideband {
    socket: UdpSocket,
    peer: SocketAddr,
}

impl UdpSideband {
    /// Bind an ephemeral local UDP socket and connect it to `host`'s RDP-UDP
    /// port (the same TCP port). Connected mode filters stray datagrams.
    pub fn connect(host: &str, port: u16, timeout: Duration) -> WireResult<Self> {
        let peer = resolve(host, port)?;
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(peer)?;
        socket.set_read_timeout(Some(timeout))?;
        Ok(Self { socket, peer })
    }

    /// Send a datagram on the side-band.
    pub fn send(&self, payload: &[u8]) -> WireResult<usize> {
        Ok(self.socket.send(payload)?)
    }

    /// Receive one datagram.
    pub fn recv(&self, buf: &mut [u8]) -> WireResult<usize> {
        Ok(self.socket.recv(buf)?)
    }

    /// The connected peer address.
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }
}

fn resolve(host: &str, port: u16) -> WireResult<SocketAddr> {
    let mut addrs = (host, port).to_socket_addrs()?;
    addrs
        .next()
        .ok_or_else(|| WireError::Protocol(format!("cannot resolve host `{host}`")))
}

/// A connected RDP transport: one TCP stream with TPKT framing, plus an
/// optional UDP side-band.
#[derive(Debug)]
pub struct WireTransport {
    stream: TcpStream,
    peer: SocketAddr,
    udp: Option<UdpSideband>,
    insecure: bool,
}

impl WireTransport {
    /// Resolve `opts.host`, open the TCP connection, and apply socket tuning
    /// (Nagle off for latency, read timeout, keepalive).
    pub fn connect(opts: TransportOptions) -> WireResult<Self> {
        if !opts.insecure {
            return Err(WireError::Unsupported(
                "secure (TLS/NLA) transport is not implemented in wire-main; \
                 pass --insecure to use Standard RDP Security"
                    .into(),
            ));
        }
        if opts.host.is_empty() {
            return Err(WireError::Protocol("empty host".into()));
        }
        let peer = resolve(&opts.host, opts.port)?;
        let stream = TcpStream::connect_timeout(&peer, opts.connect_timeout)?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(opts.recv_timeout)?;
        // NOTE: `TcpStream::set_keepalive` is still gated behind the unstable
        // `tcp_keepalive` feature on the pinned stable toolchain, so TCP
        // keepalive is deliberately not enabled here. Keepalive is a
        // nice-to-have for NAT pinholes; RDP has its own ping/pong frames
        // once the session is up, and the UDP side-band covers low-latency
        // traffic. Revisit when `tcp_keepalive` stabilizes.

        let udp = if opts.udp {
            Some(UdpSideband::connect(&opts.host, opts.port, opts.connect_timeout)?)
        } else {
            None
        };

        tracing::debug!(%peer, udp = opts.udp, "transport connected");
        Ok(Self {
            stream,
            peer,
            udp,
            insecure: opts.insecure,
        })
    }

    /// Wrap an already-accepted TCP stream (used by the in-process fake server
    /// in integration tests). Applies the same socket tuning as `connect`.
    pub fn from_stream(stream: TcpStream, insecure: bool) -> WireResult<Self> {
        let peer = stream.peer_addr()?;
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        Ok(Self {
            stream,
            peer,
            udp: None,
            insecure,
        })
    }

    /// Wrap `tpkt_body` (everything after the 4-byte TPKT header) in a TPKT
    /// frame and write it to the socket.
    pub fn send(&mut self, tpkt_body: &[u8]) -> WireResult<()> {
        let total = TPKT_HEADER_LEN + tpkt_body.len();
        if total > u16::MAX as usize {
            return Err(WireError::Protocol(format!(
                "TPKT frame too large: {total} bytes"
            )));
        }
        let mut frame = Vec::with_capacity(total);
        frame.push(TPKT_VERSION);
        frame.push(0x00);
        frame.extend_from_slice(&(total as u16).to_be_bytes());
        frame.extend_from_slice(tpkt_body);
        self.stream.write_all(&frame)?;
        Ok(())
    }

    /// Send raw bytes without TPKT framing (used by tests and the UDP path).
    pub fn send_raw(&mut self, bytes: &[u8]) -> WireResult<()> {
        self.stream.write_all(bytes)?;
        Ok(())
    }

    /// Read exactly one TPKT frame and return its body (after the header).
    /// Blocks until the whole frame arrives or the read timeout fires.
    pub fn recv(&mut self) -> WireResult<Vec<u8>> {
        let mut header = [0u8; TPKT_HEADER_LEN];
        self.read_exact(&mut header)?;
        if header[0] != TPKT_VERSION {
            return Err(WireError::Protocol(format!(
                "bad TPKT version byte {:#04x}",
                header[0]
            )));
        }
        let total = u16::from_be_bytes([header[2], header[3]]) as usize;
        if total < TPKT_HEADER_LEN {
            return Err(WireError::Protocol(format!(
                "TPKT length {total} shorter than header"
            )));
        }
        let mut body = vec![0u8; total - TPKT_HEADER_LEN];
        self.read_exact(&mut body)?;
        Ok(body)
    }

    /// Convenience: [`Self::recv`] but returns the full frame including header.
    pub fn recv_frame(&mut self) -> WireResult<Vec<u8>> {
        let body = self.recv()?;
        let mut frame = Vec::with_capacity(body.len() + TPKT_HEADER_LEN);
        frame.push(TPKT_VERSION);
        frame.push(0x00);
        frame.extend_from_slice(&((body.len() + TPKT_HEADER_LEN) as u16).to_be_bytes());
        frame.extend_from_slice(&body);
        Ok(frame)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> WireResult<()> {
        match self.stream.read_exact(buf) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(WireError::Closed),
            Err(e) => Err(e.into()),
        }
    }

    /// Gracefully shut down the TCP stream and drop the UDP side-band.
    pub fn close(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.udp = None;
    }

    /// Whether this transport runs Standard RDP Security (the `--insecure` path).
    pub fn insecure(&self) -> bool {
        self.insecure
    }

    /// The resolved peer address.
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer
    }

    /// Access the UDP side-band, if one was requested at connect time.
    pub fn udp(&mut self) -> Option<&mut UdpSideband> {
        self.udp.as_mut()
    }
}

impl Drop for WireTransport {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insecure_flag_gates_transport() {
        let opts = TransportOptions {
            host: "127.0.0.1".into(),
            port: 1,
            insecure: false,
            ..Default::default()
        };
        let err = WireTransport::connect(opts).unwrap_err();
        assert!(matches!(err, WireError::Unsupported(_)));
    }

    #[test]
    fn empty_host_rejected() {
        let opts = TransportOptions {
            host: String::new(),
            insecure: true,
            ..Default::default()
        };
        assert!(matches!(
            WireTransport::connect(opts),
            Err(WireError::Protocol(_))
        ));
    }
}
