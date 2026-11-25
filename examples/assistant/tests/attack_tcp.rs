//! Challenger's attacks on the server over TCP: one connection that says
//! nothing, and a line that is not UTF-8. Runs the real binary on a
//! free local port. `#[ignore = "OPEN Kn: ..."]` as in the other files.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
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
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let addr = format!("127.0.0.1:{port}");
    let child = Command::new(env!("CARGO_BIN_EXE_assistant-server"))
        .arg(&addr)
        .stderr(Stdio::null())
        .spawn()
        .expect("server");
    let t = Instant::now();
    while TcpStream::connect(&addr).is_err() {
        assert!(
            t.elapsed() < Duration::from_secs(10),
            "server did not start"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    Server { child, addr }
}

fn ask(addr: &str, line: &[u8]) -> std::io::Result<String> {
    let mut c = TcpStream::connect(addr)?;
    c.set_read_timeout(Some(Duration::from_secs(2)))?;
    c.write_all(line)?;
    let mut out = String::new();
    BufReader::new(c).read_line(&mut out)?;
    Ok(out)
}

// Tried and answered: a well-formed call works.
// mitra: supports Server::reachable
#[test]
fn a_paired_client_is_answered() {
    let s = start();
    let got = ask(&s.addr, b"WHOAMI cli\n").unwrap();
    assert!(got.starts_with("OK 1 Computer 1"), "{got}");
    assert_eq!(ask(&s.addr, b"WHOAMI nobody\n").unwrap(), "ERR REFUSED\n");
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
    let got = ask(&s.addr, b"WHOAMI cli\n");
    assert!(
        matches!(&got, Ok(x) if x.starts_with("OK")),
        "a paired client waits: {got:?}"
    );
}

// K11. A line that is not UTF-8 gets no answer at all: the server drops
// it silently. Nothing silent: it should say the call was refused.
// mitra: supports Assistant::told
#[test]
// CLOSED K11 by the builder (was: a call that is not UTF-8 is dropped without an answer)
fn k11_a_broken_encoding_is_refused_out_loud() {
    let s = start();
    let got = ask(&s.addr, b"POST cli 1 T caf\xe9\n");
    assert!(matches!(&got, Ok(x) if x.starts_with("ERR")), "{got:?}");
}
