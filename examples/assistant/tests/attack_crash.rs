//! Challenger, round 3: crashes and receipts against main 3036a3d.
//! `#[ignore = "OPEN Kn: ..."]` marks an open hole, as in the other files.

use assistant::fake;
use assistant::server::local::Local;
use assistant::vocabulary::*;

fn server() -> Local {
    fake::reset();
    fake::quiet();
    Local::new()
}

fn crashes(f: impl FnOnce()) -> bool {
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    std::panic::set_hook(quiet);
    r.is_err()
}

fn count(tool: &str) -> usize {
    fake::performed().iter().filter(|c| c.tool == tool).count()
}

// K15. A crash inside the owner's yes: the door opened, but the
// Confirmed note is in the one store write that never happened. After
// the restart the code still names a held call, so the same "yes N"
// opens the door a second time. `Confirmations::once`: "a yes works
// once: the call it allowed is no longer held"; `Worker::recovers`:
// "never worked on twice".
// mitra: supports Confirmations::once
#[test]
#[ignore = "OPEN K15: a crash inside a yes leaves the code valid; the same yes runs the door again"]
fn k15_a_yes_cut_short_by_a_crash_does_not_open_the_door_twice() {
    let mut l = server();
    l.say("cli", "open the door").unwrap();
    let code = l.last_code(1).expect("held");
    fake::crash_after("door");
    assert!(crashes(|| {
        let _ = l.say("owner-phone", &format!("yes {code}"));
    }));
    fake::with(|w| w.crash_after = None);
    l.restart();
    assert_eq!(count("door"), 1, "the setup: it ran once before the crash");
    l.say("owner-phone", &format!("yes {code}")).unwrap();
    assert_eq!(
        count("door"),
        1,
        "the same yes opened the door again after the restart: {}",
        l.last_text("owner-phone")
    );
}

// Tried and stopped: what one household wrote down is not told to
// another after a crash (the list is cleared at each take-up).
// mitra: supports Worker::recovers
#[test]
fn a_crash_in_one_household_names_no_call_of_another() {
    let mut l = server();
    l.say("owner-phone", "turn the light on").unwrap();
    assert_eq!(count("lights"), 1, "the setup: household 1 ran the lights");
    let e = l.next_envelope("other-phone", Body::Text(s("hello there")));
    l.take_up_only("other-phone", e);
    l.restart();
    assert!(!l.last_text("other-phone").contains("lights"), "{}", l.last_text("other-phone"));
    assert!(!l.last_text("owner-phone").contains("may have run"), "{}", l.last_text("owner-phone"));
}

// Tried and stopped: a receipt is only for the token's own message.
// mitra: supports Gateway::paired_only
#[test]
fn receipts_tell_nothing_of_another_device_or_household() {
    let mut l = server();
    l.say("owner-phone", "hello").unwrap();
    assert!(l.receipt("owner-phone", 1).is_some(), "the setup");
    assert_eq!(l.receipt("cli", 1), None);
    assert_eq!(l.receipt("other-phone", 1), None);
    assert_eq!(l.receipt("nobody", 1), None);
}

// Tried and stopped: a crash after take-up and before any tool names
// nothing, and the message is not worked on again.
// mitra: supports Worker::recovers
#[test]
fn a_crash_before_any_tool_names_nothing() {
    let mut l = server();
    l.say("owner-phone", "turn the light on").unwrap();
    let e = l.next_envelope("cli", Body::Text(s("turn the light on")));
    l.take_up_only("cli", e);
    l.restart();
    assert!(!l.last_text("cli").contains("may have run"), "{}", l.last_text("cli"));
    assert_eq!(count("lights"), 1);
}
