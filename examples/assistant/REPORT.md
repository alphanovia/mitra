# The assistant, built against its model: report (night of 2026-11-25)

Final, 20:55 UTC. Main at a5001e2.

## 1. The work

**Asked (20:01):** code the local server and the CLI, with a fake LLM in place of the real machines. Then test, above all with malicious input. Reason about what a bad input would need to cross every obstacle and act with privilege. End with a model that is as explained as possible and has no contradiction. Two threads: a builder (this one) and a challenger.

**Built** (main, `examples/assistant/`, its own cargo project):

- **The model** (`src/*.mitra`, 19 files) now sits next to the Rust it describes.
- **A fake world** (`src/fake.rs`) plays both LLMs, speech, the tools, the accounts, the codes and the clock. Tests script it. Its work model is gullible on purpose: it obeys any line that starts `CALL`. `fake::performed()` lists every tool that really ran, which makes it the privilege detector.
- **The server**:
  - the entry (`gateway`): tokens, household, size, ids;
  - the store: fixed bodies, notes only added;
  - the worker: take-up as its own write, one message at a time, crash recovery;
  - the harness: identity, plain code, gate, purpose, the loop, policy, guard, reporter;
  - the owner's yes (`confirm`);
  - an in-process server (`Local`).
- **Two binaries**, speaking a line protocol over TCP:
  - `cargo run --bin assistant-server`
  - `cargo run --bin assistant-cli -- --token cli "hello"` (tokens: `cli`, `owner-phone`, `wearable`, `other-phone`)
- **87 tests**:
  - 32 unit tests, mine, with Mitra rows;
  - 28 scenarios, mine;
  - 27 attacks, the challenger's (`tests/attack_*.rs`).

**Commits** (8 on main): 6680f6b, 75e7d1a, 551542a, dce6f68 and 52b2e7e (challenger), c75ae77, 9d15b1d (challenger, round 2), a5001e2.

## 2. Metrics

### Size
| | Before (32ef79a) | Now |
|---|---|---|
| Model files / lines | 18 / 1,720 | 19 / 2,095 |
| Parts with functions / enum and record shorthands | | 36 / 35 |
| Functions / ensures / requires / claims | | 134 / 104 / 113 / 48 |
| Example rows / end-to-end rows | | 107 / 19 |
| Choices to confirm | 14 | 16 |
| Rust: product code / unit tests in src | 0 | 2,861 / 648 lines |
| Rust: scenarios and attacks | 0 | 1,152 lines |
| Ties (`implements`) / tests tied to ensures (`supports`) | 0 | 157 / 63 |
| Reviews in evidence.mitra (all by the building agent) | 0 | 165 |

### Check (`mitra check src/system.mitra`)
| Stage | Comparisons | Hold | Unknown |
|---|---|---|---|
| Model only (32ef79a) | 116 | 97 | 19 |
| Challenger's model holes folded in | 130 | 107 | 23 |
| After K1–K11, and after K12–K14 | 132 | 109 | 23 |

The 4 new unknowns are Mitra gaps of the kinds already listed (L, P). Example: `Gateway::may_post` was decided while `Device` was finite. It takes a `Holder` now, which holds integers.

### Explain (propositions / explained / assumed / contradicted / unsupported; reviews asked)
| Stage | Prop. | Expl. | Assumed | Contr. | Unsup. | Asks |
|---|---|---|---|---|---|---|
| 0. Model only, `--no-test` (32ef79a) | 392 | 103 | 60 | 0 | 229 | 90 |
| 1. Code, unit tests and scenarios, first run | 674 | 254 | 66 | 0 | 354 | 102 |
| 2. After fixing what the reader refused (see 3.2) | 724 | 298 | 72 | 0 | 354 | 102 |
| 3. Tests tied to ensures (`supports`) | 725 | 299 | 139 | 0 | 287 | 120 |
| 4. My own test wrong, for one run | 725 | | | 11 | | |
| 5. Reviews of ensures (seeing tests) and links | 725 | 416 | 22 | 0 | 287 | 33 |
| 6. Plus reviews of ties of functions with a body | 725 | 486 | 22 | 0 | 217 | 33 |
| 7. Challenger's attack tests open (its run) | | | | 157 | | |
| 8. K1–K11 closed | 749 | 506 | 22 | 0 | 221 | 33 |
| 9. K12–K14 closed, before new reviews | 754 | 506 | 31 | 0 | 217 | 49 |
| 10. Plus reviews of the closed attack tests and new ties, now | 754 | **520** | 22 | **0** | 212 | 37 |
| 10, without any review (no store) | 754 | 313 | 149 | 0 | 292 | 145 |

**Read the last two lines together.** 207 of the 520 explained rest on reviews written by the agent that wrote the code and the tests. No human has reviewed anything.

**The 212 still unsupported** are of three kinds:
- what the fake plays: the quality of the models and of speech, the identity per device, the codes' randomness;
- what is not built: the disk, the journal, tuning, the wearable;
- ties of functions with no body in the model.

Under each, a note in `evidence.mitra` says why it is left.

### Tests and attacks
- Mine: 60 (32 unit, 28 scenarios). All passed on their first run, except one scenario whose expectation was wrong (3.1).
- Challenger, round 1 on code: 24 attacks. 13 were stopped by the first layer; 11 crossed (K1–K11). All 11 were closed.
- Challenger, round 2, against the fixes: 3 more crossed (K12–K14). K12: the K1 taint lasted one message, because the stored reply carried the page's words into the next history. K13: a stranger's words entered the owner's history. K14: the read limit was per read, not per call, and connections had no cap. All 3 are closed (a5001e2). I changed three of its tests, each marked in the file; see 3.2.
- Challenger, before the code, on the model: 21 distinct holes (H1–H21, W1–W14). All were folded into the model before or while coding, except H13 (a tool that ran before a crash leaves no note), which is still open.

### Holes by source (a hole = something wrong in the system or its model, not a typo)
| Source | Holes | Flagged by Mitra itself |
|---|---|---|
| Challenger reading the model (before code) | 21 | 0 |
| Challenger attacking the code (round 1) | 11 | 0 |
| Challenger attacking the fixes (round 2) | 3 | 0 |
| Me, running the two binaries by hand | 1 (the CLI skipped unseen replies after a restart) | 0 |
| Me, reading the links that Mitra's review asks pointed at | 3 (M17–M19) | 0, but the ask led there |
| **Total** | **39** | **0 found by Mitra; 3 found because of its asks** |

**Ratio.**
- Holes Mitra flagged by itself: 0 of 39.
- Holes it led to through a review ask: 3 of 39 (8%).
- The observer thread (MITRA-JUDGEMENT.md) counts the model-reading holes as 35 items (H1–H21, W1–W14). I count 21, because several W items repeat an H. Its total would be 53; the ratio is 0 either way.
- For comparison, on planted mutations of the routing (the red team before the night), Mitra caught 24 of 26.

### The challenger's own count (its words, ATTACK.md and log-challenger.md)
- Model, read before code: 35 items (H1–H21, W1–W14). 26 closed; 3 open (H13; W11, `sends_once` rests on "may retry" and the wearable has no send; W12, read/delivered receipts are never sent); 6 kept as assumptions or decisions (H17, H18, H10/H11/H19, H21).
- Code: round 1, 24 attacks, 13 stopped, 11 crossed, 11 closed. Round 2, 3 tried, 3 crossed, 3 closed. Round 3, re-read of a5001e2, 0 crossed.
- In total, 27 attack tests, all green; 14 crossed and 14 closed.
- How found: all 49 by reading and scripted tests. Mitra pointed at 0 before an attack.
- By its reading, Mitra could have pointed at W3/W4 (a row with two sends, a long history), W7 (a part requiring a fallible machine whose result has no failure), and the short `from` links (W11, K1, K12).
- After an attack was tied and run, Mitra flagged 10 of 10 failing tests (X015). K7, which has no ensure, showed as R004: the gap in the model was visible.
- It accepts my k12/k13 rewrite. A model asking for an act unprompted is now a named assumption: "a model asks for an act only from what it reads".

## 3. How Mitra helped, or not

### 3.1 Where it helped (with evidence)
- **A wrong test was loud at once.** My scenario `each_step_notes_what_it_decided` expected the wrong order of notes. Through its `supports` tie, `explain` showed X015 and 11 contradicted lines (Harness::noted, Worker::traced, Stored::append_only…). I fixed the test; the contradictions went back to 0.
- **Missing statements.** For bodiless functions, X001 and X007 showed measured values that the model said nothing about, and offered them as examples. I copied them into the model for `Commands::alarm`, `Commands::answer`, `Confirmations::code` and `ToolGuard::screen`. The help words and the "yes <code>" format are now model statements; before, they were words only.
- **Test helpers in place of the system.** R007 refused rows through helpers that are not tied (`fits`, `holds_all`, `starts_a_line`). The rows now go through `Composer::compose` and `Conversation::encode`, so they are measurements of the system.
- **The link asks (X012) made me read each link.** Three were wrong (M17–M19):
  - `Assistant::safe` did not rest on the owner's yes, a privileged path added tonight;
  - `Cli::sends_once` missed the count moving on;
  - `Confirmations::guessing_fails` gave the wrong reason.
- **Decided routing.** After a fix, `check` proves the new routing claims on every case: `data_never_acts` and `strangers_run_nothing` over the 24 inputs of `Policy::decide`, and the `after_gate` claims. A fix now cannot quietly regress them.
- **Model errors during edits.** P004, M022, M008 and P013 pointed at the right line every time.
- **The attack tests show in explain.** Once the challenger tied its open attacks with `supports`, each hole was a contradicted ensure in `explain`. They went back to 0 when closed.

### 3.2 Where it did not help, or got in the way
- **It found none of the 36 security holes.** None of them were flagged. The reasons fall into four kinds:
  - Each was a missing input or a missing promise. For K1, `Policy::decide` didn't know the call came after a tool output; for K3/K4, the history was by device and a read had a model-chosen input. Mitra proves claims of a function that cannot see the risk.
  - Or it was in a function with no body in the model (K2, K6, K7: guard, composer, reporter).
  - Or it was in an opaque type (K8, audio size).
  - Or it was in machine integers (K9) or TCP glue (K10, K11, K14), which the model doesn't have.
  - K12 and K13 were absences again: the history had no input saying where a turn came from. Both fixes are model changes (`history` now reads `vouched`, `read_output` and `answer_is`), and `check` decides nothing about them, because they are folds over the store.
- **A scripted fake can't tell a hole from a model acting on nothing.** The challenger's k12/k13 tests checked that the lights didn't run. The scripted cloud model asks for them in turn 2 whatever it reads. So I changed those tests to check that the page's or stranger's words are absent from the next request. That is a weaker test of the outcome, and a sharper test of the hole. The challenger was told and can push back.
- **The reader refuses bare enum variants (R001, new).** `derive(Observe)` writes `Confirm` and `Unreachable` bare. The reader refuses a variant name that two enums share (Step/Decision, NetFailure/LlmFailure, Step/Receipt/Note), even inside a field whose type says which, and `Assistant` (both a variant and a part). Workaround: Step::Confirm renamed to Step::Confirmations; Observe hand-written for 7 enums.
- **Rows the reader can't type, with no reason given (R006, new).** `Worker::address`, a tuple holding a `Party`, and `Worker::words` with a `Voice(Audio{..})` input are "not of" the function, and the message doesn't say which part fails. Not worked around: 2 rows lost.
- **Scenarios through untied glue (R014 ×419, new).** A test that drives the whole in-process server prints givens with no row after them. Workaround: the fake world is quiet in scenarios. Scenarios count through `supports` and reviews, and stay listed as R004.
- **No `\r` or `\u{…}` in Mitra texts (new).** A guard row with U+2028 can't be written; it is a plain assert.
- **A proved claim reads as unsupported.** `after_gate::benign_only`, decided on every case, shows "unsupported, only the model says" at the system level until the function's tie is reviewed. Rows on the tie do not lift the tie itself. That made me write 74 tie reviews.
- **Blame spreads to proved premises.** The challenger's numbers: with the 10 open attack tests, 157 contradicted, including claims proved on every case (`Policy::decide::stranger_reads_only`). Meanwhile K1 itself was a short `from` link.
- **Cost.**
  - Working around the reader: about 25 minutes of 100 (R001, R006, R014, R007).
  - Writing reviews: about 15 minutes.
  - Keeping the model in step: 24 model changes, each small.
  - `explain` itself is fast: about 5 s including `cargo test`.

### 3.3 Did the model change the code?
- **Yes:** the code was written function by function from the model's bodies. 74 functions were transcribed, and the routing is the model's finite functions. A function without a body in the model was free in code, and that is where 6 of the 11 attacks landed (guard, composer, reporter, speech size, TCP).
- **And the reverse:** the code changed the model 24 times (log M1–M24), each time because of a hole found by reasoning, by an attack, or by running the binaries.

## 4. Findings on Mitra, with workarounds
| # | Finding | Workaround |
|---|---|---|
| Y1 | Bare enum variants refused when two enums share a name, or a variant shares a part's name, even where the field type says which (R001) | Observe hand-written as `Enum::Variant`; one variant renamed |
| Y2 | R006 "not of" with no reason, for a tuple holding an enum and a partial-record input | none: 2 rows lost |
| Y3 | A test through code the model doesn't have can't be a measurement; its givens become R014 by the hundred | fake world quiet; `supports` + reviews |
| Y4 | No `\r`, `\u{…}` escapes in texts | plain assert |
| Y5 | A claim decided on every case stays unsupported until the function's tie is reviewed | 74 tie reviews by the author |
| Y6 | Blame spreads contradiction to premises Mitra itself proved (challenger) | none |
| Y7 | Nothing in Mitra asks "who can reach this", "what input does this decision lack", or "what follows a tool output"; holes of absence pass `check` and `explain` | an adversary thread |
| Y8 | A function with no body in the model is free in code; most code holes were there | write bodies for guard/composer-like functions, or claims on them |
| Y9 | Facts about where a stored turn came from (which sender, after which output) aren't typed; a fold over the store can't be decided, so `check` says nothing about the K12/K13 fixes | scenarios and attack tests, tied with `supports` |
| Earlier, met again | L, P (symbolic equality, case split), R (untypable values in tuples), S (`given` only on bodiless), 29 (enums can't `use`) | as before |

## 5. What remains open
- **H13:** a tool that ran just before a crash leaves no note. On restart the message is answered "interrupted" and never run twice. But the owner isn't told that the door may already be open. This needs a write-ahead note before the tool runs; it is not built.
- **Codes come from a linear generator** seeded from the clock (K5 made them differ between starts). They are guessable by anyone who knows the seed. The fake plays them; `Codes::unguessable` stays unreviewed.
- **The owner's hold shows up to 32 characters of the sender's words.** A short input can still word a misleading request. Longer or non-plain input is hidden.
- **W11:** `Cli::sends_once` rests on the client being allowed to retry; the wearable has no send at all. **W12:** read and delivered receipts are modelled but never sent.
- **What the fake plays isn't tested for real:** model quality, speech, identity per device. The disk, the journal, tuning and the wearable aren't built.
- **Top promises unreviewed on purpose:** `Assistant::safe` and `Assistant::private`. Their attack tests pass, but the agent that wrote the fixes doesn't vouch for the whole promise.
- **The challenger's next targets:** H13; two clients of one household at once on the TCP server; a long conversation near the size limits; pairing or revoking mid-work.
