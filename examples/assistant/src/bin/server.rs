//! The assistant's server: `cargo run --bin assistant-server [ADDR]`.
//! HTTP (see `wire`): axum's routes, served by hyper on tokio. The fake
//! plays every machine, unless Ollama is set up for the models (see
//! `ollama`: `ASSISTANT_OLLAMA_MODEL=...`).
//!
//! Nothing here decides: a handler reads the call and hands it to the
//! entry thread, which calls the tied code on the server (`Local`) and
//! answers at once. The model work runs apart, on one thread for each
//! household: it takes a message up, works it off the store, and writes
//! what it found back onto the store as it is then (`Local::finish`).
//! So no call waits for a model (K19, K20), and `/typing` and `/status`
//! show the work while it goes on. Each household's messages are worked
//! one at a time (C14), each with its own write-ahead list (the fake's
//! world is one per thread).
//!
//! The limits from the challenger's rounds, kept here:
//!   K10, K14: the headers, then the body, each within a deadline for the
//!             whole of it (not per read), else the call is cut off;
//!   K17:      a whole connection, answer written included, within
//!             CONN_TIME; a page of replies at most PAGE long;
//!   K14, K18: a connection still being read takes one of MOST_OPEN
//!             places, and at most MOST_PER_PEER for one peer address;
//!             once its token is read, it holds a place of its token
//!             instead (MOST_PER_TOKEN); past any of them, 503 at once
//!             (`Server::admits`); one call per connection;
//!   K20:      calls wait in a queue of at most QUEUE (else 503), and a
//!             call whose client gave up is dropped, not done;
//!   K11:      a body that is not JSON (or not UTF-8) is refused out loud.

use assistant::server::local::Local;
use assistant::vocabulary::*;
// The model's `Body`, beside axum's.
use assistant::vocabulary::Body as Body_;
use assistant::wire;
use axum::body::{to_bytes, Body};
use axum::extract::{Path, Query, Request, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use hyper_util::rt::{TokioIo, TokioTimer};
use hyper_util::service::TowerToHyperService;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::{oneshot, OwnedSemaphorePermit, Semaphore};

/// The longest body read, in bytes: past it, the call is refused.
const LONGEST_BODY: usize = 64 * 1024;
/// How long a client may take to send its headers, and then its body.
const READ_TIME: Duration = Duration::from_secs(2);
/// How long a whole connection may last, its answer written included.
const CONN_TIME: Duration = Duration::from_secs(10);
/// Connections still being read, in all and for one peer address.
const MOST_OPEN: usize = 128;
const MOST_PER_PEER: usize = 16;
/// Connections of one token (by bucket of tokens) once it is read.
const MOST_PER_TOKEN: usize = 8;
const TOKEN_BUCKETS: usize = 256;
/// Calls waiting for the entry thread.
const QUEUE: usize = 256;
/// Replies in one answer to `GET /messages`.
const PAGE: usize = 50;

const BUSY: &[u8] = b"HTTP/1.1 503 Service Unavailable\r\ncontent-type: text/plain\r\ncontent-length: 5\r\nconnection: close\r\n\r\nbusy\n";

fn busy() -> Response {
    (StatusCode::SERVICE_UNAVAILABLE, "busy").into_response()
}

/// The server's own decisions, as the model names them (server.mitra).
struct Server;
impl Server {
    /// A connection is admitted while there is room in all and for its
    /// peer; else it is told busy at once.
    // mitra: implements Server::admits
    fn admits(full: bool, peer_full: bool) -> bool {
        !full && !peer_full
    }
}

/// A call, read and checked for shape, for the entry thread.
enum Job {
    Me(String),
    Post(String, i64, wire::Post),
    Fetch(String, i64),
    Typing(String),
    Status(String, i64),
    Ack(String, i64),
}

/// A thread of the server that ends by a panic ends the process: a
/// server half alive would answer 503 for ever.
struct ExitOnPanic(&'static str);
impl Drop for ExitOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("assistant-server: the {} thread failed; stopping", self.0);
            std::process::exit(1);
        }
    }
}

fn lock(l: &Mutex<Local>) -> MutexGuard<'_, Local> {
    l.lock().expect("a thread that held the server failed")
}

/// The way to the entry thread.
#[derive(Clone)]
struct Entry(mpsc::SyncSender<(Job, oneshot::Sender<Response>)>);

impl Entry {
    /// The entry thread makes the server, and a work thread for each
    /// household; each thread pairs the devices in its own world.
    fn start() -> Entry {
        let (tx, rx) = mpsc::sync_channel::<(Job, oneshot::Sender<Response>)>(QUEUE);
        std::thread::spawn(move || {
            let _guard = ExitOnPanic("entry");
            assistant::fake::quiet();
            let local = Arc::new(Mutex::new(Local::new()));
            let households: Vec<i64> = lock(&local).stores.keys().copied().collect();
            let mut wakes = BTreeMap::new();
            for h in households {
                let (wake, woken) = mpsc::channel::<()>();
                wakes.insert(h, wake);
                let local = local.clone();
                std::thread::spawn(move || work(h, local, woken));
            }
            for (job, answer) in rx {
                // Its client gave up: nothing is done for it (K20).
                if answer.is_closed() {
                    continue;
                }
                let (response, posted) = handle(&mut lock(&local), job);
                if let Some(wake) = posted.and_then(|h| wakes.get(&h)) {
                    let _ = wake.send(());
                }
                let _ = answer.send(response);
            }
        });
        Entry(tx)
    }

    async fn ask(&self, job: Job) -> Response {
        let (tx, rx) = oneshot::channel();
        match self.0.try_send((job, tx)) {
            Ok(()) => rx.await.unwrap_or_else(|_| busy()),
            Err(_) => busy(),
        }
    }
}

/// A household's work: each message taken up, worked off the store, and
/// its writes put back; then wait for the next post.
fn work(h: i64, local: Arc<Mutex<Local>>, woken: mpsc::Receiver<()>) {
    let _guard = ExitOnPanic("work");
    assistant::fake::quiet();
    Local::pair();
    for (m, model) in assistant::ollama::install_from_env() {
        let stand_in = if m == Machine::Cloud {
            ", a stand-in for the frontier model"
        } else {
            ""
        };
        eprintln!("household {h}: {m:?} model {model} (Ollama{stand_in})");
    }
    let (models, tools) = {
        let l = lock(&local);
        (l.models.clone(), l.tools.clone())
    };
    loop {
        loop {
            let taken = lock(&local).take_next(h);
            let Some(t) = taken else { break };
            let done = Local::process(&t, models.clone(), tools.clone());
            lock(&local).finish(t, done);
        }
        if woken.recv().is_err() {
            return;
        }
        while woken.try_recv().is_ok() {}
    }
}

/// On the entry thread: the call, through the tied code. Also the
/// household of a post that was taken.
fn handle(local: &mut Local, job: Job) -> (Response, Option<i64>) {
    let refused = || (StatusCode::FORBIDDEN, "refused").into_response();
    match job {
        Job::Me(t) => {
            let me = assistant::server::gateway::Accounts::holder(Token { secret: t.clone() });
            let r = match me {
                Some(h) => Json(wire::Me {
                    household: h.household,
                    device: format!("{:?}", h.device),
                    number: h.number,
                    count: local.count(&t),
                    last_delivered: local.last_delivered(&t),
                })
                .into_response(),
                None => (StatusCode::UNAUTHORIZED, "refused").into_response(),
            };
            (r, None)
        }
        Job::Post(t, n, p) => {
            let Some(h) = assistant::server::gateway::Accounts::holder(Token { secret: t.clone() })
            else {
                return (refused(), None);
            };
            let body = match p.kind {
                wire::Kind::Text => Body_::Text(p.text),
                wire::Kind::Voice => Body_::Voice(Audio { bytes: p.text }),
            };
            let e = Envelope {
                id: MessageId {
                    from: Party::Client(h),
                    n,
                },
                body,
            };
            match local.accept(&t, e) {
                Ok(_) => (
                    Json(wire::Received { received: n }).into_response(),
                    Some(h.household),
                ),
                Err(_) => (refused(), None),
            }
        }
        Job::Fetch(t, after) => {
            let rs: Vec<wire::Reply> = local
                .fetch(&t, after)
                .into_iter()
                // One page at a time: no answer grows with the store (K17).
                .take(PAGE)
                .map(|m| wire::Reply {
                    n: m.id.n,
                    text: match m.body {
                        Body_::Text(x) => x,
                        Body_::Voice(_) => String::new(),
                    },
                })
                .collect();
            (Json(rs).into_response(), None)
        }
        Job::Typing(t) => (
            Json(wire::Typing {
                working: local.typing(&t),
            })
            .into_response(),
            None,
        ),
        Job::Status(t, n) => {
            let stage = match local.receipt(&t, n) {
                Some(Receipt::Received(_)) => Some(wire::Stage::Received),
                Some(Receipt::Read(_)) => Some(wire::Stage::Read),
                Some(Receipt::Delivered(_)) => Some(wire::Stage::Delivered),
                None => None,
            };
            let r = match stage {
                Some(status) => Json(wire::Status { status }).into_response(),
                None => (StatusCode::NOT_FOUND, "unknown").into_response(),
            };
            (r, None)
        }
        Job::Ack(t, n) => {
            local.ack(
                &t,
                MessageId {
                    from: Party::Assistant,
                    n,
                },
            );
            (StatusCode::NO_CONTENT.into_response(), None)
        }
    }
}

/// The token a call shows, if any. Whether it is paired is the tied
/// code's to say, not this one's.
fn token(h: &HeaderMap) -> Option<String> {
    h.get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

fn no_token() -> Response {
    (StatusCode::UNAUTHORIZED, "no token").into_response()
}

async fn me(State(w): State<Entry>, h: HeaderMap) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    w.ask(Job::Me(t)).await
}

/// A message: its body read whole within the deadline, then read as
/// JSON; anything else is refused out loud.
async fn post_message(
    State(w): State<Entry>,
    h: HeaderMap,
    Path(n): Path<i64>,
    body: Body,
) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    // A body said to be too long is refused before it is read.
    let said = h
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    if said.is_some_and(|n| n > LONGEST_BODY as u64) {
        return (StatusCode::PAYLOAD_TOO_LARGE, "refused: too long").into_response();
    }
    let bytes = match tokio::time::timeout(READ_TIME, to_bytes(body, LONGEST_BODY)).await {
        Err(_) => return (StatusCode::REQUEST_TIMEOUT, "refused: too slow").into_response(),
        Ok(Err(_)) => return (StatusCode::PAYLOAD_TOO_LARGE, "refused: too long").into_response(),
        Ok(Ok(b)) => b,
    };
    match serde_json::from_slice::<wire::Post>(&bytes) {
        Ok(p) => w.ask(Job::Post(t, n, p)).await,
        Err(e) => (StatusCode::BAD_REQUEST, format!("refused: {e}")).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct After {
    after: Option<i64>,
}

async fn fetch(State(w): State<Entry>, h: HeaderMap, Query(a): Query<After>) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    w.ask(Job::Fetch(t, a.after.unwrap_or(0))).await
}

async fn typing(State(w): State<Entry>, h: HeaderMap) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    w.ask(Job::Typing(t)).await
}

async fn status(State(w): State<Entry>, h: HeaderMap, Path(n): Path<i64>) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    w.ask(Job::Status(t, n)).await
}

async fn ack(State(w): State<Entry>, h: HeaderMap, Path(n): Path<i64>) -> Response {
    let Some(t) = token(&h) else {
        return no_token();
    };
    w.ask(Job::Ack(t, n)).await
}

// ---- The places a connection holds.

struct Limits {
    open: Arc<Semaphore>,
    peers: Mutex<HashMap<IpAddr, usize>>,
    tokens: Vec<Arc<Semaphore>>,
}

/// A place while the connection is read: one of all, one of its peer's.
struct Reading {
    _place: OwnedSemaphorePermit,
    peer: IpAddr,
    limits: Arc<Limits>,
}
impl Drop for Reading {
    fn drop(&mut self) {
        let mut peers = self.limits.peers.lock().expect("peers");
        if let Some(n) = peers.get_mut(&self.peer) {
            *n -= 1;
            if *n == 0 {
                peers.remove(&self.peer);
            }
        }
    }
}

/// What a connection holds: a reading place, then its token's. Held for
/// what dropping it gives back.
#[allow(dead_code)]
enum Place {
    Reading(Reading),
    Token(OwnedSemaphorePermit),
}
#[derive(Clone)]
struct Held(Arc<Mutex<Option<Place>>>);

impl Limits {
    fn new() -> Arc<Limits> {
        Arc::new(Limits {
            open: Arc::new(Semaphore::new(MOST_OPEN)),
            peers: Mutex::new(HashMap::new()),
            tokens: (0..TOKEN_BUCKETS)
                .map(|_| Arc::new(Semaphore::new(MOST_PER_TOKEN)))
                .collect(),
        })
    }

    fn admit(self: &Arc<Limits>, peer: IpAddr) -> Option<Reading> {
        let mut peers = self.peers.lock().expect("peers");
        let mine = peers.get(&peer).copied().unwrap_or(0);
        let place = self.open.clone().try_acquire_owned();
        if !Server::admits(place.is_err(), mine >= MOST_PER_PEER) {
            return None;
        }
        peers.insert(peer, mine + 1);
        Some(Reading {
            _place: place.ok()?,
            peer,
            limits: self.clone(),
        })
    }

    fn bucket(&self, token: &str) -> &Arc<Semaphore> {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        token.hash(&mut h);
        &self.tokens[h.finish() as usize % self.tokens.len()]
    }
}

/// Once the token is read, the connection gives back its reading place
/// and takes one of its token's: a device's slow calls hold its own
/// places, not everyone's (K17). Its token full: 503, nothing done.
async fn by_token(
    State(limits): State<Arc<Limits>>,
    Extension(held): Extension<Held>,
    req: Request,
    next: Next,
) -> Response {
    if let Some(t) = token(req.headers()) {
        match limits.bucket(&t).clone().try_acquire_owned() {
            Ok(p) => *held.0.lock().expect("held") = Some(Place::Token(p)),
            Err(_) => return busy(),
        }
    }
    next.run(req).await
}

fn router(e: Entry, limits: Arc<Limits>) -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/messages", get(fetch))
        .route("/messages/{n}", post(post_message))
        .route("/messages/{n}/status", get(status))
        .route("/typing", get(typing))
        .route("/replies/{n}/ack", post(ack))
        .with_state(e)
        .layer(from_fn_with_state(limits, by_token))
}

#[tokio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7878".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind");
    // The address bound, first line on stderr: with port 0, the tests
    // read which port they got.
    let bound = listener.local_addr().expect("bound");
    eprintln!("assistant-server on http://{bound}. Tokens: owner-phone, cli, wearable (household 1), other-phone (household 2).");
    let limits = Limits::new();
    let app = router(Entry::start(), limits.clone());
    loop {
        let (mut stream, peer) = match listener.accept().await {
            Ok(c) => c,
            // Out of files, say: wait a little, do not spin.
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            }
        };
        let Some(reading) = limits.admit(peer.ip()) else {
            tokio::spawn(async move {
                let _ = tokio::time::timeout(READ_TIME, stream.write_all(BUSY)).await;
            });
            continue;
        };
        let held = Held(Arc::new(Mutex::new(Some(Place::Reading(reading)))));
        let service = TowerToHyperService::new(app.clone().layer(Extension(held.clone())));
        tokio::spawn(async move {
            let conn = hyper::server::conn::http1::Builder::new()
                .timer(TokioTimer::new())
                .header_read_timeout(READ_TIME)
                .keep_alive(false)
                .serve_connection(TokioIo::new(stream), service);
            let _ = tokio::time::timeout(CONN_TIME, conn).await;
            drop(held);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: supports Server::told_busy
    #[test]
    fn a_connection_is_admitted_only_with_room() {
        mitra::check_eq!(Server::admits(false, false), true);
        mitra::check_eq!(Server::admits(true, false), false);
        mitra::check_eq!(Server::admits(false, true), false);
        mitra::check_eq!(Server::admits(true, true), false);
    }
}
