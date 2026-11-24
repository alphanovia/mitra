# log: tickets, written against tickets.mitra

Every command below ran in this folder after `export PATH=MITRA/target/release:$PATH`. Each block is what Mitra printed, verbatim, followed by `exit N` (the exit code, added by `; echo "exit $?"`).


## Round 1

**Before this round:** Read MITRA/README.md, docs/getting-started.md and docs/reference/ (chapters 1-8 and the code pages). Ran `mitra check tickets.mitra` alone first (same output as round 1's check). Wrote `Cargo.toml` (no dependencies, `[workspace]` at the end), `src/lib.rs` (types, Disk, Codec, Board, Cli, `run`), `src/main.rs` (printing and exit codes), `src/tests.rs` (unit tests, 41 with a `measures` row) and `tests/cli.rs` (6 end-to-end tests of the binary). `cargo test`: all pass.

```text
$ mitra check tickets.mitra
finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `sticky`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `increasing`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `skips`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `acks`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::answer

finding[A007]: claim `holds`: unknown, no value for `Codec::encode` at some of the 10 points tried
 --> Codec::reads

check: well formed; agree: 21 comparisons, 15 hold, 6 findings
exit 0
```

```text
$ mitra explain tickets.mitra --code src --code tests
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 49 tests, 49 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 41 measurements, 0 reviews, 12 ties
unsupported  Board::apply::grows 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::apply::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::increasing 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::replay::increasing} Board::increasing {Board::increasing} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay::increasing 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay::skips 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::skips 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::replay::skips} Board::skips {Board::skips} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::apply::sticky} Board::sticky {Board::sticky} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::acks 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Cli::answer::acks} Cli::acks {Cli::acks} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::answer::acks 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::reads::holds 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::whole 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Codec::reads::holds} Codec::whole {Codec::whole} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::cuts_last 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::durable 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::honest 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 6 only the model says)
unsupported  {Disk::cuts_last, Codec::whole, Board::skips} Tickets::honest {Tickets::honest} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::keeps 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 6 only the model says)
unsupported  {Disk::durable, Cli::acks, Board::skips} Tickets::keeps {Tickets::keeps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 3 only the model says)
unsupported  {Board::sticky} Tickets::sticky {Tickets::sticky} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::unique 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 3 only the model says)
unsupported  {Board::increasing} Tickets::unique {Tickets::unique} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::read at src/lib.rs/Disk/read 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::append at src/lib.rs/Disk/append 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::encode at src/lib.rs/Codec/encode 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::decode at src/lib.rs/Codec/decode 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::reads at src/lib.rs/Codec/reads 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::apply at src/lib.rs/Board/apply 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay at src/lib.rs/Board/replay 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::is_done at src/lib.rs/Board/is_done 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::has at src/lib.rs/Board/has 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::read at src/lib.rs/Cli/read 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::plan at src/lib.rs/Cli/plan 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::answer at src/lib.rs/Cli/answer 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_cut_in_the_title)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix login$") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_whole)
explained    Codec::reads::holds at (Done(3), "done 3") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_done_cut_before_the_end)
explained    Codec::reads::holds at (Done(12), "done 1") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_done_cut_in_the_id)
explained    Codec::reads::holds at (Added(2, "a$b"), "") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_with_an_end_in_the_title)
explained    Codec::reads::holds at (Added(7, "x"), "something else") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_against_no_prefix)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Added(1, "fix login"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Done(4))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Done(4)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Done(4)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Cut])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Cut]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Cut]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    correct(Board::replay, ([])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    Board::replay::skips at ([]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    Board::replay::increasing at ([]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    correct(Board::replay, ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    Board::replay::skips at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    Board::replay::increasing at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::is_done_of_a_done_ticket)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::is_done_of_an_open_ticket)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::has_a_ticket)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::has_no_ticket)
explained    correct(Cli::read, (["add", "fix login"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_add)
explained    correct(Cli::read, (["done", "3"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_done)
explained    correct(Cli::read, (["list"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_list)
explained    correct(Cli::read, (["show", "3"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_show)
explained    correct(Cli::read, (["done", "three"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_done_needs_a_number)
explained    correct(Cli::read, (["add"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_add_needs_a_title)
explained    correct(Cli::plan, (Add("fix login"), Board { tickets: [], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_add)
explained    correct(Cli::plan, (Finish(9), Board { tickets: [], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_finish_of_no_ticket)
explained    correct(Cli::plan, (Add(""), Board { tickets: [], next: 1 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_add_of_an_empty_title)
explained    correct(Cli::plan, (Finish(1), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_finish)
explained    correct(Cli::plan, (List, Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_list)
explained    correct(Cli::plan, (Show(2), Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: true }], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_show)
explained    correct(Cli::plan, (Show(5), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_show_of_no_ticket)
explained    correct(Cli::plan, (Unknown, Board { tickets: [], next: 1 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_unknown)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), true)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_written_change)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), true) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_written_change)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_change)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_change)
explained    correct(Cli::answer, (Write(Done(1), Nothing), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_done)
explained    Cli::answer::acks at (Write(Done(1), Nothing), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_done)
explained    correct(Cli::answer, (Read(Nothing), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_read)
explained    Cli::answer::acks at (Read(Nothing), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_read)
explained    correct(Cli::answer, (Fail(Usage), true)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_failure)
explained    Cli::answer::acks at (Fail(Usage), true) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_failure)

finding[X001]: `Cli::read` gave Unknown at ([]): nothing in the model says what it should give there, and nothing measured reads it
 --> measurements::26

finding[X001]: `Cli::read` gave Unknown at (["list", "all"]): nothing in the model says what it should give there, and nothing measured reads it
 --> measurements::27

finding[X004]: at (Added(1, "fix login"), "added 1 fix"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::0

finding[X004]: at (Added(1, "fix login"), "added 1 fix login$"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::1

finding[X004]: at (Done(3), "done 3"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::2

finding[X004]: at (Done(12), "done 1"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::3

finding[X004]: at (Added(2, "a$b"), ""), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::4

finding[X004]: at (Added(7, "x"), "something else"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::5

hint[X007]: `Cli::read` gave one value at each point measured: copy them as examples? `examples { ([]) -> Unknown, (["list", "all"]) -> Unknown }`
 --> Cli::read

ask[X012]: only the model says that `Board::replay::increasing` gives `Board::increasing`: review the link
 --> Board::increasing
  = help: once reviewed, write in the store `review Board::increasing from holds by "human:<name>";`

ask[X012]: only the model says that `Board::replay::skips` gives `Board::skips`: review the link
 --> Board::skips
  = help: once reviewed, write in the store `review Board::skips from holds by "human:<name>";`

ask[X012]: only the model says that `Board::apply::sticky` gives `Board::sticky`: review the link
 --> Board::sticky
  = help: once reviewed, write in the store `review Board::sticky from holds by "human:<name>";`

ask[X012]: only the model says that `Cli::answer::acks` gives `Cli::acks`: review the link
 --> Cli::acks
  = help: once reviewed, write in the store `review Cli::acks from holds by "human:<name>";`

ask[X012]: only the model says that `Codec::reads::holds` gives `Codec::whole`: review the link
 --> Codec::whole
  = help: once reviewed, write in the store `review Codec::whole from holds by "human:<name>";`

ask[X012]: only words say `Disk::cuts_last`, and nothing below gives it: review it, seeing the test that shows it if one does
 --> Disk::cuts_last
  = help: once reviewed, write in the store `review Disk::cuts_last holds by "human:<name>" seeing "test:<name>";`

ask[X012]: only words say `Disk::durable`, and nothing below gives it: review it, seeing the test that shows it if one does
 --> Disk::durable
  = help: once reviewed, write in the store `review Disk::durable holds by "human:<name>" seeing "test:<name>";`

ask[X012]: only the model says that (Disk::cuts_last, Codec::whole, Board::skips) together give `Tickets::honest`: review the link
 --> Tickets::honest
  = help: once reviewed, write in the store `review Tickets::honest from holds by "human:<name>";`

ask[X012]: only the model says that (Disk::durable, Cli::acks, Board::skips) together give `Tickets::keeps`: review the link
 --> Tickets::keeps
  = help: once reviewed, write in the store `review Tickets::keeps from holds by "human:<name>";`

ask[X012]: only the model says that `Board::sticky` gives `Tickets::sticky`: review the link
 --> Tickets::sticky
  = help: once reviewed, write in the store `review Tickets::sticky from holds by "human:<name>";`

ask[X012]: only the model says that `Board::increasing` gives `Tickets::unique`: review the link
 --> Tickets::unique
  = help: once reviewed, write in the store `review Tickets::unique from holds by "human:<name>";`

explain: 102 propositions, 64 explained, 0 contradicted, 38 unsupported; 8 findings, 1 hints, 11 reviews asked
exit 0
```

```text
$ mitra blame tickets.mitra --code src --code tests
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 49 tests, 49 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 41 measurements, 0 reviews, 12 ties
blame: nothing conflicts, nothing to blame
exit 0
```


**What I changed after it:** 
- X001 on `Cli::read` at `([])` and `(["list", "all"])`: `Cli::read` has no body and the model gives no example there, so a row there tells Mitra nothing. I removed those two `measures` comments (the tests stay, as plain tests; R004 lists them).
- X004 on every `Codec::reads` row: `reads` calls `encode`, which has no body. I tried to give what `encode` gave as an input. Probe (not a round; a throwaway store in the scratchpad, run with `--no-test`), each form refused:
  - in a `measures` comment, `(Codec::reads::e = ..., Codec::encode::t = "...") -> Codec::reads::ok = true` → `` finding[R001]: expected a name, found `(`, at column 10 of what follows `mitra:` ``
  - in a store, `measure Codec::reads (Event::Done(3), "done 3", Codec::encode::t = "done 3$") -> true by "human:x";` → `` error[P007]: `Codec::encode::t` is no variant of an enum of the model, nor a value ``
  - `measure Codec::reads (e = Event::Done(3), ...)` → `` error[P007]: `e` is no variant ... ``; `measure (Codec::reads::e = ...) -> Codec::reads::ok = true` → `` error[P003]: expected a name, found `(` ``; `... -> true given Codec::encode::t = ...` and `... with ...` → `` error[P003]: expected `by`, found ... ``.
  The reference shows the named form (`sun::light::w = 2.0`) only in model examples and design notes I may not read, so I left the X004 findings. The claim `Codec::reads::holds` is still judged at each row (it reads only `ok`).
- Code: a new log file's directory is now fsynced on creation, so that the file's name survives a crash too (for `Disk::durable`).



## Round 2

**Before this round:** The changes above.

```text
$ mitra check tickets.mitra
finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `sticky`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `increasing`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `skips`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `acks`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::answer

finding[A007]: claim `holds`: unknown, no value for `Codec::encode` at some of the 10 points tried
 --> Codec::reads

check: well formed; agree: 21 comparisons, 15 hold, 6 findings
exit 0
```

```text
$ mitra explain tickets.mitra --code src --code tests
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 50 tests, 50 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_list_takes_nothing_more` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::the_largest_id_is_refused` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 39 measurements, 0 reviews, 12 ties
unsupported  Board::apply::grows 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::apply::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::increasing 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::replay::increasing} Board::increasing {Board::increasing} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay::increasing 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay::skips 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::skips 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::replay::skips} Board::skips {Board::skips} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Board::apply::sticky} Board::sticky {Board::sticky} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::acks 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Cli::answer::acks} Cli::acks {Cli::acks} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::answer::acks 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::reads::holds 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::whole 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Codec::reads::holds} Codec::whole {Codec::whole} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::cuts_last 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::durable 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::honest 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 6 only the model says)
unsupported  {Disk::cuts_last, Codec::whole, Board::skips} Tickets::honest {Tickets::honest} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::keeps 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 6 only the model says)
unsupported  {Disk::durable, Cli::acks, Board::skips} Tickets::keeps {Tickets::keeps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::sticky 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 3 only the model says)
unsupported  {Board::sticky} Tickets::sticky {Tickets::sticky} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Tickets::unique 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 3 only the model says)
unsupported  {Board::increasing} Tickets::unique {Tickets::unique} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::read at src/lib.rs/Disk/read 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Disk::append at src/lib.rs/Disk/append 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::encode at src/lib.rs/Codec/encode 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::decode at src/lib.rs/Codec/decode 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Codec::reads at src/lib.rs/Codec/reads 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::apply at src/lib.rs/Board/apply 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::replay at src/lib.rs/Board/replay 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::is_done at src/lib.rs/Board/is_done 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Board::has at src/lib.rs/Board/has 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::read at src/lib.rs/Cli/read 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::plan at src/lib.rs/Cli/plan 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::answer at src/lib.rs/Cli/answer 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_cut_in_the_title)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix login$") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_whole)
explained    Codec::reads::holds at (Done(3), "done 3") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_done_cut_before_the_end)
explained    Codec::reads::holds at (Done(12), "done 1") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_done_cut_in_the_id)
explained    Codec::reads::holds at (Added(2, "a$b"), "") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_with_an_end_in_the_title)
explained    Codec::reads::holds at (Added(7, "x"), "something else") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::reads_added_against_no_prefix)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Added(1, "fix login"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e"))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Done(4))) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Done(4)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Done(4)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Cut])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Cut]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Cut]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    correct(Board::replay, ([])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    Board::replay::skips at ([]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    Board::replay::increasing at ([]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    correct(Board::replay, ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    Board::replay::skips at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    Board::replay::increasing at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::is_done_of_a_done_ticket)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::is_done_of_an_open_ticket)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::has_a_ticket)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::has_no_ticket)
explained    correct(Cli::read, (["add", "fix login"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_add)
explained    correct(Cli::read, (["done", "3"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_done)
explained    correct(Cli::read, (["list"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_list)
explained    correct(Cli::read, (["show", "3"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_show)
explained    correct(Cli::read, (["done", "three"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_done_needs_a_number)
explained    correct(Cli::read, (["add"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::read_add_needs_a_title)
explained    correct(Cli::plan, (Add("fix login"), Board { tickets: [], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_add)
explained    correct(Cli::plan, (Finish(9), Board { tickets: [], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_finish_of_no_ticket)
explained    correct(Cli::plan, (Add(""), Board { tickets: [], next: 1 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_add_of_an_empty_title)
explained    correct(Cli::plan, (Finish(1), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_finish)
explained    correct(Cli::plan, (List, Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_list)
explained    correct(Cli::plan, (Show(2), Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: true }], next: 3 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_show)
explained    correct(Cli::plan, (Show(5), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_show_of_no_ticket)
explained    correct(Cli::plan, (Unknown, Board { tickets: [], next: 1 })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::plan_unknown)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), true)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_written_change)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), true) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_written_change)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_change)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_change)
explained    correct(Cli::answer, (Write(Done(1), Nothing), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_done)
explained    Cli::answer::acks at (Write(Done(1), Nothing), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_an_unwritten_done)
explained    correct(Cli::answer, (Read(Nothing), false)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_read)
explained    Cli::answer::acks at (Read(Nothing), false) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_read)
explained    correct(Cli::answer, (Fail(Usage), true)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_failure)
explained    Cli::answer::acks at (Fail(Usage), true) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::answer_a_failure)

finding[X004]: at (Added(1, "fix login"), "added 1 fix"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::0

finding[X004]: at (Added(1, "fix login"), "added 1 fix login$"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::1

finding[X004]: at (Done(3), "done 3"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::2

finding[X004]: at (Done(12), "done 1"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::3

finding[X004]: at (Added(2, "a$b"), ""), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::4

finding[X004]: at (Added(7, "x"), "something else"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::5

ask[X012]: only the model says that `Board::replay::increasing` gives `Board::increasing`: review the link
 --> Board::increasing
  = help: once reviewed, write in the store `review Board::increasing from holds by "human:<name>";`

ask[X012]: only the model says that `Board::replay::skips` gives `Board::skips`: review the link
 --> Board::skips
  = help: once reviewed, write in the store `review Board::skips from holds by "human:<name>";`

ask[X012]: only the model says that `Board::apply::sticky` gives `Board::sticky`: review the link
 --> Board::sticky
  = help: once reviewed, write in the store `review Board::sticky from holds by "human:<name>";`

ask[X012]: only the model says that `Cli::answer::acks` gives `Cli::acks`: review the link
 --> Cli::acks
  = help: once reviewed, write in the store `review Cli::acks from holds by "human:<name>";`

ask[X012]: only the model says that `Codec::reads::holds` gives `Codec::whole`: review the link
 --> Codec::whole
  = help: once reviewed, write in the store `review Codec::whole from holds by "human:<name>";`

ask[X012]: only words say `Disk::cuts_last`, and nothing below gives it: review it, seeing the test that shows it if one does
 --> Disk::cuts_last
  = help: once reviewed, write in the store `review Disk::cuts_last holds by "human:<name>" seeing "test:<name>";`

ask[X012]: only words say `Disk::durable`, and nothing below gives it: review it, seeing the test that shows it if one does
 --> Disk::durable
  = help: once reviewed, write in the store `review Disk::durable holds by "human:<name>" seeing "test:<name>";`

ask[X012]: only the model says that (Disk::cuts_last, Codec::whole, Board::skips) together give `Tickets::honest`: review the link
 --> Tickets::honest
  = help: once reviewed, write in the store `review Tickets::honest from holds by "human:<name>";`

ask[X012]: only the model says that (Disk::durable, Cli::acks, Board::skips) together give `Tickets::keeps`: review the link
 --> Tickets::keeps
  = help: once reviewed, write in the store `review Tickets::keeps from holds by "human:<name>";`

ask[X012]: only the model says that `Board::sticky` gives `Tickets::sticky`: review the link
 --> Tickets::sticky
  = help: once reviewed, write in the store `review Tickets::sticky from holds by "human:<name>";`

ask[X012]: only the model says that `Board::increasing` gives `Tickets::unique`: review the link
 --> Tickets::unique
  = help: once reviewed, write in the store `review Tickets::unique from holds by "human:<name>";`

explain: 102 propositions, 64 explained, 0 contradicted, 38 unsupported; 6 findings, 0 hints, 11 reviews asked
exit 0
```

```text
$ mitra blame tickets.mitra --code src --code tests
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 50 tests, 50 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_list_takes_nothing_more` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::the_largest_id_is_refused` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 39 measurements, 0 reviews, 12 ties
blame: nothing conflicts, nothing to blame
exit 0
```


**What I changed after it:** 
Every remaining unsupported proposition is one no test can hold here: the claims over all inputs (`Board::apply::grows`, ...), the ties themselves, the links of each ensure's `from`, and `Disk::durable` / `Disk::cuts_last`, said only in words. The model has no rule that would carry the tested points to an `all`, and I may not change it. So I checked each myself and wrote `evidence.mitra` (reviews by `agent:claude-opus-5-5`); what each rests on is below.

Before reviewing the ties I looked again at where Rust's `i64` differs from the model's unbounded `integer`:
- `Board::apply` on `Added(i64::MAX, t)`: `id + 1` would overflow. Now it is ignored (`checked_add`), which keeps `grows`, `sticky` and `increasing`. (Between rounds I first tried refusing `i64::MAX` in the codec instead, and undid it: that would break `Codec::reads::holds` at that point.)
- `Cli::plan` of `Add` when `b.next == i64::MAX`: the event would be ignored by `apply` but acknowledged, breaking `keeps`. Now it fails with `Storage` and writes nothing.
- Both are at ids the tool never makes (ids count up from 1); only a hand-edited log reaches them. Tests `the_largest_id` and `no_id_is_left_after_the_largest`.

What the reviews rest on:
- Ties (`review Board::apply holds`, ...): I read each tied Rust function against the model's body or examples, case by case. Same result everywhere except at `i64::MAX` as above. `Cli::read` has only examples; the Rust gives each of them (all six are measured).
- `Board::apply::grows`: `Added` either returns `self` or appends one ticket and sets `next` to `id + 1 > id >= self.next`; `Done` keeps the length and `next`. `Board::apply::sticky`: `Done` maps `done` to `t.done || ...`, never false from true; `Added` keeps the old tickets as they were.
- `Board::replay::increasing`: by induction on the fold: a ticket is appended only with `id >= next` and then `next = id + 1`, so ids rise strictly and all stay below `next`. `Board::replay::skips`: the fold's `Cut` arm returns `b` unchanged.
- `Cli::answer::acks`: by the three arms; `Write` gives `Printed` only when `written`.
- `Codec::reads::holds`: `encode` escapes backslash, newline, CR and `$` in the title, so the final `$` is the line's only one; `decode` requires a final `$` and no other, so every strict prefix (byte cuts included: `Disk::read` turns a cut UTF-8 sequence into U+FFFD, which is not `$`) reads as `Cut`; `unescape(escape(t)) == t` and ids print and parse back for every i64. Test `every_cut_of_every_line_is_cut` checks every cut of six events.
- `Disk::durable`: `append` returns true only after `write_all` and `sync_all` both succeed, the file is opened in append mode, the directory is synced when the file is created, and a cut line left by an earlier crash gets a newline before the next line, so a whole line is never glued to it. Seen in `an_acknowledged_change_is_on_disk` (each acknowledged add is in the file, newline-ended, read back by the next run). A real power cut was not simulated: the review rests on fsync doing what POSIX says.
- `Disk::cuts_last`: an assumption about the file system, which I cannot test. What I checked is the part the code owns: the tool only ever appends, one line per run, after a synced read, so the unsynced bytes a crash can lose are those of the line being written. `a_cut_last_line_is_ignored` cuts the last line at every byte and checks the list, then appends after a cut.
- Links: `Board::increasing`, `Board::skips`, `Board::sticky`, `Cli::acks`, `Codec::whole` each restate the one claim they come from. `Tickets::unique` from `Board::increasing`: strictly rising ids are unique. `Tickets::sticky` from `Board::sticky`: replay only applies events. `Tickets::honest`: only the last line may be cut, a cut reads as `Cut`, and `Cut` changes nothing. `Tickets::keeps`: acknowledged only once written, written lines read back whole, a trailing cut changes nothing; **but** see the spec note below: it also needs commands not to interleave, which the code gets from a file lock, seen in `concurrent_adds_get_unique_ids`.



## Round 3

**Before this round:** `evidence.mitra` written; the `i64` edge cases fixed with two tests.

```text
$ mitra check tickets.mitra
finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `sticky`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Board::apply

finding[A007]: claim `increasing`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `skips`: unknown, not decided by bounds over all of what it reads, nor at 57 points
 --> Board::replay

finding[A007]: claim `acks`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::answer

finding[A007]: claim `holds`: unknown, no value for `Codec::encode` at some of the 10 points tried
 --> Codec::reads

check: well formed; agree: 21 comparisons, 15 hold, 6 findings
exit 0
```

```text
$ mitra explain tickets.mitra --code src --code tests --store evidence.mitra
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 51 tests, 51 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::no_id_is_left_after_the_largest` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_list_takes_nothing_more` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::the_largest_id` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 39 measurements, 29 reviews, 12 ties
explained    Board::apply::grows 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::apply::sticky 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::increasing 0.63 (b 0.26, d 0.00, u 0.74: 2 for, 0 against, agent:claude-opus-5-5)
explained    {Board::replay::increasing} Board::increasing {Board::increasing} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::replay::increasing 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::replay::skips 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::skips 0.63 (b 0.26, d 0.00, u 0.74: 2 for, 0 against, agent:claude-opus-5-5)
explained    {Board::replay::skips} Board::skips {Board::skips} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::sticky 0.63 (b 0.26, d 0.00, u 0.74: 2 for, 0 against, agent:claude-opus-5-5)
explained    {Board::apply::sticky} Board::sticky {Board::sticky} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Cli::acks 0.63 (b 0.26, d 0.00, u 0.74: 2 for, 0 against, agent:claude-opus-5-5)
explained    {Cli::answer::acks} Cli::acks {Cli::acks} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Cli::answer::acks 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::reads::holds 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::whole 0.63 (b 0.26, d 0.00, u 0.74: 2 for, 0 against, agent:claude-opus-5-5)
explained    {Codec::reads::holds} Codec::whole {Codec::whole} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Disk::cuts_last 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Disk::durable 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Tickets::honest 0.54 (b 0.07, d 0.00, u 0.93: 6 for, 0 against, agent:claude-opus-5-5)
explained    {Disk::cuts_last, Codec::whole, Board::skips} Tickets::honest {Tickets::honest} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Tickets::keeps 0.54 (b 0.07, d 0.00, u 0.93: 6 for, 0 against, agent:claude-opus-5-5)
explained    {Disk::durable, Cli::acks, Board::skips} Tickets::keeps {Tickets::keeps} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Tickets::sticky 0.60 (b 0.20, d 0.00, u 0.80: 3 for, 0 against, agent:claude-opus-5-5)
explained    {Board::sticky} Tickets::sticky {Tickets::sticky} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Tickets::unique 0.60 (b 0.20, d 0.00, u 0.80: 3 for, 0 against, agent:claude-opus-5-5)
explained    {Board::increasing} Tickets::unique {Tickets::unique} 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Disk::read at src/lib.rs/Disk/read 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Disk::append at src/lib.rs/Disk/append 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::encode at src/lib.rs/Codec/encode 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::decode at src/lib.rs/Codec/decode 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::reads at src/lib.rs/Codec/reads 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::apply at src/lib.rs/Board/apply 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::replay at src/lib.rs/Board/replay 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::is_done at src/lib.rs/Board/is_done 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Board::has at src/lib.rs/Board/has 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Cli::read at src/lib.rs/Cli/read 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Cli::plan at src/lib.rs/Cli/plan 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Cli::answer at src/lib.rs/Cli/answer 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:claude-opus-5-5)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Codec::reads::holds at (Added(1, "fix login"), "added 1 fix login$") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Codec::reads::holds at (Done(3), "done 3") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Codec::reads::holds at (Done(12), "done 1") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Codec::reads::holds at (Added(2, "a$b"), "") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Codec::reads::holds at (Added(7, "x"), "something else") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Added(1, "fix login"))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Added(1, "fix login")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_adds_a_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 }, Done(1)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_twice_stays_done)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b"))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(1, "b")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_ignores_an_old_id)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: false }], next: 3 }, Done(2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_marks_only_its_ticket)
explained    correct(Board::apply, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e"))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, Added(5, "e")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_jumps_to_a_higher_id)
explained    correct(Board::apply, (Board { tickets: [], next: 1 }, Done(4))) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Board::apply::grows at (Board { tickets: [], next: 1 }, Done(4)) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::apply::sticky at (Board { tickets: [], next: 1 }, Done(4)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::apply_done_of_no_ticket_changes_nothing)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Cut])) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Cut]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_a_cut_last_line)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Cut]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Board::replay, ([])) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::replay::skips at ([]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_of_nothing_is_empty)
explained    Board::replay::increasing at ([]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Board::replay, ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))])) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::replay::skips at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_builds_the_board)
explained    Board::replay::increasing at ([Whole(Added(1, "a")), Whole(Added(2, "b")), Whole(Done(1)), Whole(Added(2, "again"))]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Board::replay, ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))])) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    Board::replay::skips at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::replay_skips_cut_lines_anywhere)
explained    Board::replay::increasing at ([Cut, Whole(Added(1, "a")), Cut, Whole(Done(1))]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 1)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Board::is_done, (Board { tickets: [Ticket { id: 1, title: "a", done: true }, Ticket { id: 2, title: "b", done: false }], next: 3 }, 2)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 1)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Board::has, (Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 }, 2)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["add", "fix login"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["done", "3"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["list"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["show", "3"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["done", "three"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::read, (["add"])) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    correct(Cli::plan, (Add("fix login"), Board { tickets: [], next: 3 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Finish(9), Board { tickets: [], next: 3 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Add(""), Board { tickets: [], next: 1 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Finish(1), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (List, Board { tickets: [Ticket { id: 1, title: "a", done: true }], next: 2 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Show(2), Board { tickets: [Ticket { id: 1, title: "a", done: false }, Ticket { id: 2, title: "b", done: true }], next: 3 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Show(5), Board { tickets: [Ticket { id: 1, title: "a", done: false }], next: 2 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::plan, (Unknown, Board { tickets: [], next: 1 })) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), true)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), true) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::answer, (Write(Added(3, "a"), Id(3)), false)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Cli::answer::acks at (Write(Added(3, "a"), Id(3)), false) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::answer, (Write(Done(1), Nothing), false)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Cli::answer::acks at (Write(Done(1), Nothing), false) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::answer, (Read(Nothing), false)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Cli::answer::acks at (Read(Nothing), false) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::answer, (Fail(Usage), true)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:claude-opus-5-5)
explained    Cli::answer::acks at (Fail(Usage), true) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:claude-opus-5-5, machine:mitra-check)

finding[X004]: at (Added(1, "fix login"), "added 1 fix"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::0

finding[X004]: at (Added(1, "fix login"), "added 1 fix login$"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::1

finding[X004]: at (Done(3), "done 3"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::2

finding[X004]: at (Done(12), "done 1"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::3

finding[X004]: at (Added(2, "a$b"), ""), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::4

finding[X004]: at (Added(7, "x"), "something else"), `Codec::reads` needs what `Codec::encode` gave, which nothing gives: measure it as an input
 --> measurements::5

explain: 102 propositions, 102 explained, 0 contradicted, 0 unsupported; 6 findings, 0 hints, 0 reviews asked
exit 0
```

```text
$ mitra blame tickets.mitra --code src --code tests --store evidence.mitra
ran `cargo test --no-fail-fast -- --show-output` in <run folder>: 51 tests, 51 passed, 0 failed, 0 ignored

finding[R004]: the test `tests::no_id_is_left_after_the_largest` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::every_cut_of_every_line_is_cut` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_list_takes_nothing_more` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::read_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_prints_as_the_brief_says` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::the_largest_id` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_failed_write_is_not_acknowledged` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `concurrent_adds_get_unique_ids` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `add_done_list_show` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `failures_print_nothing` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `a_cut_last_line_is_ignored` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `an_acknowledged_change_is_on_disk` measures nothing the reader knows
 --> cargo test:0

evidence: 39 measurements, 29 reviews, 12 ties
blame: nothing conflicts, nothing to blame
exit 0
```


**What I changed after it:** Nothing: every proposition is explained. Stopped.


## Notes on the spec (tickets.mitra, unchanged)

- `Tickets::keeps` is `from (Disk::durable, Cli::acks, Board::skips)`, but those three do not give it alone. Two runs at once can both read the same board, both plan `Added(b.next, ...)` with the same id, and both get acknowledged; `apply` then ignores the second (`id < self.next`), so an acknowledged change is lost with no crash at all. The model needs something like "commands do not interleave" (the code holds the log locked from read to append). I reviewed the link as holding of this code, with that test, and say so here.
- The model's integers have no largest value; the code's `i64` does. The two places that differ (`apply` and `plan` at `i64::MAX`) are noted above.
- `Cli::read` has no body: anything but the six examples (`[]`, `["list", "all"]`, `["done", "-1"]`, ...) is the code's to decide, and Mitra has nothing to measure it against (X001). The code gives `Unknown` for anything not of the four shapes `add T`, `done N`, `list`, `show N` (N a decimal i64, so `done -1` is `Finish(-1)` and then fails with NoSuchTicket).
- `Codec::reads` calls the opaque `encode` and `decode`; Mitra cannot compute what `reads` should give at a row (X004) and I found no documented way to give `encode`'s value in a row.

## Summary

- Rounds: 3.
- Explain summary line of each round:
  - Round 1: `explain: 102 propositions, 64 explained, 0 contradicted, 38 unsupported; 8 findings, 1 hints, 11 reviews asked`
  - Round 2: `explain: 102 propositions, 64 explained, 0 contradicted, 38 unsupported; 6 findings, 0 hints, 11 reviews asked`
  - Round 3: `explain: 102 propositions, 102 explained, 0 contradicted, 0 unsupported; 6 findings, 0 hints, 0 reviews asked`
- Lines (`wc -l`): code 424 (src/lib.rs 397, src/main.rs 27); tests 490 (src/tests.rs 360, tests/cli.rs 130).
- 51 tests (45 unit, 6 end-to-end), all passing; 39 with a `measures` row.
