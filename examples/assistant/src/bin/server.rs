//! The assistant's server, on this machine, with fake models:
//! `cargo run --bin assistant-server`. One request per connection.

use assistant::cli::{escape, unescape};
use assistant::server::local::Local;
use assistant::vocabulary::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    let listener = TcpListener::bind(&addr).expect("bind");
    eprintln!("assistant-server on {addr}. Tokens: owner-phone, cli, wearable (household 1), other-phone (household 2).");
    assistant::fake::quiet();
    let mut local = Local::new();
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut line = String::new();
        if BufReader::new(&stream).read_line(&mut line).is_err() {
            continue;
        }
        let answer = handle(&mut local, line.trim_end_matches(['\r', '\n']));
        let _ = stream.write_all(answer.as_bytes());
    }
}

fn handle(local: &mut Local, line: &str) -> String {
    let w: Vec<&str> = line.splitn(5, ' ').collect();
    match w.as_slice() {
        ["WHOAMI", t] => match assistant::server::gateway::Accounts::holder(Token {
            secret: t.to_string(),
        }) {
            Some(h) => format!(
                "OK {} {:?} {} {} {}\n",
                h.household,
                h.device,
                h.number,
                local.count(t),
                local.last_delivered(t)
            ),
            None => "ERR REFUSED\n".to_string(),
        },
        ["POST", t, n, kind, text] => {
            let Ok(n) = n.parse::<i64>() else {
                return "ERR REFUSED\n".to_string();
            };
            let Some(h) = assistant::server::gateway::Accounts::holder(Token {
                secret: t.to_string(),
            }) else {
                return "ERR REFUSED\n".to_string();
            };
            let text = unescape(text);
            let body = if *kind == "V" {
                Body::Voice(Audio { bytes: text })
            } else {
                Body::Text(text)
            };
            match local.post(
                t,
                Envelope {
                    id: MessageId {
                        from: Party::Client(h),
                        n,
                    },
                    body,
                },
            ) {
                Ok(_) => format!("OK RECEIVED {n}\n"),
                Err(_) => "ERR REFUSED\n".to_string(),
            }
        }
        ["GET", t, after] => {
            let after = after.parse().unwrap_or(0);
            let mut out = String::new();
            for m in local.fetch(t, after) {
                let text = match m.body {
                    Body::Text(x) => x,
                    Body::Voice(_) => String::new(),
                };
                out += &format!("MSG {} {}\n", m.id.n, escape(&text));
            }
            out + "END\n"
        }
        ["TYPING", t] => format!("OK {}\n", local.typing(t)),
        ["ACK", t, n] => {
            if let Ok(n) = n.parse() {
                local.ack(
                    t,
                    MessageId {
                        from: Party::Assistant,
                        n,
                    },
                );
            }
            "OK\n".to_string()
        }
        _ => "ERR UNKNOWN\n".to_string(),
    }
}
