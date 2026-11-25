//! The CLI (see cli.mitra), and the line protocol it speaks with the
//! server. One request per connection:
//!   WHOAMI <token>                    -> OK <household> <device> <number> <count> <last reply got>
//!   POST <token> <n> <T|V> <text>     -> OK RECEIVED <n> | ERR REFUSED
//!   GET <token> <after>               -> MSG <n> <text> ... END
//!   TYPING <token>                    -> OK <true|false>
//!   ACK <token> <n>                   -> OK
//! Texts are escaped: `\` as `\\`, a new line as `\n`.

use crate::server::store::Stored;
use crate::vocabulary::*;
use mitra::Observe;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub fn escape(t: &str) -> String {
    t.replace('\\', "\\\\").replace('\n', "\\n")
}
pub fn unescape(t: &str) -> String {
    let mut out = String::new();
    let mut it = t.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some(x) => out.push(x),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// One request, its answer's lines.
pub fn ask(addr: &str, line: &str) -> Result<Vec<String>, NetFailure> {
    let mut c = TcpStream::connect(addr).map_err(|_| NetFailure::Unreachable)?;
    c.write_all(format!("{line}\n").as_bytes())
        .map_err(|_| NetFailure::Unreachable)?;
    let lines: Vec<String> = BufReader::new(c).lines().map_while(Result::ok).collect();
    if lines.is_empty() {
        return Err(NetFailure::Unreachable);
    }
    Ok(lines)
}

/// The CLI's own state (C9).
// mitra: implements Cli
#[derive(Clone, Debug, PartialEq, Eq, Observe)]
pub struct Cli {
    pub token: Token,
    pub holder: Holder,
    pub count: i64,
    pub seen: i64,
    pub retries: i64,
}

/// How many times the CLI sends a message again: a setting.
pub const RETRIES: i64 = 3;

impl Cli {
    // mitra: implements Cli::envelope
    pub fn envelope(&self, i: Input) -> Envelope {
        Envelope {
            id: MessageId {
                from: Party::Client(self.holder),
                n: self.count + 1,
            },
            body: Cli::body(i),
        }
    }

    // mitra: implements Cli::made
    pub fn made(self) -> Cli {
        Cli {
            count: self.count + 1,
            ..self
        }
    }

    // mitra: implements Cli::body
    pub fn body(i: Input) -> Body {
        match i {
            Input::Typed(t) => Body::Text(t),
            Input::Spoken(a) => Body::Voice(a),
        }
    }

    // mitra: implements Cli::sent
    pub fn sent(r: Result<Receipt, NetFailure>) -> bool {
        r.is_ok()
    }

    // mitra: implements Cli::status
    pub fn status(ok: bool) -> String {
        s(if ok {
            "sent"
        } else {
            "not sent: the assistant cannot be reached, try again later"
        })
    }

    // mitra: implements Cli::last
    pub fn last(es: Vec<Stored>, seen: i64) -> i64 {
        es.iter().map(|s| s.id.n).fold(seen, i64::max)
    }

    // mitra: implements Cli::show
    pub fn show(st: Stored) -> String {
        match st.body {
            Body::Text(x) => x,
            Body::Voice(_) => s(""),
        }
    }

    // ---- Over the network (Network, in the model).

    /// Who this token is, and where its counts stand.
    pub fn connect(addr: &str, token: &str) -> Result<Cli, NetFailure> {
        let l = ask(addr, &format!("WHOAMI {token}"))?;
        let w: Vec<&str> = l[0].split_whitespace().collect();
        if w.first() != Some(&"OK") || w.len() < 6 {
            return Err(NetFailure::Refused);
        }
        let device = match w[2] {
            "Phone" => Device::Phone,
            "Wearable" => Device::Wearable,
            _ => Device::Computer,
        };
        let n = |i: usize| w[i].parse::<i64>().unwrap_or(0);
        Ok(Cli {
            token: Token { secret: s(token) },
            holder: Holder {
                household: n(1),
                device,
                number: n(3),
            },
            count: n(4),
            seen: n(5),
            retries: RETRIES,
        })
    }

    /// Sends, and again with the same id while the network fails.
    // mitra: implements Cli::send
    pub fn send(&self, addr: &str, e: Envelope) -> Result<Receipt, NetFailure> {
        let (kind, text) = match &e.body {
            Body::Text(t) => ("T", t.clone()),
            Body::Voice(a) => ("V", a.bytes.clone()),
        };
        let line = format!(
            "POST {} {} {} {}",
            self.token.secret,
            e.id.n,
            kind,
            escape(&text)
        );
        let mut r = Err(NetFailure::Unreachable);
        for _ in 0..=self.retries {
            r = match ask(addr, &line) {
                Ok(l) if l[0].starts_with("OK") => Ok(Receipt::Received(e.id)),
                Ok(_) => Err(NetFailure::Refused),
                Err(f) => Err(f),
            };
            if r != Err(NetFailure::Unreachable) {
                break;
            }
        }
        r
    }

    /// New replies, as (number, text).
    pub fn poll(&self, addr: &str) -> Result<Vec<(i64, String)>, NetFailure> {
        let l = ask(addr, &format!("GET {} {}", self.token.secret, self.seen))?;
        Ok(l.iter()
            .filter_map(|x| x.strip_prefix("MSG "))
            .filter_map(|x| {
                let (n, t) = x.split_once(' ').unwrap_or((x, ""));
                Some((n.parse().ok()?, unescape(t)))
            })
            .collect())
    }

    pub fn working(&self, addr: &str) -> bool {
        matches!(ask(addr, &format!("TYPING {}", self.token.secret)), Ok(l) if l[0] == "OK true")
    }

    pub fn got(&self, addr: &str, n: i64) {
        let _ = ask(addr, &format!("ACK {} {}", self.token.secret, n));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli() -> Cli {
        Cli {
            token: Token { secret: s("cli") },
            holder: Holder {
                household: 1,
                device: Device::Computer,
                number: 1,
            },
            count: 0,
            seen: 0,
            retries: 3,
        }
    }

    #[test]
    fn a_message_takes_the_next_number() {
        mitra::check_eq!(
            Cli::envelope(&cli(), Input::Typed(s("hello"))),
            Envelope {
                id: MessageId {
                    from: Party::Client(cli().holder),
                    n: 1
                },
                body: Body::Text(s("hello"))
            }
        );
        mitra::check_eq!(Cli::made(cli()), Cli { count: 1, ..cli() });
    }

    #[test]
    fn not_sent_is_told() {
        mitra::check_eq!(Cli::status(true), s("sent"));
        mitra::check_eq!(
            Cli::status(false),
            s("not sent: the assistant cannot be reached, try again later")
        );
    }

    #[test]
    fn escaping_keeps_lines() {
        assert_eq!(unescape(&escape("a\nb\\n")), "a\nb\\n");
    }
}
