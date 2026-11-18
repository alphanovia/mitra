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
        out += &format!(
            "evidence lines {} and {} can't both be true, on {} of `{}`: settle them\n",
            k.pass.0, k.fail.0, k.inputs, k.subject
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
                 `uniform {subject} by <spec>`, or check the two results: one of them may be wrong\n",
                k.claim,
            ),
        };
        out += &look(subject);
    }
    verdicts.sort_by_key(|v| v.logic != Logic::Fails || !suspects(subject(v)).is_empty());
    for v in verdicts {
        let c = model.claim(&v.claim).expect("verdicts name claims");
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
    if model.written.is_none() {
        let call = call(f, input);
        return match output {
            true => format!("  evidence {call} = <output> at 1\n"),
            false => format!("  evidence correct({call}) at 1\n"),
        };
    }
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
    match model.written {
        None => format!("  evidence {claim} at 1\n"),
        Some(_) => format!("  evidence <name> {{ for {claim} holds source \"review:<who>\" }}\n"),
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn lists_what_is_missing() {
        let model = "def neg(a: Bool) -> Bool := !a\n\
                     opaque flip(a: Bool) -> Bool\n\
                     opaque id(x: Value) -> Value\n\
                     claim flip_spec : forall a, flip(a) = neg(a)\n\
                     claim flip_correct : forall a, correct(flip(a))\n\
                     claim id_correct : forall x, correct(id(x))\n\
                     claim big : forall x in 0..=99, correct(id(x))\n\
                     claim derived : forall a, flip(a) = neg(a) := by derive [flip_spec]";
        let evidence = "evidence flip(true) = false at 1\nevidence correct(flip(true)) at 1";
        let m = load(model, evidence).unwrap();
        let r = evaluate(&m).unwrap();
        assert_eq!(
            next(&m, &r),
            "flip_spec: missing {false}\n\
             \x20 evidence flip(false) = <output> at 1\n\
             flip_correct: missing {false}\n\
             \x20 evidence correct(flip(false)) at 1\n\
             id_correct: missing Value\n\
             \x20 infinite: tests can't cover it, only evidence over a set (a proof),\n\
             \x20 or a `uniform` claim, so that one test stands for a class of inputs\n\
             big: missing 0..=99\n\
             \x20 100 inputs: test them, or bring evidence over the whole set (a proof)\n\
             derived: holds assuming [flip_spec]\n\
             \x20 bring evidence on what it assumes, such as a review, or test until it is not needed\n"
        );
    }

    #[test]
    fn sizes_large_sets_and_names_failing_premises() {
        let model = "opaque pair(x: Int, y: Int) -> Bool\n\
                     opaque flip(a: Bool) -> Bool\n\
                     opaque other(a: Bool) -> Bool\n\
                     claim huge : forall x in 0..=999999999, forall y in 0..=9, correct(pair(x, y))\n\
                     claim fails : forall a, correct(flip(a))\n\
                     claim on_fails : forall a, correct(other(a)) := by derive [fails]";
        let evidence = "evidence correct(pair(0, 0)) at 1\nevidence not correct(flip(true)) at 1";
        let m = load(model, evidence).unwrap();
        let r = evaluate(&m).unwrap();
        let text = next(&m, &r);
        assert!(text.contains("  9999999999 inputs: test them"), "{text}");
        assert!(text.contains("  derived: premise fails: fails\n"), "{text}");
    }

    #[test]
    fn names_untested_classes_and_conflicts() {
        let model = "opaque div(a: Int, b: Int) -> Int\n\
                     def div_class(a: Int, b: Int) -> {\"zero\", \"rest\"} := if b == 0 then \"zero\" else \"rest\"\n\
                     claim div_correct : forall a, forall b, correct(div(a, b))\n\
                     claim div_uniform : uniform div by div_class";
        let say = |evidence: &str| {
            let m = load(model, evidence).unwrap();
            next(&m, &evaluate(&m).unwrap())
        };
        assert_eq!(
            say("evidence correct(div(6, 3)) at 1"),
            "div_correct: missing 1 of the classes of `div_class`\n\
             \x20 `div_uniform` is not vouched for: what rests on it is only assumed\n\
             \x20 no passing input in class \"zero\" of `div_class`: one test there stands for the class\n\
             div_uniform: not vouched for\n\
             \x20 review that `div` behaves the same within each class of `div_class`, then:\n\
             \x20 evidence div_uniform at 1\n"
        );
        assert_eq!(
            say("evidence correct(div(6, 3)) at 1\nevidence div_uniform at 1"),
            "div_correct: missing 1 of the classes of `div_class`\n\
             \x20 no passing input in class \"zero\" of `div_class`: one test there stands for the class\n"
        );
        let text = say(
            "evidence correct(div(6, 3)) at 1\nevidence div_uniform at 1\n\
                        evidence not correct(div(4, 2)) at 1",
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
