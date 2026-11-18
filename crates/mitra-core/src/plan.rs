//! `mitra plan`: per claim, the evidence still owed. It is worked out
//! from the claims and the evidence already there, with or without code:
//! with no evidence at all it is the list of tests to write.

use crate::ast::{calls, walk, Claim, Expr, Model, Target};
use crate::eval::{Assumed, Logic, Report, Verdict};
use crate::print::call;
use crate::Value;

/// Up to this many owed points are listed one by one.
const LIST: f64 = 12.0;

/// A set printed longer than this is not printed.
const WIDE: usize = 160;

pub fn plan(model: &Model, report: &Report) -> String {
    let mut out = String::new();
    let mut seen: Vec<&str> = Vec::new();
    for v in &report.verdicts {
        if seen.contains(&v.claim.as_str()) {
            continue;
        }
        seen.push(&v.claim);
        let c = model.claim(&v.claim).expect("verdicts name claims");
        match v.logic {
            Logic::Fails => out += &format!("{} owes nothing: it fails (`mitra eval`)\n", c.name),
            Logic::Unverified => out += &owed(model, report, c, v),
            // Held up by other claims only: its own points are still owed,
            // unless what it assumes gets its evidence.
            Logic::Assuming if !v.own_word() && !v.derived.is_empty() => {
                out += &owed(model, report, c, v);
                let s = if v.assuming.len() == 1 { "s" } else { "" };
                out += &format!("  or nothing, once {} hold{s}\n", v.assuming.join(", "));
            }
            _ => {}
        }
    }
    if out.is_empty() {
        out += "nothing owed\n";
    }
    model.named(&out)
}

fn owed(model: &Model, report: &Report, c: &Claim, v: &Verdict) -> String {
    let head = format!("{} owed:", c.name);
    // An assumption a person checks: no test settles it.
    match &c.target {
        Target::Uniform(spec) => {
            let within = match spec {
                Some(_) => "within each class",
                None => "for every input",
            };
            return format!(
                "{head} a review that `{}` behaves the same {within}\n",
                c.func
            );
        }
        Target::Uses { used, .. } => {
            let used: Vec<String> = used.iter().map(|g| format!("`{g}`")).collect();
            return format!(
                "{head} a review that `{}` calls {}\n",
                c.func,
                used.join(", ")
            );
        }
        Target::Tied => return format!("{head} `mitra tie`, once the code is written\n"),
        Target::Represented(_) => {
            return format!(
                "{head} a review that one input of {} stands for the others, for `{}`\n",
                set_text(model, c),
                c.func
            )
        }
        _ => {}
    }
    // One point stands for its class.
    if let Some(a) = v
        .assumed
        .as_ref()
        .filter(|a| !a.untested.is_empty() && !a.blind)
    {
        let mut out = format!("{head}\n");
        for class in &a.untested {
            out += &format!("  any one point of {}\n", class_set(model, a, class));
        }
        let by = report.verdicts.iter().find(|w| w.claim == a.by);
        if by.is_some_and(|w| w.logic == Logic::Unverified) {
            out += &format!("  and `{}`, which says one point is enough\n", a.by);
        }
        return out;
    }
    let mut missed = v.missed.union(&v.derived);
    // One point of a `represented` set stands for the set.
    let mut sets = String::new();
    let on_code = matches!(c.target, Target::Correct | Target::Matches(_));
    for r in model.claims.iter().filter(|r| r.func == c.func && on_code) {
        let denied = report
            .verdicts
            .iter()
            .any(|w| w.claim == r.name && w.logic == Logic::Fails);
        let Some(set) = crate::eval::represented(model, r).filter(|_| !denied) else {
            continue;
        };
        let inside = set.intersect(&v.domain);
        if !inside.intersect(&missed).is_empty() && inside.intersect(&v.tested).is_empty() {
            sets += &format!("  any one point of {}\n", set_text(model, r));
            missed = missed.minus(&inside);
        }
    }
    if !sets.is_empty() && missed.is_empty() {
        return format!("{head}\n{sets}");
    }
    let head = match sets.is_empty() {
        true => head,
        false => format!("{head}\n{sets}  and"),
    };
    let size = missed.size_bound();
    let mut shown = missed.show(&v.domain);
    if shown.len() > WIDE {
        shown = format!(
            "the inputs not covered ({} chars to print: `mitra eval --all`)",
            shown.len()
        );
    }
    let Some(points) = missed.values().filter(|_| size.is_some_and(|n| n <= LIST)) else {
        return match size {
            Some(n) => {
                format!("{head} every point of {shown} ({n}), or a claim one point can stand for\n")
            }
            None => {
                format!("{head} {shown} is infinite: a claim one point can stand for, or a proof\n")
            }
        };
    };
    let names: Vec<&String> = c.pattern.iter().filter_map(|p| p.var.as_ref()).collect();
    let mut lines: Vec<String> = Vec::new();
    let mut push = |line: String| {
        if !lines.contains(&line) {
            lines.push(line);
        }
    };
    for x in points {
        let env: Vec<(String, Value)> = names
            .iter()
            .map(|n| (*n).clone())
            .zip(x.iter().cloned())
            .collect();
        match &c.target {
            Target::Matches(e) => {
                let expected = crate::eval::computed(model, e, &env);
                if !observed(model, &c.func, &x) {
                    push(point(&call(&c.func, &x), expected));
                }
                // The other side may read the code too.
                walk(e, &mut |sub| {
                    let Expr::Call(g, args) = sub else { return };
                    if !model.declared_function(g).is_some_and(|g| !g.is_spec()) {
                        return;
                    }
                    let values: Option<Vec<Value>> = args
                        .iter()
                        .map(|a| crate::eval::computed(model, a, &env))
                        .collect();
                    if let Some(values) = values.filter(|v| !observed(model, g, v)) {
                        push(point(&call(g, &values), None));
                    }
                });
            }
            Target::Correct => push(format!("  {}, judged correct\n", call(&c.func, &x))),
            Target::Property(e) => {
                // `f(…) == e`: what the claim itself expects of the call.
                let stated = match e {
                    Expr::Binary("==", l, r) => [(l, r), (r, l)]
                        .into_iter()
                        .find(|(a, b)| subject_call(model, a) && calls_no_subject(model, b)),
                    _ => None,
                };
                let mut found = false;
                walk(e, &mut |sub| {
                    let Expr::Call(g, args) = sub else { return };
                    if !model.declared_function(g).is_some_and(|g| !g.is_spec()) {
                        return;
                    }
                    let values: Option<Vec<Value>> = args
                        .iter()
                        .map(|a| crate::eval::computed(model, a, &env))
                        .collect();
                    let Some(values) = values else { return };
                    found = true;
                    if observed(model, g, &values) {
                        return;
                    }
                    let expected = stated
                        .filter(|(a, _)| std::ptr::eq::<Expr>(&***a, sub))
                        .and_then(|(_, b)| crate::eval::computed(model, b, &env));
                    push(point(&call(g, &values), expected));
                });
                if !found {
                    push(format!("  evidence on the claim at {}\n", call("", &x)));
                }
            }
            _ => {}
        }
    }
    format!("{head}\n{}", lines.concat())
}

fn point(call: &str, expected: Option<Value>) -> String {
    match expected {
        Some(v) => format!("  {call}, expected {v}\n"),
        None => format!("  {call}\n"),
    }
}

fn subject_call(model: &Model, e: &Expr) -> bool {
    matches!(e, Expr::Call(g, _) if model.declared_function(g).is_some_and(|g| !g.is_spec()))
}

fn calls_no_subject(model: &Model, e: &Expr) -> bool {
    !calls(e)
        .iter()
        .any(|g| model.declared_function(g).is_some_and(|g| !g.is_spec()))
}

/// A class as a set of inputs: `{ (a, b) | b == 0 }`.
fn class_set(model: &Model, a: &Assumed, class: &Value) -> String {
    let Some(spec) = a.spec.as_ref().filter(|_| a.calls.is_empty()) else {
        let text = crate::eval::class_text(std::slice::from_ref(class), &a.spec, &a.calls);
        return match text.is_empty() {
            true => "its inputs".to_string(),
            false => format!("the inputs with {text}"),
        };
    };
    let Some(f) = model.function(spec) else {
        return format!("class {class} of `{spec}`");
    };
    let names: Vec<&str> = f.params.iter().map(|p| p.0.as_str()).collect();
    let inputs = match names.len() {
        1 => names[0].to_string(),
        _ => format!("({})", names.join(", ")),
    };
    let body = f.body.as_ref().map(crate::print::expr).unwrap_or_default();
    let cond = match class {
        Value::Bool(true) => body,
        Value::Bool(false) => format!("!({body})"),
        v => format!("{body} == {v}"),
    };
    format!("{{ {inputs} | {cond} }}")
}

/// Whether an output of this call is already in the evidence.
fn observed(model: &Model, g: &str, values: &[Value]) -> bool {
    model.evidence.iter().any(|e| {
        let point = e.pattern.iter().map(|p| p.set.as_ref()?.single());
        e.about == g
            && matches!(e.judgment, crate::Judgment::Output(_))
            && point.collect::<Option<Vec<Value>>>().as_deref() == Some(values)
    })
}

/// The set of a `represented` claim, as written.
fn set_text(model: &Model, r: &Claim) -> String {
    let written = model.written.as_ref();
    let text = written.and_then(|w| crate::parse::written(&w.0, r.line));
    let inside = text.as_deref().and_then(|t| {
        let from = t.find("represented(")?;
        let start = from + t[from..].find(',')? + 1;
        let end = t.rfind(')')?;
        Some(t.get(start..end)?.trim().to_string())
    });
    inside.unwrap_or_else(|| format!("the set of `{}`", r.name))
}
