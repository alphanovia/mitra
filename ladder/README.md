# The ladder

Models of growing difficulty, kept in the repository and kept passing,
to test the language and the trust model on something real before
deciding more of either.

**The rule.** A rung is a folder with five parts, and a test keeps them
together:

| Part | Where |
|---|---|
| the model | `model/model.mitra` |
| the Rust code, tied to it, with its tests | `src/lib.rs` |
| the ledger | `model/evidence.mitra` |
| the expected report (`mitra assess`) | `model/expected.txt` |
| seeded breaks, each with what a user would report | `breaks.txt`, last run in `breaks-report.txt` |

Each rung is there for one question. A rung is not started before the
one under it has its five parts.

| Rung | Subject | The question it is there for | State |
|---|---|---|---|
| 1 | [clock](1-clock) | a finite domain; sample and review evidence | five parts |
| 2 | [account](2-account) | a record and an invariant | five parts |
| 3 | checksum or average | what an operation does on a machine integer (overflow) | not started |
| 4 | sort, dedupe | collections, claims over a whole domain | not started |
| 5 | parse a config line | strings, `Result` | not started |
| 6 | a store on disk with a clock | boundaries, evidence on an `assume` | not started |
| 7 | `Store<K, V>` with two implementers | contracts, generics | not started |
| 8 | two stores, one importing the other | promises across stores | not started |
| 9 | one request handler, a store and a clock behind contracts | a slice of a webapp | not started |
| 10 | Mitra itself | `crates/*/model` | exists |

## Running a rung

```sh
cargo test -p ladder-clock                 # its own tests
cargo test -p mitra-cli --test ladder      # the report is the expected one
scripts/rung.sh ladder/1-clock             # after a change, committed: record the ledger and the report again
python3 scripts/breaks.py ladder/1-clock   # break the code one way at a time
cd ladder/1-clock && mitra assess          # read the report (it writes the ties again)
```

The reviews in the ledgers are an agent's, the one that wrote the model
and the code (`agent:anthropic/fable-5.2/ladder`). No person vouched
for anything here.
