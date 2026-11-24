use super::*;

fn t(id: i64, title: &str, done: bool) -> Ticket {
    Ticket { id, title: title.to_string(), done }
}

fn added(id: i64, title: &str) -> Event {
    Event::Added(id, title.to_string())
}

fn args(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|x| x.to_string()).collect()
}

// ---------------------------------------------------------------- Codec

// mitra: measures Codec::reads (Event::Added(1, "fix login"), "added 1 fix") -> true
#[test]
fn reads_added_cut_in_the_title() {
    assert!(Codec::reads(&added(1, "fix login"), "added 1 fix"));
}

// mitra: measures Codec::reads (Event::Added(1, "fix login"), "added 1 fix login$") -> true
#[test]
fn reads_added_whole() {
    assert!(Codec::reads(&added(1, "fix login"), "added 1 fix login$"));
}

// mitra: measures Codec::reads (Event::Done(3), "done 3") -> true
#[test]
fn reads_done_cut_before_the_end() {
    assert!(Codec::reads(&Event::Done(3), "done 3"));
}

// mitra: measures Codec::reads (Event::Done(12), "done 1") -> true
#[test]
fn reads_done_cut_in_the_id() {
    assert!(Codec::reads(&Event::Done(12), "done 1"));
}

// mitra: measures Codec::reads (Event::Added(2, "a$b"), "") -> true
#[test]
fn reads_added_with_an_end_in_the_title() {
    assert!(Codec::reads(&added(2, "a$b"), ""));
}

// mitra: measures Codec::reads (Event::Added(7, "x"), "something else") -> true
#[test]
fn reads_added_against_no_prefix() {
    assert!(Codec::reads(&added(7, "x"), "something else"));
}

/// Every cut of every line of a set of events reads as Cut, and the whole
/// line reads back as its event.
#[test]
fn every_cut_of_every_line_is_cut() {
    let events = vec![
        added(1, "fix login"),
        added(10, ""),
        added(-3, "dollar $ and \\ back\nslash"),
        added(123456789, "héllo wörld"),
        Event::Done(1),
        Event::Done(42),
    ];
    for e in &events {
        let line = Codec::encode(e);
        assert_eq!(Codec::decode(&line), Line::Whole(e.clone()));
        for i in 0..line.len() {
            if line.is_char_boundary(i) {
                assert!(Codec::reads(e, &line[..i]), "{:?} cut at {}", e, i);
                assert_eq!(Codec::decode(&line[..i]), Line::Cut);
            }
        }
    }
}

// ---------------------------------------------------------------- Board

// mitra: measures Board::apply (Board { tickets: [], next: 1 }, Event::Added(1, "fix login")) -> Board { tickets: [Ticket { id: 1, title: "fix login", done: false }], next: 2 }
#[test]
fn apply_adds_a_ticket() {
    let b = Board::empty().apply(&added(1, "fix login"));
    assert_eq!(b, Board { tickets: vec![t(1, "fix login", false)], next: 2 });
}

// mitra: measures Board::apply (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Event::Done(1)) -> Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }
#[test]
fn apply_done_twice_stays_done() {
    let b = Board { tickets: vec![t(1, "a", true)], next: 2 };
    assert_eq!(b.apply(&Event::Done(1)), b);
}

// mitra: measures Board::apply (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Event::Added(1, "b")) -> Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }
#[test]
fn apply_ignores_an_old_id() {
    let b = Board { tickets: vec![t(1, "a", false)], next: 2 };
    assert_eq!(b.apply(&added(1, "b")), b);
}

// mitra: measures Board::apply (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Event::Done(2)) -> Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: true }], next: 3 }
#[test]
fn apply_done_marks_only_its_ticket() {
    let b = Board { tickets: vec![t(1, "a", false), t(2, "b", false)], next: 3 };
    assert_eq!(
        b.apply(&Event::Done(2)),
        Board { tickets: vec![t(1, "a", false), t(2, "b", true)], next: 3 }
    );
}

// mitra: measures Board::apply (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Event::Added(5, "e")) -> Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 5, title: "e", done: false }], next: 6 }
#[test]
fn apply_jumps_to_a_higher_id() {
    let b = Board { tickets: vec![t(1, "a", false)], next: 2 };
    assert_eq!(
        b.apply(&added(5, "e")),
        Board { tickets: vec![t(1, "a", false), t(5, "e", false)], next: 6 }
    );
}

// mitra: measures Board::apply (Board { tickets: [], next: 1 }, Event::Done(4)) -> Board { tickets: [], next: 1 }
#[test]
fn apply_done_of_no_ticket_changes_nothing() {
    assert_eq!(Board::empty().apply(&Event::Done(4)), Board::empty());
}

// mitra: measures Board::replay ([Line::Whole(Event::Added(1, "a")), Line::Cut]) -> Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }
#[test]
fn replay_skips_a_cut_last_line() {
    let b = Board::replay(&[Line::Whole(added(1, "a")), Line::Cut]);
    assert_eq!(b, Board { tickets: vec![t(1, "a", false)], next: 2 });
}

// mitra: measures Board::replay ([]) -> Board { tickets: [], next: 1 }
#[test]
fn replay_of_nothing_is_empty() {
    assert_eq!(Board::replay(&[]), Board::empty());
}

// mitra: measures Board::replay ([Line::Whole(Event::Added(1, "a")), Line::Whole(Event::Added(2, "b")), Line::Whole(Event::Done(1)), Line::Whole(Event::Added(2, "again"))]) -> Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }
#[test]
fn replay_builds_the_board() {
    let b = Board::replay(&[
        Line::Whole(added(1, "a")),
        Line::Whole(added(2, "b")),
        Line::Whole(Event::Done(1)),
        Line::Whole(added(2, "again")),
    ]);
    assert_eq!(b, Board { tickets: vec![t(1, "a", true), t(2, "b", false)], next: 3 });
}

// mitra: measures Board::replay ([Line::Cut, Line::Whole(Event::Added(1, "a")), Line::Cut, Line::Whole(Event::Done(1))]) -> Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }
#[test]
fn replay_skips_cut_lines_anywhere() {
    let b = Board::replay(&[
        Line::Cut,
        Line::Whole(added(1, "a")),
        Line::Cut,
        Line::Whole(Event::Done(1)),
    ]);
    assert_eq!(b, Board { tickets: vec![t(1, "a", true)], next: 2 });
}

// mitra: measures Board::is_done (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 1) -> true
#[test]
fn is_done_of_a_done_ticket() {
    let b = Board { tickets: vec![t(1, "a", true), t(2, "b", false)], next: 3 };
    assert!(b.is_done(1));
}

// mitra: measures Board::is_done (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 2) -> false
#[test]
fn is_done_of_an_open_ticket() {
    let b = Board { tickets: vec![t(1, "a", true), t(2, "b", false)], next: 3 };
    assert!(!b.is_done(2));
}

// mitra: measures Board::has (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 1) -> true
#[test]
fn has_a_ticket() {
    assert!(Board { tickets: vec![t(1, "a", false)], next: 2 }.has(1));
}

// mitra: measures Board::has (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 2) -> false
#[test]
fn has_no_ticket() {
    assert!(!Board { tickets: vec![t(1, "a", false)], next: 2 }.has(2));
}

// ---------------------------------------------------------------- Cli

// mitra: measures Cli::read (["add", "fix login"]) -> Command::Add("fix login")
#[test]
fn read_add() {
    assert_eq!(Cli::read(&args(&["add", "fix login"])), Command::Add("fix login".into()));
}

// mitra: measures Cli::read (["done", "3"]) -> Command::Finish(3)
#[test]
fn read_done() {
    assert_eq!(Cli::read(&args(&["done", "3"])), Command::Finish(3));
}

// mitra: measures Cli::read (["list"]) -> Command::List
#[test]
fn read_list() {
    assert_eq!(Cli::read(&args(&["list"])), Command::List);
}

// mitra: measures Cli::read (["show", "3"]) -> Command::Show(3)
#[test]
fn read_show() {
    assert_eq!(Cli::read(&args(&["show", "3"])), Command::Show(3));
}

// mitra: measures Cli::read (["done", "three"]) -> Command::Unknown
#[test]
fn read_done_needs_a_number() {
    assert_eq!(Cli::read(&args(&["done", "three"])), Command::Unknown);
}

// mitra: measures Cli::read (["add"]) -> Command::Unknown
#[test]
fn read_add_needs_a_title() {
    assert_eq!(Cli::read(&args(&["add"])), Command::Unknown);
}

// The model gives no row here (Cli::read has no body): not a measurement.
#[test]
fn read_nothing() {
    assert_eq!(Cli::read(&args(&[])), Command::Unknown);
}

// The model gives no row here (Cli::read has no body): not a measurement.
#[test]
fn read_list_takes_nothing_more() {
    assert_eq!(Cli::read(&args(&["list", "all"])), Command::Unknown);
}

// mitra: measures Cli::plan (Command::Add("fix login"), Board { tickets: [], next: 3 }) -> Step::Write(Event::Added(3, "fix login"), Reply::Id(3))
#[test]
fn plan_add() {
    let b = Board { tickets: vec![], next: 3 };
    assert_eq!(
        Cli::plan(&Command::Add("fix login".into()), &b),
        Step::Write(added(3, "fix login"), Reply::Id(3))
    );
}

// mitra: measures Cli::plan (Command::Finish(9), Board { tickets: [], next: 3 }) -> Step::Fail(Failure::NoSuchTicket)
#[test]
fn plan_finish_of_no_ticket() {
    let b = Board { tickets: vec![], next: 3 };
    assert_eq!(Cli::plan(&Command::Finish(9), &b), Step::Fail(Failure::NoSuchTicket));
}

// mitra: measures Cli::plan (Command::Add(""), Board { tickets: [], next: 1 }) -> Step::Fail(Failure::EmptyTitle)
#[test]
fn plan_add_of_an_empty_title() {
    assert_eq!(Cli::plan(&Command::Add(String::new()), &Board::empty()), Step::Fail(Failure::EmptyTitle));
}

// mitra: measures Cli::plan (Command::Finish(1), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }) -> Step::Write(Event::Done(1), Reply::Nothing)
#[test]
fn plan_finish() {
    let b = Board { tickets: vec![t(1, "a", false)], next: 2 };
    assert_eq!(Cli::plan(&Command::Finish(1), &b), Step::Write(Event::Done(1), Reply::Nothing));
}

// mitra: measures Cli::plan (Command::List, Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }) -> Step::Read(Reply::All([Ticket { id: 1, title: "a", done: true }]))
#[test]
fn plan_list() {
    let b = Board { tickets: vec![t(1, "a", true)], next: 2 };
    assert_eq!(Cli::plan(&Command::List, &b), Step::Read(Reply::All(vec![t(1, "a", true)])));
}

// mitra: measures Cli::plan (Command::Show(2), Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: true }], next: 3 }) -> Step::Read(Reply::One(Ticket { id: 2, title: "b", done: true }))
#[test]
fn plan_show() {
    let b = Board { tickets: vec![t(1, "a", false), t(2, "b", true)], next: 3 };
    assert_eq!(Cli::plan(&Command::Show(2), &b), Step::Read(Reply::One(t(2, "b", true))));
}

// mitra: measures Cli::plan (Command::Show(5), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }) -> Step::Fail(Failure::NoSuchTicket)
#[test]
fn plan_show_of_no_ticket() {
    let b = Board { tickets: vec![t(1, "a", false)], next: 2 };
    assert_eq!(Cli::plan(&Command::Show(5), &b), Step::Fail(Failure::NoSuchTicket));
}

// mitra: measures Cli::plan (Command::Unknown, Board { tickets: [], next: 1 }) -> Step::Fail(Failure::Usage)
#[test]
fn plan_unknown() {
    assert_eq!(Cli::plan(&Command::Unknown, &Board::empty()), Step::Fail(Failure::Usage));
}

// mitra: measures Cli::answer (Step::Write(Event::Added(3, "a"), Reply::Id(3)), true) -> Outcome::Printed(Reply::Id(3))
#[test]
fn answer_a_written_change() {
    let s = Step::Write(added(3, "a"), Reply::Id(3));
    assert_eq!(Cli::answer(&s, true), Outcome::Printed(Reply::Id(3)));
}

// mitra: measures Cli::answer (Step::Write(Event::Added(3, "a"), Reply::Id(3)), false) -> Outcome::Failed(Failure::Storage)
#[test]
fn answer_an_unwritten_change() {
    let s = Step::Write(added(3, "a"), Reply::Id(3));
    assert_eq!(Cli::answer(&s, false), Outcome::Failed(Failure::Storage));
}

// mitra: measures Cli::answer (Step::Write(Event::Done(1), Reply::Nothing), false) -> Outcome::Failed(Failure::Storage)
#[test]
fn answer_an_unwritten_done() {
    let s = Step::Write(Event::Done(1), Reply::Nothing);
    assert_eq!(Cli::answer(&s, false), Outcome::Failed(Failure::Storage));
}

// mitra: measures Cli::answer (Step::Read(Reply::Nothing), false) -> Outcome::Printed(Reply::Nothing)
#[test]
fn answer_a_read() {
    assert_eq!(Cli::answer(&Step::Read(Reply::Nothing), false), Outcome::Printed(Reply::Nothing));
}

// mitra: measures Cli::answer (Step::Fail(Failure::Usage), true) -> Outcome::Failed(Failure::Usage)
#[test]
fn answer_a_failure() {
    assert_eq!(Cli::answer(&Step::Fail(Failure::Usage), true), Outcome::Failed(Failure::Usage));
}

// ---------------------------------------------------------------- printing

#[test]
fn render_prints_as_the_brief_says() {
    assert_eq!(render(&Reply::Id(3)), "#3\n");
    assert_eq!(render(&Reply::Nothing), "");
    assert_eq!(render(&Reply::One(t(1, "fix login", false))), "#1 [ ] fix login\n");
    assert_eq!(
        render(&Reply::All(vec![t(1, "fix login", true), t(2, "b", false)])),
        "#1 [x] fix login\n#2 [ ] b\n"
    );
    assert_ne!(code(Failure::Usage), 0);
    assert_ne!(code(Failure::EmptyTitle), 0);
    assert_ne!(code(Failure::NoSuchTicket), 0);
    assert_ne!(code(Failure::Storage), 0);
}

/// The largest i64 reads back whole, and applying it changes nothing, for it
/// has no next id (the model's integers have no largest one).
#[test]
fn the_largest_id() {
    let e = added(i64::MAX, "x");
    assert_eq!(Codec::decode(&Codec::encode(&e)), Line::Whole(e.clone()));
    assert_eq!(Board::empty().apply(&e), Board::empty());
    assert_eq!(Cli::read(&args(&["show", "99999999999999999999"])), Command::Unknown);
}

#[test]
fn no_id_is_left_after_the_largest() {
    let b = Board { tickets: vec![], next: i64::MAX };
    assert_eq!(Cli::plan(&Command::Add("x".into()), &b), Step::Fail(Failure::Storage));
}
