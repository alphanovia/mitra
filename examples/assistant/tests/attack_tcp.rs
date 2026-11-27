//! Challenger's attacks on the server over TCP: one connection that says
//! nothing, and a line that is not UTF-8. Runs the real binary on a
//! free local port. `#[ignore = "OPEN Kn: ..."]` as in the other files.
//!
//! Changed by the builder with the HTTP port (axum): each attack is kept,
//! and written as raw HTTP bytes on the socket, so no client library
//! smooths what is sent.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Server {
    child: Child,
    addr: String,
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start() -> Server {
    // Changed by the builder with the HTTP port: the server binds port 0
    // and says the address it got, so no other test can take it between
    // a probe and the bind (the old probe raced, more so with tokio).
    let mut child = Command::new(env!("CARGO_BIN_EXE_assistant-server"))
        .arg("127.0.0.1:0")
        .stderr(Stdio::piped())
        .spawn()
        .expect("server");
    let mut line = String::new();
    let mut err = BufReader::new(child.stderr.take().unwrap());
    err.read_line(&mut line)
        .expect("the server says where it listens");
    // The rest is drained, so a later line never meets a closed pipe.
    std::thread::spawn(move || std::io::copy(&mut err, &mut std::io::sink()));
    // "assistant-server on http://127.0.0.1:PORT. Tokens: ..."
    let addr = line
        .split("http://")
        .nth(1)
        .and_then(|r| r.split(". ").next())
        .expect("an address")
        .to_string();
    Server { child, addr }
}

/// Raw bytes in, the whole answer out (the server closes each call).
fn raw(addr: &str, bytes: &[u8]) -> std::io::Result<String> {
    let mut c = TcpStream::connect(addr)?;
    c.set_read_timeout(Some(Duration::from_secs(5)))?;
    c.write_all(bytes)?;
    let mut out = vec![];
    let _ = c.read_to_end(&mut out);
    Ok(String::from_utf8_lossy(&out).into_owned())
}

fn get(addr: &str, path: &str, token: &str) -> std::io::Result<String> {
    raw(
        addr,
        format!("GET {path} HTTP/1.1\r\nhost: x\r\nauthorization: Bearer {token}\r\n\r\n")
            .as_bytes(),
    )
}

fn post(addr: &str, path: &str, token: &str, body: &[u8]) -> std::io::Result<String> {
    let mut b = format!(
        "POST {path} HTTP/1.1\r\nhost: x\r\nauthorization: Bearer {token}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    b.extend_from_slice(body);
    raw(addr, &b)
}

fn ok(answer: &str) -> bool {
    answer.starts_with("HTTP/1.1 200")
}

// Tried and answered: a well-formed call works.
// mitra: supports Server::reachable
#[test]
fn a_paired_client_is_answered() {
    let s = start();
    let got = get(&s.addr, "/me", "cli").unwrap();
    assert!(ok(&got), "{got}");
    assert!(
        got.contains(r#""household":1,"device":"Computer","number":1"#),
        "{got}"
    );
    let got = get(&s.addr, "/me", "nobody").unwrap();
    assert!(got.starts_with("HTTP/1.1 401"), "{got}");
}

// And a message goes through, and its reply comes back.
// mitra: supports Server::reachable
#[test]
fn a_message_is_received_and_answered() {
    let s = start();
    let got = post(
        &s.addr,
        "/messages/1",
        "cli",
        br#"{"kind":"text","text":"hello"}"#,
    )
    .unwrap();
    assert!(ok(&got) && got.contains(r#"{"received":1}"#), "{got}");
    let got = get(&s.addr, "/messages/1/status", "cli").unwrap();
    assert!(ok(&got), "{got}");
    let got = get(&s.addr, "/messages?after=0", "cli").unwrap();
    assert!(
        ok(&got) && got.contains("Here is my answer to: hello"),
        "{got}"
    );
}

// K10. One connection that opens and sends nothing holds the server:
// it reads one line at a time, with no time limit, before any token is
// checked. Anyone who can reach the port stops every household.
// mitra: supports Server::reachable
#[test]
// CLOSED K10 by the builder (was: one silent connection blocks the server for everyone (no read timeout, one at a time))
fn k10_a_silent_connection_does_not_block_others() {
    let s = start();
    let _silent = TcpStream::connect(&s.addr).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    let got = get(&s.addr, "/me", "cli");
    assert!(
        matches!(&got, Ok(x) if ok(x)),
        "a paired client waits: {got:?}"
    );
}

// Added by the builder with the HTTP port: a silent connection is also
// closed once the time to send the headers is out, so it holds nothing.
// mitra: supports Server::reachable
#[test]
fn k10_a_silent_connection_is_closed() {
    let s = start();
    let mut silent = TcpStream::connect(&s.addr).unwrap();
    silent
        .set_read_timeout(Some(Duration::from_secs(6)))
        .unwrap();
    let t = Instant::now();
    let mut out = vec![];
    let _ = silent.read_to_end(&mut out);
    assert!(
        t.elapsed() < Duration::from_secs(5),
        "a silent connection was kept {:?}",
        t.elapsed()
    );
}

// K11. A line that is not UTF-8 gets no answer at all: the server drops
// it silently. Nothing silent: it should say the call was refused.
// mitra: supports Assistant::told
#[test]
// CLOSED K11 by the builder (was: a call that is not UTF-8 is dropped without an answer)
fn k11_a_broken_encoding_is_refused_out_loud() {
    let s = start();
    let got = post(
        &s.addr,
        "/messages/1",
        "cli",
        b"{\"kind\":\"text\",\"text\":\"caf\xe9\"}",
    );
    assert!(
        matches!(&got, Ok(x) if x.starts_with("HTTP/1.1 4")),
        "{got:?}"
    );
}

// K14. The read limit is per read, not per call: a client that sends one
// byte a second keeps its connection, and a thread, for as long as it
// likes (64 KiB at 2 s a byte is about 36 hours). Threads are not
// bounded, so enough slow clients wear the server out.
// mitra: supports Server::reachable
#[test]
// CLOSED K14 by the builder (was: the read time limit is per byte, not per call; slow clients hold threads without bound)
fn k14_a_slow_call_is_cut_off_at_the_time_limit() {
    let s = start();
    // Changed by the builder with the HTTP port, after review: the cut
    // must come within the deadline, not merely an answer that is not OK
    // (a hung server gives none), and others are served meanwhile.
    let cut = drip(
        &s.addr,
        b"",
        b"GET /me HTTP/1.1\r\nauthorization: Bearer cli\r\n\r\n",
    );
    assert!(cut.0 < Duration::from_secs(4), "cut after {:?}", cut.0);
    assert!(!ok(&cut.1), "a call drip-fed was still served: {:?}", cut.1);
}

/// Sends `head` at once, then `slow` a byte every 100 ms; halfway through
/// the first second, another client asks and must be answered. When the
/// server cut the call (a write fails, or its answer is read), and what
/// it said.
fn drip(addr: &str, head: &[u8], slow: &[u8]) -> (Duration, String) {
    let mut c = TcpStream::connect(addr).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let t = Instant::now();
    c.write_all(head).unwrap();
    for (k, b) in slow.iter().enumerate() {
        if c.write_all(&[*b]).is_err() {
            return (t.elapsed(), String::new());
        }
        if k == 5 {
            let other = get(addr, "/me", "owner-phone").unwrap();
            assert!(ok(&other), "another client waits meanwhile: {other}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let mut out = String::new();
    let _ = c.read_to_string(&mut out);
    (t.elapsed(), out)
}

// Added by the builder with the HTTP port: the body has its own deadline,
// for the whole body, not per read.
// mitra: supports Server::reachable
#[test]
fn k14_a_slow_body_is_cut_off_at_the_time_limit() {
    let s = start();
    let body = br#"{"kind":"text","text":"hello there, slowly"}"#;
    let head = format!(
        "POST /messages/1 HTTP/1.1\r\nhost: x\r\nauthorization: Bearer cli\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n",
        body.len()
    );
    let cut = drip(&s.addr, head.as_bytes(), body);
    assert!(cut.0 < Duration::from_secs(4), "cut after {:?}", cut.0);
    assert!(
        !ok(&cut.1),
        "a body drip-fed for 4 s was taken: {:?}",
        cut.1
    );
}

// Added by the builder with the HTTP port: past the connection cap, a
// call is told the server is busy, at once (K14). One peer here, so it
// is the cap for one peer that is met (MOST_PER_PEER, K18).
// mitra: supports Server::told_busy
#[test]
fn k14_past_the_cap_a_call_is_told_busy() {
    let s = start();
    let held: Vec<TcpStream> = (0..64)
        .map(|_| TcpStream::connect(&s.addr).unwrap())
        .collect();
    std::thread::sleep(Duration::from_millis(200));
    let got = get(&s.addr, "/me", "cli").unwrap();
    assert!(got.starts_with("HTTP/1.1 503"), "{got}");
    drop(held);
}
