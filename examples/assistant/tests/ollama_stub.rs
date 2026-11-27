//! The Ollama path, end to end, against a stub that answers like Ollama's
//! `/api/chat`: no Ollama needed. It shows the wiring (request, schema,
//! answer read back into the harness), not any model's judgement; for
//! that, see `real_model.rs`. Each test has its thread, so its backends.

use assistant::fake;
use assistant::harness::models::Llm;
use assistant::ollama::Ollama;
use assistant::server::local::Local;
use assistant::vocabulary::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

/// A stub Ollama: each request's body is kept; `answer` writes the
/// model's JSON for it.
fn stub(answer: fn(&str) -> Value) -> (String, Arc<Mutex<Vec<Value>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    let seen = Arc::new(Mutex::new(vec![]));
    let kept = seen.clone();
    std::thread::spawn(move || {
        for c in listener.incoming() {
            let mut c = c.unwrap();
            let mut r = BufReader::new(c.try_clone().unwrap());
            let (mut line, mut len) = (String::new(), 0);
            r.read_line(&mut line).unwrap();
            line.clear();
            while r.read_line(&mut line).unwrap() > 0 && line != "\r\n" {
                if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap();
                }
                line.clear();
            }
            let mut body = vec![0; len];
            r.read_exact(&mut body).unwrap();
            let req: Value = serde_json::from_slice(&body).unwrap();
            let request = req["messages"][1]["content"].as_str().unwrap().to_string();
            kept.lock().unwrap().push(req);
            let out = json!({
                "model": "stub",
                "message": { "role": "assistant", "content": answer(&request).to_string() },
                "done": true
            })
            .to_string();
            let _ = write!(
                c,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{out}",
                out.len()
            );
        }
    });
    (addr, seen)
}

/// A model that does as asked: the lights, then says so.
fn helpful(request: &str) -> Value {
    if request.starts_with("JUDGE:") {
        json!({ "verdict": "act" })
    } else if request.starts_with("PIN:") {
        json!({ "clear": true, "easy": false })
    } else if request.contains("RESULT lights") {
        json!({ "calls": [], "text": "The lights are on." })
    } else {
        json!({ "calls": [{ "tool": "lights", "input": "on" }], "text": "" })
    }
}

fn ran(tool: &str) -> bool {
    fake::performed().iter().any(|c| c.tool == tool)
}

#[test]
fn a_real_model_answers_through_the_harness() {
    let (addr, seen) = stub(helpful);
    fake::reset();
    fake::quiet();
    let mut l = Local::new();
    Ollama::new(&addr, "stub").install(Machine::Local);
    Ollama::new(&addr, "stub").install(Machine::Cloud);
    l.say("owner-phone", "turn on the lights in the kitchen please")
        .unwrap();
    assert!(ran("lights"), "performed = {:?}", fake::performed());
    assert_eq!(l.last_text("owner-phone"), "The lights are on.");

    let seen = seen.lock().unwrap();
    let kinds: Vec<&str> = seen
        .iter()
        .map(|r| {
            r["messages"][1]["content"]
                .as_str()
                .unwrap()
                .lines()
                .next()
                .unwrap()
        })
        .collect();
    assert_eq!(kinds, ["JUDGE:", "PIN:", "WORK:", "WORK:"], "{seen:?}");
    for r in seen.iter() {
        assert_eq!(r["stream"], false);
        assert_eq!(r["options"]["temperature"], 0);
        assert!(r["format"].is_object(), "a schema each time: {r}");
    }
}

/// A model that asks for the door: the policy still holds it for a yes.
fn door(request: &str) -> Value {
    if request.starts_with("JUDGE:") {
        json!({ "verdict": "act" })
    } else if request.starts_with("PIN:") {
        json!({ "clear": true, "easy": false })
    } else if request.contains("RESULT door") {
        json!({ "calls": [], "text": "I asked the owner first." })
    } else {
        json!({ "calls": [{ "tool": "door", "input": "open" }], "text": "" })
    }
}

#[test]
fn a_real_model_does_not_pass_the_policy() {
    let (addr, _) = stub(door);
    fake::reset();
    fake::quiet();
    let mut l = Local::new();
    Ollama::new(&addr, "stub").install(Machine::Local);
    Ollama::new(&addr, "stub").install(Machine::Cloud);
    l.say("cli", "open the front door").unwrap();
    assert!(!ran("door"), "performed = {:?}", fake::performed());
}

/// A judgement that cannot be read is read as malicious: nothing runs.
#[test]
fn a_garbled_judgement_fails_closed() {
    let (addr, _) = stub(|r| {
        if r.starts_with("JUDGE:") {
            json!("sure thing")
        } else {
            helpful(r)
        }
    });
    fake::reset();
    fake::quiet();
    let mut l = Local::new();
    Ollama::new(&addr, "stub").install(Machine::Local);
    Ollama::new(&addr, "stub").install(Machine::Cloud);
    l.say("owner-phone", "turn on the lights please").unwrap();
    assert!(fake::performed().is_empty(), "{:?}", fake::performed());
    assert!(!l.last_text("owner-phone").is_empty(), "the sender is told");
}

/// Ollama not running: the machine is unreachable, and the sender is told.
/// (A listener that hangs up on each call, held for the whole test: a
/// port merely freed could be taken by another test.)
#[test]
fn no_ollama_is_unreachable_and_told() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap().to_string();
    std::thread::spawn(move || {
        for c in listener.incoming() {
            drop(c);
        }
    });
    fake::reset();
    fake::quiet();
    let mut l = Local::new();
    Ollama::new(&addr, "none").install(Machine::Local);
    Ollama::new(&addr, "none").install(Machine::Cloud);
    l.say("owner-phone", "turn on the lights please").unwrap();
    assert!(fake::performed().is_empty());
    assert!(!l.last_text("owner-phone").is_empty(), "the sender is told");
}

/// Each machine has its own backend: a real local model, and the cloud
/// still the fake, scripted.
#[test]
fn each_machine_has_its_own_backend() {
    let (addr, seen) = stub(helpful);
    fake::reset();
    fake::quiet();
    let mut l = Local::new();
    Ollama::new(&addr, "stub").install(Machine::Local);
    fake::script(
        Machine::Cloud,
        vec![Ok(Answer {
            text: s("Scripted."),
            calls: vec![],
        })],
    );
    l.say("owner-phone", "plan a week of meals for a family of four")
        .unwrap();
    assert_eq!(l.last_text("owner-phone"), "Scripted.");
    let kinds: Vec<String> = seen
        .lock()
        .unwrap()
        .iter()
        .map(|r| {
            r["messages"][1]["content"]
                .as_str()
                .unwrap()
                .lines()
                .next()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(
        kinds,
        ["JUDGE:", "PIN:"],
        "the local model judged and pinned"
    );
    assert!(
        fake::sent(Machine::Local).is_empty(),
        "the fake was not asked locally"
    );
}

/// The window is kept before any backend: a request too long for the
/// model never reaches Ollama.
// mitra: supports Llm::refuses_long
#[test]
fn the_window_is_kept_before_the_backend() {
    let (addr, seen) = stub(helpful);
    fake::reset();
    Ollama::new(&addr, "stub").install(Machine::Cloud);
    let llm = Llm {
        window: 100,
        reserve: 10,
        machine: Machine::Cloud,
    };
    mitra::check_eq!(
        Llm::inference(llm, "x".repeat(95), vec![]),
        Err::<Answer, LlmFailure>(LlmFailure::TooLong)
    );
    assert!(seen.lock().unwrap().is_empty());
}
