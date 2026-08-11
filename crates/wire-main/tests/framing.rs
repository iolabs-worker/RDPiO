//! Integration tests for the wire-main transport: TPKT framing round-trips
//! over an in-process TCP pair, and the UDP side-band socket.

use std::net::{TcpListener, UdpSocket};
use std::thread;

use wire_main::pdu::{mcs, x224};
use wire_main::transport::{TPKT_VERSION, TransportOptions, UdpSideband, WireTransport};

fn pair() -> (WireTransport, WireTransport) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server_thread = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        WireTransport::from_stream(stream, true).unwrap()
    });
    let client = WireTransport::connect(TransportOptions {
        host: "127.0.0.1".into(),
        port: addr.port(),
        insecure: true,
        ..Default::default()
    })
    .unwrap();
    let server = server_thread.join().unwrap();
    (client, server)
}

#[test]
fn transport_tpkt_roundtrip_both_directions() {
    let (mut client, mut server) = pair();

    let body = b"hello from client";
    client.send(body).unwrap();
    let got = server.recv().unwrap();
    assert_eq!(got, body);

    let reply = b"hello from server";
    server.send(reply).unwrap();
    let got = client.recv().unwrap();
    assert_eq!(got, reply);
}

#[test]
fn transport_frames_are_length_prefixed() {
    let (mut client, mut server) = pair();

    // Two back-to-back frames must arrive as two distinct frames.
    client.send(b"first").unwrap();
    client.send(b"second").unwrap();
    assert_eq!(server.recv().unwrap(), b"first");
    assert_eq!(server.recv().unwrap(), b"second");
}

#[test]
fn transport_recv_frame_includes_header() {
    let (mut client, mut server) = pair();
    client.send(b"abcd").unwrap();
    let frame = server.recv_frame().unwrap();
    assert_eq!(frame[0], TPKT_VERSION);
    assert_eq!(u16::from_be_bytes([frame[2], frame[3]]) as usize, frame.len());
    assert_eq!(&frame[4..], b"abcd");
}

#[test]
fn transport_oversized_frame_rejected() {
    let (mut client, _server) = pair();
    let huge = vec![0u8; u16::MAX as usize];
    let err = client.send(&huge).unwrap_err();
    assert!(err.to_string().contains("too large"));
}

#[test]
fn transport_mcs_data_roundtrip_framing() {
    let (mut client, mut server) = pair();

    // Send a fake "Send Data Request" the way the session layer would.
    let payload = vec![0xde, 0xad, 0xbe, 0xef];
    let req = mcs::send_data_request(1001, 1003, &payload);
    let mut body = Vec::new();
    x224::write_data_header(req.len(), &mut body).unwrap();
    body.extend_from_slice(&req);
    client.send(&body).unwrap();

    let frame = server.recv().unwrap();
    let parsed = mcs::parse_send_data_request(&frame).unwrap();
    assert_eq!(parsed.channel_id, 1003);
    assert_eq!(parsed.data, payload);
}

#[test]
fn transport_udp_sideband_sends_datagrams() {
    let sink = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = sink.local_addr().unwrap();
    let sideband = UdpSideband::connect("127.0.0.1", addr.port(), std::time::Duration::from_secs(2))
        .unwrap();
    sideband.send(b"ping").unwrap();

    let mut buf = [0u8; 64];
    let (n, _from) = sink.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"ping");
}
