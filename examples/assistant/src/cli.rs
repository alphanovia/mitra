//! The CLI (see cli.mitra), and its side of the HTTP protocol (see
//! `wire`). Network failures, as the model names them:
//!   no answer, a broken connection, or a 5xx (503: the server is full)
//!     -> Unreachable: the server may not have the call, send it again;
//!   a 4xx -> Refused: the server answered no.

use crate::server::store::Stored;
use crate::vocabulary::*;
use crate::wire;
use mitra::Observe;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// How long one call may take, answer included.
const CALL_TIME: Duration = Duration::from_secs(10);
/// The first pause before sending a lost call again; doubled each time.
const PAUSE: Duration = Duration::from_millis(250);

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(CALL_TIME))
        .http_status_as_error(false)
        .build()
        .into()
}

/// One call: its status and body, or the network's failure.
fn call(
    addr: &str,
    token: &str,
    method: &str,
    path: &str,
    body: Option<&wire::Post>,
) -> Result<(u16, String), NetFailure> {
    let url = format!("{}{path}", wire::base(addr));
    let auth = format!("Bearer {token}");
    let a = agent();
    let got = match (method, body) {
        ("GET", _) => a.get(&url).header("Authorization", &auth).call(),
        (_, Some(b)) => a.post(&url).header("Authorization", &auth).send_json(b),
        (_, None) => a.post(&url).header("Authorization", &auth).send_empty(),
    };
    let mut r = got.map_err(|_| NetFailure::Unreachable)?;
    let status = r.status().as_u16();
    let text = r
        .body_mut()
        .read_to_string()
        .map_err(|_| NetFailure::Unreachable)?;
    match status {
        200..=299 => Ok((status, text)),
        400..=499 => Err(NetFailure::Refused),
        _ => Err(NetFailure::Unreachable),
    }
}

fn json<T: DeserializeOwned>(r: Result<(u16, String), NetFailure>) -> Result<T, NetFailure> {
    let (_, text) = r?;
    serde_json::from_str(&text).map_err(|_| NetFailure::Unreachable)
}

/// Where one of this device's messages stands (W12).
pub fn status_of(addr: &str, token: &str, n: i64) -> Result<wire::Stage, NetFailure> {
    json::<wire::Status>(call(
        addr,
        token,
        "GET",
        &format!("/messages/{n}/status"),
        None,
    ))
    .map(|s| s.status)
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
        let me: wire::Me = json(call(addr, token, "GET", "/me", None))?;
        let device = match me.device.as_str() {
            "Phone" => Device::Phone,
            "Wearable" => Device::Wearable,
            _ => Device::Computer,
        };
        Ok(Cli {
            token: Token { secret: s(token) },
            holder: Holder {
                household: me.household,
                device,
                number: me.number,
            },
            count: me.count,
            seen: me.last_delivered,
            retries: RETRIES,
        })
    }

    /// Sends, and again with the same id while the network fails.
    // mitra: implements Cli::send
    pub fn send(&self, addr: &str, e: Envelope) -> Result<Receipt, NetFailure> {
        let body = match &e.body {
            Body::Text(t) => wire::Post {
                kind: wire::Kind::Text,
                text: t.clone(),
            },
            Body::Voice(a) => wire::Post {
                kind: wire::Kind::Voice,
                text: a.bytes.clone(),
            },
        };
        let path = format!("/messages/{}", e.id.n);
        let post = || {
            call(addr, &self.token.secret, "POST", &path, Some(&body))
                .map(|_| Receipt::Received(e.id))
        };
        let mut k = 0;
        let mut r = post();
        while Cli::again(Cli::lost(&r), k >= self.retries) {
            // A pause before each try, longer each time: a busy server
            // gets room, and the tries are not spent at once.
            std::thread::sleep(PAUSE * (1 << k));
            k += 1;
            r = post();
        }
        r
    }

    // mitra: implements Cli::lost
    /// The network lost the call: the server may not have it.
    pub fn lost(r: &Result<Receipt, NetFailure>) -> bool {
        *r == Err(NetFailure::Unreachable)
    }

    // mitra: implements Cli::again
    /// Whether to send again (W11): while lost, until out of tries.
    pub fn again(lost: bool, out_of_tries: bool) -> bool {
        lost && !out_of_tries
    }

    /// New replies, as (number, text).
    pub fn poll(&self, addr: &str) -> Result<Vec<(i64, String)>, NetFailure> {
        let path = format!("/messages?after={}", self.seen);
        let rs: Vec<wire::Reply> = json(call(addr, &self.token.secret, "GET", &path, None))?;
        Ok(rs.into_iter().map(|r| (r.n, r.text)).collect())
    }

    pub fn working(&self, addr: &str) -> bool {
        matches!(
            json::<wire::Typing>(call(addr, &self.token.secret, "GET", "/typing", None)),
            Ok(wire::Typing { working: true })
        )
    }

    pub fn got(&self, addr: &str, n: i64) {
        let _ = call(
            addr,
            &self.token.secret,
            "POST",
            &format!("/replies/{n}/ack"),
            None,
        );
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
}
