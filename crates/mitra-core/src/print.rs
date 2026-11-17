//! The text projection, the other way: print a model or evidence back as
//! Mitra text that parses to the same AST.
//!
//! Commands come out grouped (functions, claims, profiles, policies), one
//! per line. Expressions get the fewest parentheses that keep their meaning.

use crate::ast::*;
use crate::set::{Set, Value};

pub fn print_model(m: &Model) -> String {
    let mut out = String::new();
    for f in &m.functions {
        let head = format!("{}{} : {}", f.name, binders(&f.params), f.range);
        out += &match &f.body {
            None => format!("opaque {head}\n"),
            Some(e) => format!("def {head} := {}\n", expr(e)),
        };
    }
    for c in &m.claims {
        let params = m.function(&c.func).map(|f| &f.params[..]).unwrap_or(&[]);
        let (forall, args) = pattern(&c.pattern, params);
        let body = match &c.target {
            Target::Correct => format!("correct ({}{args})", c.func),
            Target::Matches(e) => format!("{}{args} = {}", c.func, expr(e)),
        };
        let from = if c.from.is_empty() {
            String::new()
        } else {
            format!(" := by derive [{}]", c.from.join(", "))
        };
        out += &format!("claim {} : {forall}{body}{from}\n", c.name);
    }
    for t in &m.tests {
        let line = print_evidence(std::slice::from_ref(&t.judgment), Some(m));
        let judgment = line.trim_end().strip_prefix("evidence ").unwrap();
        let judgment = judgment.strip_suffix(" at 1").unwrap();
        out += &format!("test {} : {judgment}\n", t.name);
    }
    for p in &m.profiles {
        out += &format!("profile {} for {}\n", p.name, p.subject);
        for (region, w) in &p.regions {
            out += &match region {
                Some(s) => format!("  | x ∈ {s} => {w}\n"),
                None => format!("  | _ => {w}\n"),
            };
        }
    }
    for p in &m.policies {
        out += &match &p.kind {
            PolicyKind::Strict if p.name == "strict" => "policy strict\n".to_string(),
            PolicyKind::Strict => format!("policy {} := strict\n", p.name),
            PolicyKind::Coverage(profile) => format!("policy {} := coverage {profile}\n", p.name),
        };
    }
    out
}

/// Evidence, one line each. `model` names the bound inputs after the
/// parameters; without it they are `x1`, `x2`, ….
pub fn print_evidence(evidence: &[Evidence], model: Option<&Model>) -> String {
    let mut out = String::new();
    for e in evidence {
        let func = match &e.judgment {
            Judgment::Holds(_) => model
                .and_then(|m| m.claim(&e.about))
                .map(|c| c.func.as_str()),
            _ => Some(e.about.as_str()),
        };
        let params = func
            .and_then(|f| model?.function(f))
            .map(|f| &f.params[..])
            .unwrap_or(&[]);
        let whole = e.pattern.iter().all(|p| p.var.is_none() && p.set.is_none());
        let (forall, args) = match &e.judgment {
            Judgment::Holds(_) if whole => (String::new(), String::new()),
            _ => pattern(&e.pattern, params),
        };
        let call = format!("{}{args}", e.about);
        let judgment = match &e.judgment {
            Judgment::Correct(true) => format!("correct ({call})"),
            Judgment::Correct(false) => format!("¬ correct ({call})"),
            Judgment::Holds(true) => call,
            Judgment::Holds(false) => format!("¬ {call}"),
            Judgment::Output(v) => format!("{call} = {v}"),
        };
        let mut attrs = Vec::new();
        if let Some(s) = &e.source {
            attrs.push(format!("source \"{s}\""));
        }
        if let Some(r) = &e.rev {
            attrs.push(format!("rev \"{r}\""));
        }
        let attrs = if attrs.is_empty() {
            String::new()
        } else {
            format!("@[{}] ", attrs.join(", "))
        };
        out += &format!("{attrs}evidence {forall}{judgment} at {}\n", e.confidence);
    }
    out
}

/// `(a b : Int) (c : Bool)`: neighbours of the same type share a binder.
fn binders(params: &[(String, Set)]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < params.len() {
        let mut j = i;
        while j + 1 < params.len() && params[j + 1].1 == params[i].1 {
            j += 1;
        }
        let names: Vec<&str> = params[i..=j].iter().map(|p| p.0.as_str()).collect();
        out += &format!(" ({} : {})", names.join(" "), params[i].1);
        i = j + 1;
    }
    out
}

/// A pattern as `∀` binders and the arguments of the call.
fn pattern(p: &Pattern, params: &[(String, Set)]) -> (String, String) {
    let mut forall = String::new();
    let mut args = String::new();
    for (i, pos) in p.iter().enumerate() {
        let value = match (&pos.var, &pos.set) {
            (None, Some(s)) => s.single(),
            _ => None,
        };
        if let Some(v) = value {
            args += &format!(" {}", value_arg(&v));
            continue;
        }
        let x = pos.var.clone().unwrap_or_else(|| match params.get(i) {
            Some((name, _)) => name.clone(),
            None => format!("x{}", i + 1),
        });
        forall += &match &pos.set {
            None => format!("∀ {x}, "),
            Some(s) => format!("∀ {x} ∈ {s}, "),
        };
        args += &format!(" {x}");
    }
    (forall, args)
}

/// `f 1 (-2)`: a call on one input tuple.
pub fn call(f: &str, input: &[Value]) -> String {
    let args: String = input.iter().map(|v| format!(" {}", value_arg(v))).collect();
    format!("{f}{args}")
}

/// A value as an argument: negative integers need parentheses.
fn value_arg(v: &Value) -> String {
    match v {
        Value::Int(i) if *i < 0 => format!("({i})"),
        v => v.to_string(),
    }
}

/// Binding strength, loosest first.
const LET: u8 = 0;
const UNARY: u8 = 6;
const APP: u8 = 7;
const ATOM: u8 = 8;

fn level(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "+" | "-" => 4,
        "*" => 5,
        _ => 3,
    }
}

pub fn expr(e: &Expr) -> String {
    at(e, LET)
}

/// `e`, in a place that needs at least binding strength `min`.
fn at(e: &Expr, min: u8) -> String {
    let (s, strength) = match e {
        Expr::Lit(Value::Int(i)) if *i < 0 => (i.to_string(), UNARY),
        Expr::Lit(v) => (v.to_string(), ATOM),
        Expr::Var(x) => (x.clone(), ATOM),
        Expr::Call(f, args) => {
            let args: Vec<String> = args.iter().map(|a| at(a, ATOM)).collect();
            (format!("{f} {}", args.join(" ")), APP)
        }
        Expr::Unary(op, a) => {
            let inner = at(a, UNARY);
            // `- -x` must not print as the comment `--x`, and `-(5)` must not
            // print as the literal `-5`.
            let glued = inner.starts_with('-')
                || (*op == "-" && inner.starts_with(|c: char| c.is_ascii_digit()));
            let inner = if glued { format!("({inner})") } else { inner };
            (format!("{op}{inner}"), UNARY)
        }
        Expr::Binary(op, a, b) => {
            let l = level(op);
            (format!("{} {op} {}", at(a, l), at(b, l + 1)), l)
        }
        Expr::If(c, t, f) => (
            format!("if {} then {} else {}", expr(c), expr(t), expr(f)),
            LET,
        ),
        Expr::Let(x, v, b) => (format!("let {x} := {}; {}", expr(v), expr(b)), LET),
    };
    if strength < min {
        format!("({s})")
    } else {
        s
    }
}
