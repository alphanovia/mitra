//! `mitra next`: what can't all be true, the claims that fail, the claims
//! that hold only under an assumption, and for each unverified claim what
//! is missed and the evidence that would settle it.

use crate::ast::{Model, Target};
use crate::eval::{Assumed, Logic, Report};
use crate::print::call;
use crate::Value;

/// Up to this many missing inputs are listed one by one.
const LIST: usize = 5;

/// A missing set longer than this is summed up, not printed.
const WIDE: usize = 100;

pub fn next(model: &Model, report: &Report) -> String {
    let mut out = String::new();
    for k in &report.contradictions {
        // A requirement in prose is named, not the function that holds it.
        let on = match k.subject.strip_suffix("::prose") {
            Some(name) => format!("`{name}`"),
            None => format!("{} of `{}`", k.inputs, k.subject),
        };
        out += &format!(
            "evidence lines {} and {} can't both be true, on {on}: settle them\n",
            k.pass.0, k.fail.0
        );
    }
    for r in &report.roots {
        out += &format!("root cause: {}\n", r.text());
    }
    // One verdict per claim; among failing claims, first those on a
    // subject that uses nothing failing: the likely root causes.
    let mut verdicts: Vec<&crate::eval::Verdict> = Vec::new();
    for v in &report.verdicts {
        if !verdicts.iter().any(|w| w.claim == v.claim) {
            verdicts.push(v);
        }
    }
    let subject = |v: &crate::eval::Verdict| &model.claim(&v.claim).expect("checked").func;
    let suspects = |f: &String| -> Vec<&crate::eval::Suspect> {
        report.suspects.iter().filter(|s| &s.subject == f).collect()
    };
    let look = |f: &String| -> String {
        suspects(f)
            .iter()
            .map(|s| {
                format!(
                    "  look first at `{}`, which `{f}` uses: {} fail{} there\n",
                    s.uses,
                    s.failing.join(", "),
                    if s.failing.len() == 1 { "s" } else { "" }
                )
            })
            .collect()
    };
    // What can't all be true comes first.
    let mut conflicts: Vec<&crate::eval::Conflict> = report.conflicts.iter().collect();
    let of = |k: &crate::eval::Conflict| &model.claim(&k.claim).expect("checked").func;
    conflicts.sort_by_key(|k| !suspects(of(k)).is_empty());
    for k in conflicts {
        let subject = of(k);
        let about = k.on.as_ref().unwrap_or(subject);
        let (pass, fail) = (call(about, &k.pass.0), call(about, &k.fail.0));
        out += &match &k.spec {
            _ if !k.calls.is_empty() => format!(
                "{}: fails: `{pass}` passes and `{fail}` fails, both with {}\n  \
                 find what differs between the two inputs and split a class,\n  \
                 or check the two results: one of them may be wrong\n",
                k.claim,
                crate::eval::class_text(std::slice::from_ref(&k.class), &k.spec, &k.calls),
            ),
            Some(spec) => format!(
                "{}: fails: `{pass}` passes and `{fail}` fails, both in class {} of `{spec}`\n  \
                 find what differs between the two inputs and split the class in `{spec}`,\n  \
                 or check the two results: one of them may be wrong\n",
                k.claim, k.class,
            ),
            None => format!(
                "{}: fails: `{pass}` passes and `{fail}` fails\n  \
                 find what differs between the two inputs and sort the inputs into classes,\n  \
                 `claim c = uniform({subject}, <classes>);`, or check the two results: one of them may be wrong\n",
                k.claim,
            ),
        };
        out += &look(subject);
    }
    verdicts.sort_by_key(|v| v.logic != Logic::Fails || !suspects(subject(v)).is_empty());
    for v in verdicts {
        let c = model.claim(&v.claim).expect("verdicts name claims");
        // A requirement in prose: what settles it is not a test.
        if model.captured(&c.name).is_some() {
            if v.logic == Logic::Unverified {
                out += &format!(
                    "{}: nobody vouched that the claims say what the text says\n  \
                     review them against the text, then: evidence <name> {{ for {} holds source \"review:<who>\" }}\n",
                    c.name, c.name
                );
            }
            continue;
        }
        if let Some(e) = model.ensure(&c.name) {
            if e.needs.is_some() && v.logic == Logic::Assuming {
                continue;
            }
            out += &match v.logic {
                Logic::Fails if v.refuted.is_empty() => {
                    format!("{}: fails with a claim it rests on\n", c.name)
                }
                Logic::Fails => format!(
                    "{}: fails: evidence says it is not met\n  \
                     write the claim that would have caught it, or correct the evidence\n",
                    c.name
                ),
                Logic::Unverified if e.claims.is_empty() && !e.boundary => format!(
                    "{}: no claim in its subject says it\n  write the claims that say it\n",
                    c.name
                ),
                Logic::Assuming if !v.own_word() => format!(
                    "{}: holds assuming [{}]\n  settle what it rests on\n",
                    c.name,
                    v.assuming.join(", ")
                ),
                _ => String::new(),
            };
            continue;
        }
        let conflict = report.conflicts.iter().any(|k| k.claim == c.name);
        if v.logic == Logic::Fails && c.target.is_structural() && !conflict {
            out += &format!("{}: fails: evidence says so\n", c.name);
        }
        if v.logic == Logic::Fails && !c.target.is_structural() {
            out += &format!(
                "{}: fails on {}\n",
                c.name,
                crate::eval::brief(&v.refuted, &v.domain)
            );
            out += &look(&c.func);
        }
        if v.logic == Logic::Holds && !c.target.is_structural() && !suspects(&c.func).is_empty() {
            out += &format!("{}: holds, but is suspect\n", c.name);
            out += &look(&c.func);
        }
        if v.own_word() {
            out += &format!(
                "{}: an assumption, with nothing but its own word\n  observe `{}` to check it, or leave it: the claims that read it say so\n",
                c.name,
                model.subjects_of(c).join("`, `")
            );
        } else if v.logic == Logic::Assuming {
            out += &format!(
                "{}: holds assuming [{}]\n  bring evidence on what it assumes, such as a review, or test until it is not needed\n",
                c.name,
                v.assuming.join(", ")
            );
        }
        if v.logic != Logic::Unverified {
            continue;
        }
        if let (Target::Represented(_), true) = (&c.target, v.tested.is_empty()) {
            out += &format!(
                "{}: not vouched for\n  review that one input of its set stands for the others, for `{}`, then:\n{}",
                c.name,
                c.func,
                vouch(model, &c.name)
            );
            continue;
        }
        if let Target::Uniform(spec) = &c.target {
            let within = match spec {
                Some(spec) => format!("within each class of `{spec}`"),
                None => "for every input".to_string(),
            };
            out += &if v.tested.is_empty() {
                format!(
                    "{}: not vouched for\n  review that `{}` behaves the same {within}, then:\n{}",
                    c.name,
                    c.func,
                    vouch(model, &c.name)
                )
            } else {
                format!(
                    "{}: vouched for in part, or both vouched for and denied: settle the evidence on it\n",
                    c.name
                )
            };
            continue;
        }
        if let (Target::Uses { .. }, false) = (&c.target, v.tested.is_empty()) {
            out += &format!(
                "{}: vouched for in part, or both vouched for and denied: settle the evidence on it\n",
                c.name
            );
            continue;
        }
        if let Target::Uses { used, only } = &c.target {
            let used: Vec<String> = used.iter().map(|g| format!("`{g}`")).collect();
            let (more, until) = if *only {
                (
                    " and no other subject",
                    "until then it gives suspects, but no root cause",
                )
            } else {
                ("", "until then it still gives suspects")
            };
            out += &format!(
                "{}: not vouched for\n  review that `{}` calls {}{more}, then:\n{}  {until}\n",
                c.name,
                c.func,
                used.join(", "),
                vouch(model, &c.name)
            );
            continue;
        }
        let missing = v.missed.clone();
        let mut shown = missing.show(&v.domain);
        if shown.len() > WIDE {
            shown = match v.tested.union(&v.derived).size_bound() {
                Some(n) => format!("all of {} but {n} inputs", v.domain.show(&v.domain)),
                None => "a set too long to print".to_string(),
            };
        }
        match &v.assumed {
            Some(Assumed {
                usable: true,
                untested,
                spec: Some(spec),
                ..
            }) if !untested.is_empty() => {
                let n = untested.len();
                out += &format!("{}: missing {n} of the classes of `{spec}`\n", c.name)
            }
            Some(a) if a.usable && !a.untested.is_empty() && !a.calls.is_empty() => {
                let calls: Vec<String> = a.calls.iter().map(|c| format!("`{c}`")).collect();
                out += &format!(
                    "{}: missing {} of the classes of {}\n",
                    c.name,
                    a.untested.len(),
                    calls.join(", ")
                )
            }
            _ => out += &format!("{}: missing {shown}\n", c.name),
        }
        let premise = |logic: Logic| -> Vec<&str> {
            c.from
                .iter()
                .filter(|p| {
                    report
                        .verdicts
                        .iter()
                        .any(|w| &&w.claim == p && w.logic == logic)
                })
                .map(|p| p.as_str())
                .collect()
        };
        let (failing, waiting) = (premise(Logic::Fails), premise(Logic::Unverified));
        if !failing.is_empty() {
            out += &format!("  derived: premise fails: {}\n", failing.join(", "));
        }
        if !waiting.is_empty() {
            out += &format!("  derived: waits on {}\n", waiting.join(", "));
        }
        out += &look(&c.func);
        if let Some(why) = &v.unsorted {
            out += &format!("  {why}\n");
        }
        // With a uniform claim, one passing input per class is enough.
        if let Some(a) = &v.assumed {
            let by = report.verdicts.iter().find(|w| w.claim == a.by);
            match by.map(|w| w.logic) {
                Some(Logic::Fails) => {
                    out += &format!("  `{}` fails: its classes stand for nothing\n", a.by)
                }
                Some(Logic::Unverified) => {
                    out += &format!(
                        "  `{}` is not vouched for: what rests on it is only assumed\n",
                        a.by
                    )
                }
                _ => {}
            }
            if a.blind {
                out +=
                    "  a failure over a set too large to sort into classes: nothing is assumed\n";
                continue;
            }
            if a.untested.is_empty() {
                out += "  some inputs have both a pass and a failure: settle them\n";
            }
            for class in &a.untested {
                let tainted = a.tainted.contains(class);
                let text = crate::eval::class_text(std::slice::from_ref(class), &a.spec, &a.calls);
                out += &match (&a.spec, tainted) {
                    _ if !a.calls.is_empty() && tainted => format!(
                        "  {text} have an input with both a pass and a failure: not relied on\n"
                    ),
                    _ if !a.calls.is_empty() => format!(
                        "  no passing input with {text}: one test there stands for the others\n"
                    ),
                    (Some(spec), true) => format!(
                        "  class {class} of `{spec}` has an input with both a pass and a failure: not relied on\n"
                    ),
                    (Some(spec), false) => format!(
                        "  no passing input in class {class} of `{spec}`: one test there stands for the class\n"
                    ),
                    (None, true) => {
                        "  an input has both a pass and a failure: no test stands for the others\n"
                            .to_string()
                    }
                    (None, false) => {
                        "  no passing input: one test stands for every input\n".to_string()
                    }
                };
            }
            continue;
        }
        let subject = model.function(&c.func).is_some_and(|f| !f.is_spec());
        // Size first: listing a large finite set is slow.
        let size = missing.size_bound();
        match size {
            Some(n) if n <= LIST as f64 && matches!(c.target, Target::Property(_)) => {
                // The outputs the property reads on each input, for the
                // calls whose arguments are names or values.
                let Target::Property(e) = &c.target else {
                    unreachable!()
                };
                let names: Vec<&String> = c.pattern.iter().filter_map(|p| p.var.as_ref()).collect();
                let mut lines: Vec<String> = Vec::new();
                for input in missing.values().unwrap_or_default() {
                    crate::ast::walk(e, &mut |e| {
                        let crate::Expr::Call(g, args) = e else { return };
                        if !model.declared_function(g).is_some_and(|g| !g.is_spec()) {
                            return;
                        }
                        let value = |a: &crate::Expr| match a {
                            crate::Expr::Lit(v) => Some(v.clone()),
                            crate::Expr::Var(x) => {
                                let at = names.iter().position(|n| *n == x)?;
                                Some(input[at].clone())
                            }
                            _ => None,
                        };
                        let Some(values) = args.iter().map(value).collect::<Option<Vec<_>>>()
                        else {
                            return;
                        };
                        let line = ask(model, g, &values, true);
                        if !lines.contains(&line) {
                            lines.push(line);
                        }
                    });
                }
                out += &lines.concat();
            }
            Some(n) if subject && n <= LIST as f64 => {
                for input in missing.values().unwrap_or_default() {
                    out += &ask(model, &c.func, &input, matches!(c.target, Target::Matches(_)));
                }
            }
            Some(n) => {
                out += &format!(
                    "  {n} inputs: test them, or bring evidence over the whole set (a proof)\n"
                )
            }
            None => out += "  infinite: tests can't cover it, only evidence over a set (a proof),\n  or a `uniform` claim, so that one test stands for a class of inputs\n",
        }
    }
    if out.is_empty() {
        out += "nothing to do\n";
    }
    model.named(&out)
}

/// The evidence to bring on one call, in the syntax of the model.
fn ask(model: &Model, f: &str, input: &[Value], output: bool) -> String {
    let names = model
        .function(f)
        .map(|f| f.params.iter().map(|p| p.0.clone()));
    let given: Vec<String> = names
        .into_iter()
        .flatten()
        .zip(input)
        .map(|(n, v)| format!("{n} = {v}"))
        .collect();
    let given = match given.is_empty() {
        true => String::new(),
        false => format!(" given {}", given.join(", ")),
    };
    let says = if output { "gives <output>" } else { "holds" };
    format!("  evidence <name> {{ for {f}{given} {says} }}\n")
}

/// The evidence that vouches for a claim, in the syntax of the model.
pub(crate) fn vouch(model: &Model, claim: &str) -> String {
    let _ = model;
    format!("  evidence <name> {{ for {claim} holds source \"review:<who>\" }}\n")
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn say(model: &str, evidence: &str) -> String {
        let m = load(model, evidence).unwrap();
        next(&m, &evaluate(&m).unwrap())
    }

    #[test]
    fn lists_what_is_missing() {
        let model = "fn neg(a: Bool) -> Bool { !a }\n\
                     subject fn flip(a: Bool) -> Bool;\n\
                     subject fn id(x: Any) -> Any;\n\
                     claim flip_spec(a) { flip(a) == neg(a) }\n\
                     claim flip_correct(a) { correct(flip(a)) }\n\
                     claim id_correct(x) { correct(id(x)) }\n\
                     claim big(x: 0..=99) { correct(id(x)) }";
        let evidence = "evidence t { for flip given a = true gives false }\n\
                        evidence t { for flip given a = true holds }";
        assert_eq!(
            say(model, evidence),
            "flip_spec: missing {false}\n\
             \x20 evidence <name> { for flip given a = false gives <output> }\n\
             flip_correct: missing {false}\n\
             \x20 evidence <name> { for flip given a = false holds }\n\
             id_correct: missing Any\n\
             \x20 infinite: tests can't cover it, only evidence over a set (a proof),\n\
             \x20 or a `uniform` claim, so that one test stands for a class of inputs\n\
             big: missing 0..=99\n\
             \x20 100 inputs: test them, or bring evidence over the whole set (a proof)\n"
        );
    }

    #[test]
    fn sizes_large_sets() {
        let model = "subject fn pair(x: Int, y: Int) -> Bool;\n\
                     claim huge(x: 0..=999999999, y: 0..=9) { correct(pair(x, y)) }";
        let text = say(model, "evidence t { for pair given x = 0, y = 0 holds }");
        assert!(text.contains("  9999999999 inputs: test them"), "{text}");
    }

    #[test]
    fn names_untested_classes_and_conflicts() {
        let model = "import std::claims::{uniform};\n\
                     subject fn div(a: Int, b: Int) -> Int;\n\
                     fn div_class(a: Int, b: Int) -> {\"zero\", \"rest\"} { if b == 0 { \"zero\" } else { \"rest\" } }\n\
                     claim div_correct(a, b) { correct(div(a, b)) }\n\
                     claim div_uniform = uniform(div, div_class);";
        let pass = "evidence t { for div given a = 6, b = 3 holds }\n";
        let vouched = format!("{pass}evidence r {{ for div_uniform holds }}\n");
        assert_eq!(
            say(model, pass),
            "div_correct: missing 1 of the classes of `div_class`\n\
             \x20 `div_uniform` is not vouched for: what rests on it is only assumed\n\
             \x20 no passing input in class \"zero\" of `div_class`: one test there stands for the class\n\
             div_uniform: not vouched for\n\
             \x20 review that `div` behaves the same within each class of `div_class`, then:\n\
             \x20 evidence <name> { for div_uniform holds source \"review:<who>\" }\n"
        );
        assert_eq!(
            say(model, &vouched),
            "div_correct: missing 1 of the classes of `div_class`\n\
             \x20 no passing input in class \"zero\" of `div_class`: one test there stands for the class\n"
        );
        let text = say(
            model,
            &format!("{vouched}evidence t {{ for div given a = 4, b = 2 fails }}"),
        );
        assert!(
            text.starts_with(
                "div_uniform: fails: `div(6, 3)` passes and `div(4, 2)` fails, both in class \"rest\" of `div_class`\n"
            ),
            "{text}"
        );
        assert!(text.contains("div_correct: fails on {(4, 2)}\n"), "{text}");
    }
}
