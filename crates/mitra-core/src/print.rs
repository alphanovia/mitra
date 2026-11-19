//! Claims, evidence and expressions as the language writes them, for
//! reports. Expressions get the fewest parentheses that keep their meaning.

use crate::ast::*;
use crate::set::{Set, Value};

/// One claim, as a model would write it, without the line break.
pub fn print_claim(m: &Model, c: &Claim) -> String {
    format!("claim {} {{ {} }}", c.name, contract(m, c))
}

/// Evidence, one block each. `model` names the inputs after the
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
        let mut given = Vec::new();
        for (i, pos) in e.pattern.iter().enumerate() {
            let x = pos.var.clone().unwrap_or_else(|| match params.get(i) {
                Some((name, _)) => name.clone(),
                None => format!("x{}", i + 1),
            });
            match (pos.set.as_ref(), pos.set.as_ref().and_then(|s| s.single())) {
                (_, Some(v)) if pos.var.is_none() => given.push(format!("{x} = {v}")),
                (Some(s), _) => given.push(format!("{x} in {s}")),
                (None, _) => {}
            }
        }
        let given = match given.is_empty() {
            true => String::new(),
            false => format!(" given {}", given.join(", ")),
        };
        let says = match &e.judgment {
            Judgment::Correct(true) | Judgment::Holds(true) => "holds".to_string(),
            Judgment::Correct(false) | Judgment::Holds(false) => "fails".to_string(),
            Judgment::Output(v) => format!("gives {v}"),
            Judgment::Measured(_, value) => format!("gives rate = {value}"),
        };
        let mut more = String::new();
        if e.confidence != 1.0 {
            more += &format!(" trust {}", e.confidence);
        }
        if let Some(s) = &e.source {
            more += &format!(" source {}", crate::set::quote(s));
        }
        if let Some(r) = &e.rev {
            more += &format!(" rev {}", crate::set::quote(r));
        }
        out += &format!("evidence e {{ for {}{given} {says}{more} }}\n", e.about);
    }
    out
}

/// The arguments of the call a pattern stands for: `(a, 1)`.
pub(crate) fn pattern(p: &Pattern, params: &[(String, Set)]) -> (String, String) {
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
        args.push(x);
    }
    // A function without inputs is called as `f()`.
    (String::new(), format!("({})", args.join(", ")))
}

/// `f(1, -2)`: a call on one input tuple.
pub fn call(f: &str, input: &[Value]) -> String {
    let args: Vec<String> = input.iter().map(|v| v.to_string()).collect();
    format!("{f}({})", args.join(", "))
}

// mitra: implements print::negative
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

// mitra: implements print::level
fn level(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "+" | "-" | "|" | "\\" => 4,
        "*" | "/" | "&" => 5,
        _ => 3,
    }
}

pub fn expr(e: &Expr) -> String {
    at(e, LET)
}

/// One claim as a model would write it in full: `given a: S, b: T then e`.
pub fn contract(m: &Model, c: &Claim) -> String {
    if let Some(e) = m.captured(&c.name) {
        return format!("the claims of its subject say what `{}` says", e.name);
    }
    if let Some(e) = m.ensure(&c.name) {
        let mut on = e.claims.clone();
        on.extend(e.requires.iter().cloned());
        if !e.boundary {
            on.push(format!("{}::captured", e.name));
        }
        return match on.is_empty() {
            true => format!("{:?}, assumed", e.prose),
            false => format!("{:?}, resting on {}", e.prose, on.join(", ")),
        };
    }
    match &c.target {
        Target::Uniform(None) => format!("uniform({})", c.func),
        Target::Uniform(Some(g)) => {
            // Classes written inline: as written.
            let inline = m.function(g).filter(|_| g.ends_with("::case"));
            let classes = match inline.and_then(|g| Some((g, g.body.as_ref()?))) {
                Some((g, body)) => {
                    let names: Vec<&str> = g.params.iter().map(|p| p.0.as_str()).collect();
                    match g.range == Set::bool() {
                        true => format!("|{}| {}", names.join(", "), expr(body)),
                        false => {
                            format!("|{}| -> {} {{ {} }}", names.join(", "), g.range, expr(body))
                        }
                    }
                }
                None => g.clone(),
            };
            format!("uniform({}, {classes})", c.func)
        }
        Target::Uses { used, only } => {
            let only = if *only { "only " } else { "" };
            format!("fn {} uses {only}{}", c.func, used.join(", "))
        }
        Target::Tied => format!("the code of {} is tied to the model", c.func),
        Target::Represented(set) => {
            let g = m.function(set);
            let names: Vec<String> = g
                .map(|g| g.params.iter().map(|p| p.0.clone()).collect())
                .unwrap_or_default();
            let cond = g
                .and_then(|g| g.body.as_ref())
                .map(expr)
                .unwrap_or_default();
            let sets: Vec<String> = g
                .map(|g| g.params.iter().map(|p| p.1.to_string()).collect())
                .unwrap_or_default();
            match cond == "true" {
                true => format!("represented({}, ({}))", c.func, sets.join(", ")),
                false => format!(
                    "represented({}, {{ ({}) | {cond} }})",
                    c.func,
                    names.join(", ")
                ),
            }
        }
        _ => {
            let f = m.function(&c.func);
            let params = f.map(|f| &f.params[..]).unwrap_or(&[]);
            let mut given = Vec::new();
            let mut args = Vec::new();
            for (i, pos) in c.pattern.iter().enumerate() {
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
                let set = pos.set.as_ref().or(params.get(i).map(|p| &p.1));
                given.push(match set {
                    Some(s) => format!("{x}: {s}"),
                    None => x.clone(),
                });
                args.push(x);
            }
            let args = format!("({})", args.join(", "));
            let body = match &c.target {
                Target::Correct => format!("correct({}{args})", c.func),
                Target::Matches(e) => {
                    // The claim a body adds: the body itself.
                    let body = match e {
                        Expr::Call(g, _) if g == &format!("{}::body", c.func) => {
                            m.function(g).and_then(|g| g.body.as_ref())
                        }
                        _ => None,
                    };
                    format!("{}{args} == {}", c.func, expr(body.unwrap_or(e)))
                }
                Target::Property(e) => expr(e),
                _ => unreachable!(),
            };
            match given.is_empty() {
                true => body,
                false => format!("given {} then {body}", given.join(", ")),
            }
        }
    }
}

/// `e`, in a place that needs at least binding strength `min`.
fn at(e: &Expr, min: u8) -> String {
    let (s, strength) = match e {
        Expr::Lit(v) if negative(v) => (v.to_string(), UNARY),
        Expr::Lit(v) => (v.to_string(), ATOM),
        Expr::Var(x) => (x.clone(), ATOM),
        Expr::Correct(f, args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("correct({f}({}))", args.join(", ")), ATOM)
        }
        Expr::Call(f, args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("{f}({})", args.join(", ")), ATOM)
        }
        Expr::Tuple(args) => {
            let args: Vec<String> = args.iter().map(expr).collect();
            (format!("({})", args.join(", ")), ATOM)
        }
        // A set `{a, b}` or a map `{k: v}`.
        Expr::Record(name, args) if name == crate::set::SET || name == crate::set::MAP => {
            let items = match &args[..] {
                [Expr::List(items)] => &items[..],
                _ => &[],
            };
            let each = |e: &Expr| match (e, name == crate::set::MAP) {
                (Expr::Tuple(kv), true) => format!("{}: {}", expr(&kv[0]), expr(&kv[1])),
                _ => expr(e),
            };
            let items: Vec<String> = items.iter().map(each).collect();
            match items.is_empty() && name == crate::set::MAP {
                true => ("{:}".to_string(), ATOM),
                false => (format!("{{{}}}", items.join(", ")), ATOM),
            }
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
        Expr::Each(kind, x, list, a) => {
            let list = at(list, level("+"));
            (format!("{kind}({x} in {list}, {})", expr(a)), ATOM)
        }
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
        Expr::Unary("keys", a) => (format!("keys({})", expr(a)), ATOM),
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
            format!("if {} {{ {} }} else {{ {} }}", expr(c), expr(t), expr(f)),
            ATOM,
        ),
        Expr::Let(x, v, b) => (format!("let {x} = {}; {}", expr(v), expr(b)), LET),
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

    // mitra: evidence for t_level_or
    #[test]
    fn t_level_or() {
        assert_eq!(level("||"), 1);
    }

    // mitra: evidence for t_level_and
    #[test]
    fn t_level_and() {
        assert_eq!(level("&&"), 2);
    }

    // mitra: evidence for t_level_compare
    #[test]
    fn t_level_compare() {
        assert_eq!(level("<="), 3);
    }

    // mitra: evidence for t_level_sum
    #[test]
    fn t_level_sum() {
        assert_eq!(level("-"), 4);
    }

    // mitra: evidence for t_level_union
    #[test]
    fn t_level_union() {
        assert_eq!(level("|"), 4);
    }

    // mitra: evidence for t_level_intersection
    #[test]
    fn t_level_intersection() {
        assert_eq!(level("&"), 5);
    }

    // mitra: evidence for t_level_in
    #[test]
    fn t_level_in() {
        assert_eq!(level("in"), 3);
    }

    // mitra: evidence for t_level_product
    #[test]
    fn t_level_product() {
        assert_eq!(level("/"), 5);
    }

    // mitra: evidence for t_negative_int
    #[test]
    fn t_negative_int() {
        assert!(negative(&Value::Int(-1)));
    }

    // mitra: evidence for t_negative_zero
    #[test]
    fn t_negative_zero() {
        assert!(!negative(&Value::Int(0)));
    }

    // mitra: evidence for t_negative_float
    #[test]
    fn t_negative_float() {
        assert!(negative(&Value::Float(-0.5)));
    }

    // mitra: evidence for t_negative_string
    #[test]
    fn t_negative_string() {
        assert!(!negative(&Value::Str("-1".into())));
    }
}
