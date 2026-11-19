# Keywords: requirements in prose

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## ensure

`ensure name = "text";` in a subject: what it promises, in plain words.
Mitra never reads the text. The requirement is a claim, `signup::confirms`,
that rests on:

- every claim of its subject (also `f::correct` and `f::spec`);
- `signup::confirms::captured`: someone's word that these claims say
  what the text says. Nobody can compute it; a reviewer vouches for it;
- what the subject [requires](#require).

It holds as far as they do, and fails when one of its claims fails.

```mitra
subject signup {
    ensure confirms = "A new user receives a confirmation email.";
    fn register(email: {"a@x", "b@x"}) -> {"created", "refused"};
    fn sent(email: {"a@x", "b@x"}) -> Bool;
    claim confirmation_sent(email) when register(email) == "created" { sent(email) }
}
```
```evidence
evidence a { for register given email = "a@x" gives "created" }
evidence b { for register given email = "b@x" gives "refused" }
evidence c { for sent given email = "a@x" gives true }
evidence r { for register holds source "review:me" }
evidence s { for sent holds source "review:me" }
```
```text
signup::register::correct: holds
signup::sent::correct: holds
signup::confirmation_sent: holds
signup::confirms::captured: unverified
  nobody vouched that the claims say what the text says
signup::confirms: holds assuming [signup::confirms::captured]
  says            "A new user receives a confirmation email."
  rests on        `signup::register::correct`, `signup::sent::correct`, `signup::confirmation_sent`
```

Once a reviewer agrees that the claims say what the text says:

```mitra
subject signup {
    ensure confirms = "A new user receives a confirmation email.";
    fn register(email: {"a@x", "b@x"}) -> {"created", "refused"};
    fn sent(email: {"a@x", "b@x"}) -> Bool;
    claim confirmation_sent(email) when register(email) == "created" { sent(email) }
}
```
```evidence
evidence a { for register given email = "a@x" gives "created" }
evidence b { for register given email = "b@x" gives "refused" }
evidence c { for sent given email = "a@x" gives true }
evidence r { for register holds source "review:me" }
evidence s { for sent holds source "review:me" }
evidence v { for signup::confirms::captured holds source "review:me" trust 0.8 }
```
```text
signup::register::correct: holds
signup::sent::correct: holds
signup::confirmation_sent: holds
signup::confirms::captured: holds
signup::confirms: holds
  says            "A new user receives a confirmation email."
  rests on        `signup::register::correct`, `signup::sent::correct`, `signup::confirmation_sent`
```

Evidence may be brought on the requirement itself, by someone who knows
nothing of the model: a support ticket. With the same model and the
evidence above, one more line:

```mitra
subject signup {
    ensure confirms = "A new user receives a confirmation email.";
    fn register(email: {"a@x", "b@x"}) -> {"created", "refused"};
    fn sent(email: {"a@x", "b@x"}) -> Bool;
    claim confirmation_sent(email) when register(email) == "created" { sent(email) }
}
```
```evidence
evidence a { for register given email = "a@x" gives "created" }
evidence b { for register given email = "b@x" gives "refused" }
evidence c { for sent given email = "a@x" gives true }
evidence r { for register holds source "review:me" }
evidence s { for sent holds source "review:me" }
evidence v { for signup::confirms::captured holds source "review:me" trust 0.8 }
evidence bug_812 { for signup::confirms fails trust 0.9 source "support:ticket-812" }
```
```text
signup::register::correct: holds
signup::sent::correct: holds
signup::confirmation_sent: holds
signup::confirms::captured: holds
signup::confirms: fails
  says            "A new user receives a confirmation email."
  fails: evidence says it is not met
  rests on        `signup::register::correct`, `signup::sent::correct`, `signup::confirmation_sent`
```

The claims still hold, so something else is wrong. The error lists the
candidates: the text, the word that the claims say it, the ticket.

```text
    likely    the text of `signup::confirms`, model.mitra:2                    none   0.00 to 0.28
    unlikely  that the claims say what `signup::confirms` says, model.mitra:2  0.8    0.80 to 1.00
    unlikely  evidence.mitra:7 (support:ticket-812)                            0.9    0.90 to 1.00
```

Evidence that the requirement holds (`for signup::confirms holds`) says
nothing: it holds only as far as its claims and its capture.

A subject with an `ensure` and no claim: the requirement is
`unverified`, and `mitra plan` owes a claim that says it.

## require

`require name = other::promise;` in a subject or a boundary: it needs a
promise made elsewhere. The need has a name of its own,
`signup::delivery`, for reports and evidence. It holds as far as the
promise it names, and every `ensure` of the block rests on it. In an
error on such an `ensure`, "that `mailer::delivers` is what
`signup::delivery` needs" is a candidate, with the promise itself.

`require name = "text";` is a need nobody promises yet: assumed.

In a boundary, `ensure` is what the world is promised to do. It rests
on the `assume` lines of its block, as a subject's on its claims, with
its own `mailer::delivers::captured`: someone's word that the
assumptions read the promise fairly. Evidence against it has as
candidates the text (the world broke its promise), the capture (the
model misread it), each assumption and the evidence. With no `assume`
in the block it is taken on its word: `holds assuming [itself]`.

`require` is to intent what [uses](kw-declarations.md#uses) is to code,
and the two are compared. M105: a `require` on a promise of a block
that no function beside it `uses`. M106: a function that `uses` a block
which makes promises, with no `require` beside it naming one; only in a
block that states an `ensure` or a `require`.

`trust n` before the `;` of an `ensure` or a `require` is the trust
given to the text, or to the link; none when not said.

```mitra
boundary mailer {
    ensure delivers = "The provider delivers a message within one minute, or reports a failure.";
    fn send(to: {"a@x", "b@x"}) -> {"sent", "failed"};
    assume mostly_sent() trust 0.9 { rate(t in {"a@x", "b@x"}, send(t) == "sent") >= 0.5 }
}
subject signup {
    ensure confirms = "A new user receives a confirmation email.";
    require delivery = mailer::delivers trust 0.8;
    fn register(email: {"a@x", "b@x"}) -> {"created", "refused"} uses mailer::send;
    claim confirmation_sent(email) when register(email) == "created" { mailer::send(email) == "sent" }
}
```
```text
mailer::mostly_sent: holds assuming [itself]
signup::register::uses: holds
signup::register::correct: unverified
  no evidence on  `signup::register("a@x")`, `signup::register("b@x")`
signup::confirmation_sent: unverified
  no evidence on  `email = "a@x"`, `email = "b@x"`
mailer::delivers::captured: unverified
  nobody vouched that the claims say what the text says
mailer::delivers: holds assuming [mailer::mostly_sent, mailer::delivers::captured]
  says            "The provider delivers a message within one minute, or reports a failure."
  rests on        `mailer::mostly_sent`
signup::delivery: holds assuming [mailer::mostly_sent, mailer::delivers::captured]
  needs           `mailer::delivers`
signup::confirms::captured: unverified
  nobody vouched that the claims say what the text says
signup::confirms: holds assuming [signup::register::correct, signup::confirmation_sent, mailer::mostly_sent, mailer::delivers::captured, signup::confirms::captured]
  says            "A new user receives a confirmation email."
  rests on        `signup::register::correct`, `signup::confirmation_sent`
  requires        `signup::delivery`
```

`require` of a name no `ensure` has is refused, and so are
`ensure name "text";` and `require other::promise;`, the forms without `=`.
