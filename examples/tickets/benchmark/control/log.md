# Work log

## Round 1
- Command: `cargo test --no-fail-fast`
- Result: 13 passed, 1 failed (`tests::failed_write_then_recovery`, at a torn write that lost only its final `\n`), plus one unused-`mut` warning.
- Cause: before appending after a torn tail, the tool wrote a plain `\n`, which turned the torn (but otherwise checksum-valid) record into a valid line, so an unacknowledged change was read back.
- Change: after a torn tail, write `\0\n` first. A NUL is never in a valid record, so the torn line always fails to decode. Removed the stray `mut`. Added a CLI test that cuts only the last `\n` and then appends.

## Round 2
- Command: `cargo test --no-fail-fast`
- Result: all 14 tests pass (7 unit, 7 CLI), no warnings.
- Change: none.

## Totals
- Rounds: 2
- Lines of code: 294 (src/lib.rs without its test module: 276, src/main.rs: 18)
- Lines of tests: 262 (unit tests in src/lib.rs: 117, tests/cli.rs: 145)
