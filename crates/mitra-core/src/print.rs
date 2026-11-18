//! The text projection, the other way: print a model or evidence back as
//! Mitra text that parses to the same AST.
//!
//! Commands come out grouped (functions, claims, profiles, policies), one
//! per line. Expressions get the fewest parentheses that keep their meaning.

use crate::ast::*;
use crate::set::{Set, Value};

pub fn print_model(m: &Model) -> String {
    let mut out = String::new();
    // In the order written: a record may use an enum, and an enum a record.
    for d in &m.declared {
        match d {
            Decl::Record(i) => {
                let r = &m.records[*i];
                out += &format!("record {}{}\n", r.name, binders(&r.fields));
            }
            Decl::Enum(i) => {
                let e = &m.enums[*i];
                let params = match e.params.is_empty() {
                    true => String::new(),
                    false => format!("({})", e.params.join(", ")),
                };
                out += &format!("enum {}{params} {{ {} }}\n", e.name, e.body);
            }
        }
    }
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
        Target::Property(e) => expr(e),
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
        if let Judgment::Measured(measure, value) = &e.judgment {
            let attrs = attributes(e);
            let (m, c) = (expr(measure), e.confidence);
            out += &format!("{attrs}evidence {m} = {value} at {c}\n");
            continue;
        }
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
            Judgment::Measured(..) => unreachable!(),
        };
        let attrs = attributes(e);
        out += &format!("{attrs}evidence {forall}{judgment} at {}\n", e.confidence);
    }
    out
}

/// `@[source "…", rev "…"] `, or nothing.
fn attributes(e: &Evidence) -> String {
    let mut attrs = Vec::new();
    if let Some(s) = &e.source {
        attrs.push(format!("source {}", crate::set::quote(s)));
    }
    if let Some(r) = &e.rev {
        attrs.push(format!("rev {}", crate::set::quote(r)));
    }
    match attrs.is_empty() {
        true => String::new(),
        false => format!("@[{}] ", attrs.join(", ")),
    }
}

/// `(a: Int, b: Int, c: Bool)`
fn binders(params: &[(String, Set)]) -> String {
    let each: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    format!("({})", each.join(", "))
}

/// A pattern as `forall` binders and the arguments of the call.
pub(crate) fn pattern(p: &Pattern, params: &[(String, Set)]) -> (String, String) {
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

/// mitra: opaque negative(v: Value) -> Bool
/// mitra: claim negative_correct : forall v, correct(negative(v))
/// mitra: -- `Value` hides the type: a spec can't ask whether `v` is a number.
/// mitra: claim negative_ints : forall v in Int, negative(v) == (v < 0)
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

/// mitra: opaque level(op: String) -> UInt8
/// mitra: def level_spec(op: String) -> 1..=5 :=
/// mitra:   if op == "||" then 1 else if op == "&&" then 2 else if op == "+" || op == "-" then 4
/// mitra:   else if op == "*" || op == "/" then 5 else 3
/// mitra: claim level_is : forall op, level(op) = level_spec(op)
/// mitra: -- One class per arm of the `match`.
/// mitra: claim level_uniform : uniform level by level_spec
fn level(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "+" | "-" => 4,
        "*" | "/" => 5,
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
        Expr::Tuple(args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("({})", args.join(", ")), ATOM)
        }
        Expr::Record(name, args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("{name}({})", args.join(", ")), ATOM)
        }
        Expr::Field(a, field) => {
            // `t.0.1` and `5.0` would read as numbers: `(t.0).1`, `(5).0`.
            let numeric = |f: &str| f.starts_with(|c: char| c.is_ascii_digit());
            let number = match &**a {
                Expr::Field(_, f) => numeric(f),
                Expr::Lit(Value::Int(_)) => true,
                _ => false,
            };
            let inner = at(a, ATOM);
            match number && numeric(field) {
                true => (format!("({inner}).{field}"), ATOM),
                false => (format!("{inner}.{field}"), ATOM),
            }
        }
        Expr::List(args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("[{}]", args.join(", ")), ATOM)
        }
        Expr::Index(a, i) => (format!("{}[{}]", at(a, ATOM), expr(i)), ATOM),
        Expr::Len(a) => (format!("len({})", expr(a)), ATOM),
        Expr::Rate(x, set, a) => (format!("rate({x} in {set}, {})", expr(a)), ATOM),
        Expr::Score(kind, f) => (format!("{kind}({f})"), ATOM),
        Expr::Match(a, arms) => {
            let arms: Vec<String> = arms
                .iter()
                .map(|arm| {
                    let pattern = match &arm.variant {
                        None => "_".to_string(),
                        Some((name, fields)) if fields.is_empty() => name.clone(),
                        Some((name, fields)) => format!("{name}({})", fields.join(", ")),
                    };
                    format!("{pattern} => {}", expr(&arm.body))
                })
                .collect();
            (format!("match {} {{ {} }}", expr(a), arms.join(", ")), ATOM)
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

#[cfg(test)]
mod tests {
    use super::*;

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.

    // mitra: test t_level_or : level("||") = 1
    #[test]
    fn t_level_or() {
        assert_eq!(level("||"), 1);
    }

    // mitra: test t_level_and : level("&&") = 2
    #[test]
    fn t_level_and() {
        assert_eq!(level("&&"), 2);
    }

    // mitra: test t_level_compare : level("<=") = 3
    #[test]
    fn t_level_compare() {
        assert_eq!(level("<="), 3);
    }

    // mitra: test t_level_sum : level("-") = 4
    #[test]
    fn t_level_sum() {
        assert_eq!(level("-"), 4);
    }

    // mitra: test t_level_product : level("/") = 5
    #[test]
    fn t_level_product() {
        assert_eq!(level("/"), 5);
    }

    // mitra: test t_negative_int : negative(-1) = true
    #[test]
    fn t_negative_int() {
        assert!(negative(&Value::Int(-1)));
    }

    // mitra: test t_negative_zero : negative(0) = false
    #[test]
    fn t_negative_zero() {
        assert!(!negative(&Value::Int(0)));
    }

    // mitra: test t_negative_float : negative(-0.5) = true
    #[test]
    fn t_negative_float() {
        assert!(negative(&Value::Float(-0.5)));
    }

    // mitra: test t_negative_string : negative("-1") = false
    #[test]
    fn t_negative_string() {
        assert!(!negative(&Value::Str("-1".into())));
    }
}
