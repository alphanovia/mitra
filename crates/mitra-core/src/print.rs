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
        let head = format!("{}{} -> {}", f.name, binders(&f.params), f.range);
        out += &match &f.body {
            None => format!("opaque {head}\n"),
            Some(e) => format!("def {head} := {}\n", expr(e)),
        };
    }
    for c in &m.claims {
        out += &print_claim(m, c);
        out += "\n";
    }
    for t in &m.tests {
        let line = print_evidence(std::slice::from_ref(&t.judgment), Some(m));
        let judgment = line.trim_end().strip_prefix("evidence ").unwrap();
        let judgment = judgment.strip_suffix(" at 1").unwrap();
        out += &format!("test {} : {judgment}\n", t.name);
    }
    out
}

/// One claim, as written in a model, without the line break.
pub fn print_claim(m: &Model, c: &Claim) -> String {
    if let Target::Uniform(g) = &c.target {
        let by = g.as_ref().map(|g| format!(" by {g}")).unwrap_or_default();
        return format!("claim {} : uniform {}{by}", c.name, c.func);
    }
    if let Target::Uses { used, only } = &c.target {
        let only = if *only { "only " } else { "" };
        let used = used.join(", ");
        return format!("claim {} : {only}uses {} [{used}]", c.name, c.func);
    }
    let params = m.function(&c.func).map(|f| &f.params[..]).unwrap_or(&[]);
    let (forall, args) = pattern(&c.pattern, params);
    let body = match &c.target {
        Target::Correct => format!("correct({}{args})", c.func),
        Target::Matches(e) => format!("{}{args} = {}", c.func, expr(e)),
        Target::Uniform(_) | Target::Uses { .. } => unreachable!(),
    };
    let from = if c.from.is_empty() {
        String::new()
    } else {
        format!(" := by derive [{}]", c.from.join(", "))
    };
    format!("claim {} : {forall}{body}{from}", c.name)
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
            Judgment::Correct(true) => format!("correct({call})"),
            Judgment::Correct(false) => format!("not correct({call})"),
            Judgment::Holds(true) => call,
            Judgment::Holds(false) => format!("not {call}"),
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

/// `(a: Int, b: Int, c: Bool)`
fn binders(params: &[(String, Set)]) -> String {
    let each: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    format!("({})", each.join(", "))
}

/// A pattern as `forall` binders and the arguments of the call.
fn pattern(p: &Pattern, params: &[(String, Set)]) -> (String, String) {
    let mut forall = String::new();
    let mut args = Vec::new();
    for (i, pos) in p.iter().enumerate() {
        let value = match (&pos.var, &pos.set) {
            (None, Some(s)) => s.single(),
            _ => None,
        };
        if let Some(v) = value {
            args.push(v.to_string());
            continue;
        }
        let x = pos.var.clone().unwrap_or_else(|| match params.get(i) {
            Some((name, _)) => name.clone(),
            None => format!("x{}", i + 1),
        });
        forall += &match &pos.set {
            None => format!("forall {x}, "),
            Some(s) => format!("forall {x} in {s}, "),
        };
        args.push(x);
    }
    let args = if args.is_empty() {
        String::new()
    } else {
        format!("({})", args.join(", "))
    };
    (forall, args)
}

/// `f(1, -2)`: a call on one input tuple.
pub fn call(f: &str, input: &[Value]) -> String {
    let args: Vec<String> = input.iter().map(|v| v.to_string()).collect();
    format!("{f}({})", args.join(", "))
}

fn negative(v: &Value) -> bool {
    match v {
        Value::Int(i) => *i < 0,
        Value::Float(x) => x.is_sign_negative(),
        _ => false,
    }
}

/// Binding strength, loosest first.
const LET: u8 = 0;
const UNARY: u8 = 6;
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
        Expr::Lit(v) if negative(v) => (v.to_string(), UNARY),
        Expr::Lit(v) => (v.to_string(), ATOM),
        Expr::Var(x) => (x.clone(), ATOM),
        Expr::Call(f, args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("{f}({})", args.join(", ")), ATOM)
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
