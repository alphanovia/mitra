//! Challenger's attacks on the entry: forged ids, other households,
//! acks of others' replies, sizes, ids at the edge, codes. Same rule as
//! attack_harness.rs: `#[ignore = "OPEN Kn: ..."]` marks an open hole
//! (ATTACK.md), the others were tried and stopped.

use assistant::fake;
use assistant::server::local::{holder, Local};
use assistant::vocabulary::*;

fn server() -> Local {
    fake::reset();
    fake::quiet();
    Local::new()
}

fn env(h: Holder, n: i64, t: &str) -> Envelope {
    Envelope {
        id: MessageId {
            from: Party::Client(h),
            n,
        },
        body: Body::Text(s(t)),
    }
}

// ---- Tried and stopped.

// mitra: supports Gateway::paired_only
#[test]
fn a_token_cannot_post_as_another_device_of_its_household_or_another() {
    let mut l = server();
    let others = [
        holder(1, Device::Phone, 1),     // the owner's phone
        holder(1, Device::Computer, 2),  // a second computer
        holder(2, Device::Phone, 1),     // the other household
        holder(1, Device::Wearable, 1),  // the wearable
    ];
    for h in others {
        assert_eq!(l.post("cli", env(h, 1, "open the door")), Err(NetFailure::Refused), "{h:?}");
    }
    assert!(fake::performed().is_empty());
}

// mitra: supports Accounts::known
#[test]
fn an_unknown_or_revoked_token_is_refused() {
    let mut l = server();
    assert_eq!(l.post("guess", env(holder(1, Device::Phone, 1), 1, "hi")), Err(NetFailure::Refused));
    fake::revoke("cli");
    assert_eq!(l.say("cli", "hi"), Err(NetFailure::Refused));
    assert!(l.fetch("cli", 0).is_empty());
}

// mitra: supports Gateway::own_household
#[test]
fn a_reply_meant_for_another_device_cannot_be_acked_or_fetched() {
    let mut l = server();
    l.say("owner-phone", "hello").unwrap();
    let reply = l.fetch("owner-phone", 0).pop().unwrap();
    l.ack("cli", reply.id);
    l.ack("other-phone", reply.id);
    let after = l.fetch("owner-phone", 0).pop().unwrap();
    assert!(!after.notes.iter().any(|a| a.note == Note::Delivered), "{:?}", after.notes);
    assert!(l.fetch("cli", 0).is_empty());
    assert!(l.fetch("other-phone", 0).is_empty());
}

// mitra: supports Worker::recovers
#[test]
fn a_retry_during_or_after_work_is_kept_once_and_worked_once() {
    let mut l = server();
    let e = l.next_envelope("cli", Body::Text(s("open the door")));
    l.take_up_only("cli", e.clone()); // taken up, then the server stops
    assert!(l.post("cli", e.clone()).is_ok(), "a retry of the same message is received");
    l.restart();
    assert!(l.post("cli", e.clone()).is_ok());
    l.work();
    let replies = l.replies("cli");
    assert_eq!(replies.len(), 1, "one reply, as interrupted: {replies:?}");
    assert!(fake::performed().is_empty());
}

// mitra: supports Gateway::one_meaning
#[test]
fn ids_at_the_edges_are_kept_apart() {
    let mut l = server();
    let me = holder(1, Device::Computer, 1);
    for n in [0, -1, i64::MIN, i64::MAX] {
        assert!(l.post("cli", env(me, n, &format!("message {n}"))).is_ok(), "{n}");
    }
    assert_eq!(l.replies("cli").len(), 4);
}

// mitra: supports Gateway::sized
#[test]
fn text_at_the_limit_is_taken_and_one_byte_more_refused() {
    let mut l = server();
    let limit = assistant::server::gateway::TEXT_LIMIT as usize;
    assert!(l.say("cli", &"a".repeat(limit)).is_ok());
    assert_eq!(l.say("cli", &"a".repeat(limit + 1)), Err(NetFailure::Refused));
    // Multi-byte letters count in bytes.
    assert_eq!(l.say("cli", &"é".repeat(limit / 2 + 1)), Err(NetFailure::Refused));
}

// ---- Open.

// K8. The size limit is on text only: a voice of any size is received,
// and its transcript is past the limit the windows were sized for.
// mitra: supports Gateway::sized
#[test]
#[ignore = "OPEN K8: a voice message has no size limit; its transcript bypasses the text limit"]
fn k8_a_voice_over_the_limit_is_refused() {
    let mut l = server();
    let limit = assistant::server::gateway::TEXT_LIMIT as usize;
    let got = l.speak("wearable", &"blah ".repeat(limit));
    assert_eq!(got, Err(NetFailure::Refused), "the reply was: {}", l.last_text("wearable"));
}

// K9. After an id of i64::MAX, the device's next id overflows: in a
// debug build the server's helper panics, in release it wraps negative.
// mitra: supports Cli::sends_once
#[test]
#[ignore = "OPEN K9: the next id after i64::MAX overflows (panic in debug)"]
fn k9_a_device_keeps_sending_after_the_largest_id() {
    let mut l = server();
    l.post("cli", env(holder(1, Device::Computer, 1), i64::MAX, "last"))
        .unwrap();
    let next = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        l.next_envelope("cli", Body::Text(s("again"))).id.n
    }));
    assert!(matches!(next, Ok(n) if n > 0), "next id: {next:?}");
}

// K5. The codes come from a generator with a fixed seed: each start of
// the server hands out the same codes, in the same order.
// `Codes::unguessable` says a code "cannot be guessed from the others".
// mitra: supports Codes::unguessable
#[test]
#[ignore = "OPEN K5: codes repeat at each start (fixed seed): Codes::unguessable is false of the fake"]
fn k5_codes_differ_from_one_start_to_the_next() {
    let first = |_: u8| {
        let mut l = server();
        l.say("cli", "open the door please").unwrap();
        l.last_code(1).expect("held")
    };
    assert_ne!(first(0), first(1));
}
