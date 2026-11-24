# log: semver against semver.mitra

Tests live in `src/lib.rs` (`mod tests`) and `tests/cli.rs`, so every
explain/blame call passes `--code src --code tests`. From round 2 on,
`--store evidence.mitra` is passed too. Each round's full output is
below, verbatim (also kept in `rounds/rN.txt`).

## Before round 1: what I wrote

- `Cargo.toml` (lib + bin `semver`, `[workspace]` at the end).
- `src/lib.rs`: `Level`, `Failure`, `Version { major, minor, patch: u64, pre: Option<String> }`,
  `Version::bump/parse/render`, `level`, `run`, each tied with `// mitra: implements …`;
  one unit test per example row of the model, each with `// mitra: measures …`.
- `src/main.rs`: prints `run`'s text and exits 0, or prints a message on
  stderr only and exits 2 (Usage), 3 (UnknownLevel) or 4 (BadVersion).
- `tests/cli.rs`: runs the binary end to end (stdout empty and exit != 0 on failure).
- Gap I had to decide: the model's integers are unbounded, Rust's `u64` is
  not. `bump` returns `Option<Version>` (None at `u64::MAX`), and `run`
  maps that to `Failure::BadVersion`; parse rejects numbers past `u64`.
  Two tests for this measure nothing in the model (R004).

## Round 1

```text
$ mitra check semver.mitra
finding[A007]: claim `usage`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::run

finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

finding[A007]: claim `one_step`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

check: well formed; agree: 58 comparisons, 55 hold, 3 findings
$ cargo test > report.txt 2>&1
$ mitra explain semver.mitra --code src --code tests --report report.txt
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 0 reviews, 5 ties
unsupported  Cli::answers 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 2 only the model says)
unsupported  {Cli::run::usage} Cli::answers {Cli::answers} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::run::usage 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Semver::args 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Semver::bumps 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 10 only the model says)
unsupported  {Cli::answers, Version::bumps, Version::reads} Semver::bumps {Semver::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::bump::drops_pre 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::bump::grows 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::bump::one_step 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::bumps 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 4 only the model says)
unsupported  {Version::bump::grows, Version::bump::one_step, Version::bump::drops_pre} Version::bumps {Version::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::parse::empty 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::reads 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 3 only the model says)
unsupported  {Version::render::round_trip, Version::parse::empty} Version::reads {Version::reads} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::render::round_trip 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::bump at src/lib.rs/Version/bump 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::parse at src/lib.rs/Version/parse 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Version::render at src/lib.rs/Version/render 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::level at src/lib.rs/level 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
unsupported  Cli::run at src/lib.rs/run 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_minor)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_minor)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_minor)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_minor)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_major_drops_pre)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_major_drops_pre)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_major_drops_pre)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_major_drops_pre)
explained    correct(Version::bump, (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_patch)
explained    Version::bump::drops_pre at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_patch)
explained    Version::bump::grows at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_patch)
explained    Version::bump::one_step at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::bump_patch)
explained    correct(Version::parse, ("1.4.2")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_plain)
explained    Version::parse::empty at ("1.4.2") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_plain)
explained    correct(Version::parse, ("1.4.10")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_two_digits)
explained    Version::parse::empty at ("1.4.10") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_two_digits)
explained    correct(Version::parse, ("0.0.0")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_zeros)
explained    Version::parse::empty at ("0.0.0") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_zeros)
explained    correct(Version::parse, ("1.4.2-rc.1")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_pre)
explained    Version::parse::empty at ("1.4.2-rc.1") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_pre)
explained    correct(Version::parse, ("")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_empty)
explained    Version::parse::empty at ("") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_empty)
explained    correct(Version::parse, ("1.4")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_two_parts)
explained    Version::parse::empty at ("1.4") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_two_parts)
explained    correct(Version::parse, ("1.4.2.0")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_four_parts)
explained    Version::parse::empty at ("1.4.2.0") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_four_parts)
explained    correct(Version::parse, ("01.4.2")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_leading_zero)
explained    Version::parse::empty at ("01.4.2") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_leading_zero)
explained    correct(Version::parse, ("1.x.2")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_letter)
explained    Version::parse::empty at ("1.x.2") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_letter)
explained    correct(Version::parse, ("-1.4.2")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_sign)
explained    Version::parse::empty at ("-1.4.2") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_sign)
explained    correct(Version::parse, ("1.4.2-")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_empty_pre)
explained    Version::parse::empty at ("1.4.2-") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_empty_pre)
explained    correct(Version::parse, ("1.4.2+build")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_build)
explained    Version::parse::empty at ("1.4.2+build") 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::parse_build)
explained    correct(Version::render, (Version { major: 1, minor: 5, patch: 0, pre: None })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_plain)
explained    Version::render::round_trip at (Version { major: 1, minor: 5, patch: 0, pre: None }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_plain)
explained    correct(Version::render, (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") })) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_pre)
explained    Version::render::round_trip at (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_pre)
explained    correct(Cli::level, ("minor")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::level_minor)
explained    correct(Cli::level, ("Minor")) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::level_case_matters)
explained    correct(Cli::run, (["bump", "minor", "1.4.2"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    Cli::run::usage at (["bump", "minor", "1.4.2"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    correct(Cli::run, (["bump", "major", "1.4.2-rc.1"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_major_pre)
explained    Cli::run::usage at (["bump", "major", "1.4.2-rc.1"]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_major_pre)
explained    correct(Cli::run, (["bump", "patch", "1.4.9"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_patch)
explained    Cli::run::usage at (["bump", "patch", "1.4.9"]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_patch)
explained    correct(Cli::run, (["bump", "minor"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    Cli::run::usage at (["bump", "minor"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    correct(Cli::run, (["grow", "minor", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_not_bump)
explained    Cli::run::usage at (["grow", "minor", "1.4.2"]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_not_bump)
explained    correct(Cli::run, (["bump", "huge", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_unknown_level)
explained    Cli::run::usage at (["bump", "huge", "1.4.2"]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_unknown_level)
explained    correct(Cli::run, (["bump", "huge", "1.4"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_level_before_version)
explained    Cli::run::usage at (["bump", "huge", "1.4"]) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_level_before_version)
explained    correct(Cli::run, (["bump", "minor", "1.4"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)
explained    Cli::run::usage at (["bump", "minor", "1.4"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)

explain: 78 propositions, 58 explained, 0 contradicted, 20 unsupported; 0 findings, 0 hints
$ mitra blame semver.mitra --code src --code tests --report report.txt
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 0 reviews, 5 ties
blame: nothing conflicts, nothing to blame
```

What it says: every example row is explained by its test, but the
universal claims (`drops_pre`, `grows`, `one_step`, `empty`, `round_trip`,
`usage`), the ensures, the require `args`, and the five ties are held only
by the model. Tests at points do not hold an "for every input" claim, and
the model has no rules/proofs I may add.

Changed for round 2:
- I found a real hole while thinking about `round_trip`: a `Version` with
  `pre: Some("")` or `Some("a+b")` renders to a text `parse` refuses. I made
  the fields private and added `Version::new`, which refuses those, so every
  `Version` the code can build reads back. (`new_refuses_what_does_not_read_back`.)
- Added property tests (no `measures` row: a claim cannot be named in one):
  `bump_claims_hold` (241 versions x 3 levels, all three claims),
  `render_round_trips` (241 versions), `parse_refuses_near_misses`,
  `run_usage_claim_holds` (0 to 4 words).
- Wrote `evidence.mitra` with reviews by `agent:anthropic/claude-opus-5-5`
  of the six claims, the require `args` (main.rs skips the program name),
  and the five ties (each Rust function read against the model's body).

## Round 2

```text
$ mitra check semver.mitra
finding[A007]: claim `usage`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::run

finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

finding[A007]: claim `one_step`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

check: well formed; agree: 58 comparisons, 55 hold, 3 findings
$ cargo test > report.txt 2>&1
$ mitra explain semver.mitra --code src --code tests --report report.txt --store evidence.mitra
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::bump_claims_hold` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::new_refuses_what_does_not_read_back` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::parse_refuses_near_misses` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_round_trips` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::run_usage_claim_holds` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 12 reviews, 5 ties
unsupported  Cli::answers 0.56 (b 0.11, d 0.00, u 0.89: 1 for, 0 against, 1 only the model says, agent:anthropic/claude-opus-5-5)
unsupported  {Cli::run::usage} Cli::answers {Cli::answers} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Cli::run::usage 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Semver::args 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  Semver::bumps 0.50 (b 0.00, d 0.00, u 1.00: 6 for, 0 against, 4 only the model says, agent:anthropic/claude-opus-5-5)
unsupported  {Cli::answers, Version::bumps, Version::reads} Semver::bumps {Semver::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::bump::drops_pre 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump::grows 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump::one_step 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  Version::bumps 0.55 (b 0.09, d 0.00, u 0.91: 3 for, 0 against, 1 only the model says, agent:anthropic/claude-opus-5-5)
unsupported  {Version::bump::grows, Version::bump::one_step, Version::bump::drops_pre} Version::bumps {Version::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::parse::empty 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  Version::reads 0.56 (b 0.11, d 0.00, u 0.89: 2 for, 0 against, 1 only the model says, agent:anthropic/claude-opus-5-5)
unsupported  {Version::render::round_trip, Version::parse::empty} Version::reads {Version::reads} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::render::round_trip 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump at src/lib.rs/Version/bump 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::parse at src/lib.rs/Version/parse 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::render at src/lib.rs/Version/render 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Cli::level at src/lib.rs/level 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Cli::run at src/lib.rs/run 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::bump, (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.10")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.10") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("0.0.0")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("0.0.0") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2-rc.1")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2-rc.1") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2.0")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2.0") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("01.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("01.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.x.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.x.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("-1.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("-1.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2-")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2-") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2+build")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2+build") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::render, (Version { major: 1, minor: 5, patch: 0, pre: None })) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::render::round_trip at (Version { major: 1, minor: 5, patch: 0, pre: None }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_plain)
explained    correct(Version::render, (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") })) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::render::round_trip at (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_pre)
explained    correct(Cli::level, ("minor")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    correct(Cli::level, ("Minor")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    correct(Cli::run, (["bump", "minor", "1.4.2"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    Cli::run::usage at (["bump", "minor", "1.4.2"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    correct(Cli::run, (["bump", "major", "1.4.2-rc.1"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_major_pre)
explained    Cli::run::usage at (["bump", "major", "1.4.2-rc.1"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "patch", "1.4.9"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_patch)
explained    Cli::run::usage at (["bump", "patch", "1.4.9"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "minor"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    Cli::run::usage at (["bump", "minor"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    correct(Cli::run, (["grow", "minor", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_not_bump)
explained    Cli::run::usage at (["grow", "minor", "1.4.2"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "huge", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_unknown_level)
explained    Cli::run::usage at (["bump", "huge", "1.4.2"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "huge", "1.4"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_level_before_version)
explained    Cli::run::usage at (["bump", "huge", "1.4"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "minor", "1.4"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)
explained    Cli::run::usage at (["bump", "minor", "1.4"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)

explain: 78 propositions, 70 explained, 0 contradicted, 8 unsupported; 0 findings, 0 hints
$ mitra blame semver.mitra --code src --code tests --report report.txt --store evidence.mitra
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::bump_claims_hold` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::new_refuses_what_does_not_read_back` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::parse_refuses_near_misses` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_round_trips` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::run_usage_claim_holds` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 12 reviews, 5 ties
blame: nothing conflicts, nothing to blame
```

What it says: the claims, ties and require are explained. Left: the four
ensures (each `N for` from its premises, but still resting on "1 only the
model says"), and four propositions written `{premises} E {E}`: the rule
each ensure's `from (…)` makes.

Changed for round 3: added reviews of the four ensures (`Version::bumps`,
`Version::reads`, `Cli::answers`, `Semver::bumps`), each after reading the
code and tests named in the comment above it in `evidence.mitra`.

## Round 3

```text
$ mitra check semver.mitra
finding[A007]: claim `usage`: unknown, not decided by bounds over all of what it reads, nor at 64 points
 --> Cli::run

finding[A007]: claim `grows`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

finding[A007]: claim `one_step`: unknown, not decided by bounds over all of what it reads, nor at 18 points
 --> Version::bump

check: well formed; agree: 58 comparisons, 55 hold, 3 findings
$ cargo test > report.txt 2>&1
$ mitra explain semver.mitra --code src --code tests --report report.txt --store evidence.mitra
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::bump_claims_hold` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::new_refuses_what_does_not_read_back` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::parse_refuses_near_misses` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_round_trips` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::run_usage_claim_holds` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 16 reviews, 5 ties
explained    Cli::answers 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  {Cli::run::usage} Cli::answers {Cli::answers} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Cli::run::usage 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Semver::args 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Semver::bumps 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  {Cli::answers, Version::bumps, Version::reads} Semver::bumps {Semver::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::bump::drops_pre 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump::grows 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump::one_step 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bumps 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  {Version::bump::grows, Version::bump::one_step, Version::bump::drops_pre} Version::bumps {Version::bumps} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::parse::empty 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::reads 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
unsupported  {Version::render::round_trip, Version::parse::empty} Version::reads {Version::reads} 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
explained    Version::render::round_trip 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::bump at src/lib.rs/Version/bump 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::parse at src/lib.rs/Version/parse 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Version::render at src/lib.rs/Version/render 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Cli::level at src/lib.rs/level 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    Cli::run at src/lib.rs/run 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, agent:anthropic/claude-opus-5-5)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: None }, Minor) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::bump, (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 1, minor: 4, patch: 2, pre: Some("rc.1") }, Major) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::bump, (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch)) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::bump::drops_pre at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::grows at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    Version::bump::one_step at (Version { major: 0, minor: 9, patch: 9, pre: None }, Patch) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.10")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.10") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("0.0.0")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("0.0.0") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2-rc.1")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2-rc.1") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2.0")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2.0") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("01.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("01.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.x.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.x.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("-1.4.2")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("-1.4.2") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2-")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2-") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::parse, ("1.4.2+build")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::parse::empty at ("1.4.2+build") 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Version::render, (Version { major: 1, minor: 5, patch: 0, pre: None })) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::render::round_trip at (Version { major: 1, minor: 5, patch: 0, pre: None }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_plain)
explained    correct(Version::render, (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") })) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    Version::render::round_trip at (Version { major: 1, minor: 2, patch: 3, pre: Some("rc.1") }) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::render_pre)
explained    correct(Cli::level, ("minor")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    correct(Cli::level, ("Minor")) 0.78 (b 0.55, d 0.00, u 0.45: 2 for, 0 against, machine:mitra-check, agent:anthropic/claude-opus-5-5)
explained    correct(Cli::run, (["bump", "minor", "1.4.2"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    Cli::run::usage at (["bump", "minor", "1.4.2"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_minor)
explained    correct(Cli::run, (["bump", "major", "1.4.2-rc.1"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_major_pre)
explained    Cli::run::usage at (["bump", "major", "1.4.2-rc.1"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "patch", "1.4.9"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_patch)
explained    Cli::run::usage at (["bump", "patch", "1.4.9"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "minor"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    Cli::run::usage at (["bump", "minor"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:usage_prints_nothing_and_fails)
explained    correct(Cli::run, (["grow", "minor", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_not_bump)
explained    Cli::run::usage at (["grow", "minor", "1.4.2"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "huge", "1.4.2"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_unknown_level)
explained    Cli::run::usage at (["bump", "huge", "1.4.2"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "huge", "1.4"])) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::run_level_before_version)
explained    Cli::run::usage at (["bump", "huge", "1.4"]) 0.68 (b 0.36, d 0.00, u 0.64: 3 for, 0 against, agent:anthropic/claude-opus-5-5, machine:mitra-check)
explained    correct(Cli::run, (["bump", "minor", "1.4"])) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)
explained    Cli::run::usage at (["bump", "minor", "1.4"]) 0.75 (b 0.50, d 0.00, u 0.50: 1 for, 0 against, test:tests::run_bad_version)

explain: 78 propositions, 74 explained, 0 contradicted, 4 unsupported; 0 findings, 0 hints
$ mitra blame semver.mitra --code src --code tests --report report.txt --store evidence.mitra
finding[R004]: the test `tests::bump_overflow_is_none` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::bump_claims_hold` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::new_refuses_what_does_not_read_back` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::parse_refuses_near_misses` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::render_round_trips` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `tests::run_usage_claim_holds` measures nothing the reader knows
 --> cargo test:0

finding[R004]: the test `overflow_prints_nothing_and_fails` measures nothing the reader knows
 --> cargo test:0

evidence: 30 measurements, 16 reviews, 5 ties
blame: nothing conflicts, nothing to blame
```

The four `{premises} E {E}` rules stay unsupported. I tried to review them
from the store (probes, not kept; outputs verbatim):

```text
== review Version::bumps::from holds by "agent:x";
error[P003]: expected a name, found `from`
== review Version::bumps::rule holds by "agent:x";
error[P003]: expected a name, found `rule`
== review Version::bumps::0 holds by "agent:x";
error[P003]: expected a name, found `0`
== review {Version::bump::grows, Version::bump::one_step, Version::bump::drops_pre} Version::bumps {Version::bumps} holds by "agent:x";
error[P003]: expected a name, found `{`
1 | review {Version::bump::grows, Version::bump::one_step, Version::bump::drops_pre} Version::bumps {Version::bumps} holds by "agent:x";
== review rule Version::bumps holds by "agent:x";
error[P003]: expected a name, found `rule`
== review Version::bumps::bumps holds by "agent:x";
finding[R006]: `Version::bumps::bumps` is no claim, rule, require, ensure or tied item of the model
== review "Version::bumps" holds by "agent:x";
error[P003]: expected a name, found a string
== review ensure Version::bumps holds by "agent:x";
error[P003]: expected a name, found `ensure`
```

`review Version::bumps` reviews the ensure, not its `from` rule, and I
found no path that names the rule. Only the model could hold it (a proof,
or a written rule), and I may not change the model. I stopped here: no
further code or evidence change can move these four.

## Where I think the spec is wrong or loose

- `Version::render::round_trip` claims `parse(render(v)) == Ok(v)` for every
  `Version`, but the model's `Version` allows `pre: Some("")`,
  `pre: Some("a+b")` and negative numbers, which render to texts its own
  `parse` must refuse. `mitra check` reports it as holding (parse has no
  body, so it can only be tried at examples). The code closes the hole with
  `Version::new`; the model should restrict `pre` (and the numbers to `>= 0`).
- `bump` cannot fail in the model, but a real tool has a largest number.
  The code fails with `BadVersion` (exit 4, nothing on stdout) when the
  bumped number does not fit `u64`; the model says nothing of this.
- The model does not say what characters a pre-release may hold (SemVer
  allows `[0-9A-Za-z-.]` with no empty identifiers). The code accepts any
  non-empty text without `+`, which is what the model says.
- The model gives `Failure` but no exit codes; the code picks 2/3/4.

## Summary

- Rounds: 3.
- Explain summary line per round:
  - round 1: `explain: 78 propositions, 58 explained, 0 contradicted, 20 unsupported; 0 findings, 0 hints`
  - round 2: `explain: 78 propositions, 70 explained, 0 contradicted, 8 unsupported; 0 findings, 0 hints`
  - round 3: `explain: 78 propositions, 74 explained, 0 contradicted, 4 unsupported; 0 findings, 0 hints`
- The 4 left unsupported are the ensures' `from` rules, which no review in
  the store can name.
- Lines of code: 156 (src/lib.rs before `mod tests`: 141; src/main.rs: 15).
- Lines of tests: 292 (src/lib.rs `mod tests`: 256; tests/cli.rs: 36);
  37 tests, all passing.
