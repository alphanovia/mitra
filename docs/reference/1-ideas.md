# 1. The ideas

## Model, system, evidence

- The **system** is what you build: code, and the world it runs in.
- The **model** is what you say the system does, written in Mitra. It
  is code too: it has functions, values and types, and Mitra can run
  it.
- The **evidence** is what was seen of the system: tests that passed or
  failed, measurements, reviews by people or agents, proofs.

Mitra never assumes the model is right, nor the code, nor the tests. It
weighs what each says against the others.

## The three questions

| Command | Question | Reads |
|---|---|---|
| `mitra check` | Is the model coherent on its own? | the model |
| `mitra explain` | Does the model explain what the evidence saw? | model, code, the tests it runs, store |
| `mitra blame` | If not, which piece is most likely wrong? | the same |

`check` needs no code at all: you can settle a spec before writing a
line of it.

## Claims, and what holds them

The model is made of statements: a function's **claims** (`q * b <= a`),
a part's **requires** (what it needs) and **ensures** (what it
promises), **rules** (`p, q |- c`: if p and q then c), and **ties**
(this Rust function is that model function).

Each statement is held by something:

| Held by | Example | Strength |
|---|---|---|
| only the model | a claim nobody tested | none: it is a hypothesis |
| a test run | `div(6, 2) == 3` passed | one run's worth |
| many runs | 100 runs at that point | more |
| a review | "I read `spec`, it holds" | one voice's worth |
| a proof the checker took | `by ponens(…)` | as strong as Mitra knows |

A proof is the strongest evidence, but it is never required. Most
claims live on tests and reviews.

## Verdicts

`mitra explain` gives each statement one verdict:

- **explained**: evidence supports it, without resting on anything only
  the model says;
- **assumed**: evidence supports it, but only together with
  assumptions: a link no one reviewed yet (an ensure's `from` list,
  which a review lifts), or a require said in words (what the world
  must give, which no review of the model lifts). The line says which;
- **contradicted**: evidence goes against it;
- **unsupported**: only the model says it.

A require that names another part's ensure (`require checked:
check::sound;`) is that ensure by another name: Mitra checks the link
itself, so it is never an assumption.

## Trust numbers

Each verdict comes with numbers:

```text
explained correct(calc::div, (6, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::div_by_two)
```

- `b` (belief), `d` (disbelief) and `u` (uncertainty) add up to 1. They
  come from counting runs for and against: with `r` for, `s` against,
  `b = r / (r + s + 2)`, `d = s / (r + s + 2)`, `u = 2 / (r + s + 2)`.
- `0.67` is the **expected** value, `b + 0.5 × u`: with no evidence it
  is 0.5, and it moves toward 1 or 0 as evidence piles up.
- Then the counts, and the **voices** that gave them: `test:…` for a
  test, `human:…`, `agent:…`, `machine:…` for the others.

One passing test gives 0.67; ten give 0.92. A proof weighs as a
thousand runs. The base rate (0.5), the prior weight (2), the weight of
a proof and of each voice are settings of `mitra.toml`
([chapter 8](8-commands.md#mitratoml)). This is Subjective Logic (Jøsang, 2016) on counts.

## Findings are not errors

`mitra check` stops on an **error** (`error[M005]`): the model is not
well formed. A **finding** (`finding[A007]`) is something it saw and
reports, never silently drops: a claim it could not decide, a test it
could not use. An **ask** (`ask[X012]`) is a statement only a person
can settle, such as a link between an ensure and what it rests on:
`explain` lists each one and the review line to write. The exit code
stays 0. Every code has a page in
[codes/](codes/README.md).
