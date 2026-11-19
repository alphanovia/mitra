//! The type of every expression, worked out from the text. A type is a
//! set of values; an expression is refused when the set it can be never
//! meets the set its place takes: `g(a) + "x"`, a field of an `Int`, a
//! call with an input the function never accepts. Where nothing is known
//! the type is `Any`, and nothing is refused.

use crate::ast::{Expr, Function, Model, Target};
use crate::set::Set;
use crate::Error;

type Env = Vec<(String, Set)>;

/// Check every body and every claim of the model.
pub fn check(m: &Model) -> Result<(), Error> {
    let at = |line: usize| move |msg: String| Error::new(line, msg);
    for f in m.functions.iter().chain(&m.properties) {
        let Some(body) = &f.body else { continue };
        let mut env: Env = f.params.clone();
        let got = of(m, body, &mut env).map_err(at(f.line))?;
        if got.intersect(&f.range).is_empty() && f.written.is_none() {
            let (got, want) = (show(m, &got), show(m, &f.range));
            let msg = format!(
                "`{}` gives {got}, never its result type {want}",
                plain(&f.name)
            );
            return Err(Error::new(f.line, msg));
        }
    }
    for c in &m.claims {
        let (Target::Matches(e) | Target::Property(e)) = &c.target else {
            continue;
        };
        let params = m.function(&c.func).map(|f| f.params.clone());
        let mut env: Env = params.unwrap_or_default();
        let got = of(m, e, &mut env).map_err(at(c.line))?;
        if matches!(c.target, Target::Property(_)) && got.intersect(&Set::bool()).is_empty() {
            let msg = format!("`{}` is {}, never true or false", c.name, show(m, &got));
            return Err(Error::new(c.line, msg));
        }
    }
    Ok(())
}

/// The name of a function as the model writes it.
fn plain(name: &str) -> &str {
    let name = name.strip_suffix("::body").unwrap_or(name);
    name.strip_suffix("::spec").unwrap_or(name)
}

fn show(m: &Model, set: &Set) -> String {
    let text = m.named(&set.to_string());
    match text.len() > 60 {
        true => "a value of another type".to_string(),
        false => format!("`{text}`"),
    }
}

fn unknown(set: &Set) -> bool {
    Set::any().is_subset(set)
}

fn numbers() -> Set {
    Set::int().union(&Set::float())
}

/// `e` must be able to be a `want`: else it is refused, said as `role`.
fn needs(m: &Model, e: &Expr, got: &Set, want: &Set, role: &str) -> Result<(), String> {
    if got.intersect(want).is_empty() {
        let text = crate::print::expr(e);
        return Err(format!(
            "{role} takes {}, and `{text}` is {}",
            show(m, want),
            show(m, got)
        ));
    }
    Ok(())
}

/// The inputs of a call against what the function takes.
fn call(m: &Model, f: &Function, args: &[Expr], env: &mut Env) -> Result<(), String> {
    for (a, (name, want)) in args.iter().zip(&f.params) {
        let got = of(m, a, env)?;
        let role = format!("`{}`, as `{name}`,", plain(&f.name));
        needs(m, a, &got, want, &role)?;
    }
    Ok(())
}

/// The set of values `e` can be.
pub fn of(m: &Model, e: &Expr, env: &mut Env) -> Result<Set, String> {
    let any = Set::any();
    Ok(match e {
        Expr::Lit(v) => Set::point(v),
        Expr::Var(x) => match env.iter().rev().find(|b| &b.0 == x) {
            Some((_, set)) => set.clone(),
            None => any,
        },
        Expr::Call(f, args) => match m.function(f) {
            Some(g) if g.written.is_none() && g.params.len() == args.len() => {
                call(m, g, args, env)?;
                g.range.clone()
            }
            _ => {
                for a in args {
                    of(m, a, env)?;
                }
                // A value written with its record's name.
                match m.records.iter().find(|r| &r.name == f) {
                    Some(r) => r.set(),
                    None => any,
                }
            }
        },
        Expr::Correct(f, args) => {
            if let Some(g) = m.function(f).filter(|g| g.params.len() == args.len()) {
                call(m, g, args, env)?;
            }
            Set::bool()
        }
        Expr::Unary(op, a) => {
            let got = of(m, a, env)?;
            match *op {
                "!" => {
                    needs(m, a, &got, &Set::bool(), "`!`")?;
                    Set::bool()
                }
                "-" => {
                    needs(m, a, &got, &numbers(), "`-`")?;
                    got.intersect(&numbers())
                }
                _ => any,
            }
        }
        Expr::Binary(op, a, b) => {
            let (x, y) = (of(m, a, env)?, of(m, b, env)?);
            let both = |set: &Set| !x.intersect(set).is_empty() && !y.intersect(set).is_empty();
            match *op {
                "==" | "!=" | "in" => Set::bool(),
                "&&" | "||" => {
                    let role = format!("`{op}`");
                    needs(m, a, &x, &Set::bool(), &role)?;
                    needs(m, b, &y, &Set::bool(), &role)?;
                    Set::bool()
                }
                "<" | "<=" | ">" | ">=" | "+" | "-" | "*" | "/" => {
                    let (ints, floats) = (both(&Set::int()), both(&Set::float()));
                    // A set also takes `-`... no: `\` is its difference.
                    if !ints && !floats {
                        let role = format!("`{op}`");
                        needs(m, a, &x, &numbers(), &role)?;
                        needs(m, b, &y, &numbers(), &role)?;
                        let (l, r) = (crate::print::expr(a), crate::print::expr(b));
                        return Err(format!(
                            "`{op}` takes two integers or two floats, and `{l}` is {}, `{r}` is {}",
                            show(m, &x),
                            show(m, &y)
                        ));
                    }
                    match (*op, ints, floats) {
                        ("<" | "<=" | ">" | ">=", _, _) => Set::bool(),
                        (_, true, false) => Set::int(),
                        (_, false, true) => Set::float(),
                        _ => numbers(),
                    }
                }
                _ => any,
            }
        }
        Expr::If(c, a, b) => {
            let got = of(m, c, env)?;
            needs(m, c, &got, &Set::bool(), "`if`")?;
            of(m, a, env)?.union(&of(m, b, env)?)
        }
        Expr::Let(x, v, body) => {
            let bound = of(m, v, env)?;
            env.push((x.clone(), bound));
            let out = of(m, body, env);
            env.pop();
            out?
        }
        Expr::Tuple(parts) => {
            let mut sets = Vec::new();
            for p in parts {
                sets.push(of(m, p, env)?);
            }
            Set::tuple(sets)
        }
        Expr::Record(name, args) => {
            let record = m.records.iter().find(|r| &r.name == name);
            for (i, a) in args.iter().enumerate() {
                let got = of(m, a, env)?;
                if let Some((field, want)) = record.and_then(|r| r.fields.get(i)) {
                    needs(m, a, &got, want, &format!("`{name}`, as `{field}`,"))?;
                }
            }
            match record {
                Some(r) => r.set(),
                None => any,
            }
        }
        Expr::Field(a, name) => {
            let got = of(m, a, env)?;
            if unknown(&got) || name.parse::<usize>().is_ok() {
                return Ok(any);
            }
            // The records it can be, and those of them with the field.
            let can = |r: &&crate::ast::Record| !got.intersect(&r.set()).is_empty();
            let records: Vec<&crate::ast::Record> = m.records.iter().filter(can).collect();
            let mut out = Set::empty();
            for r in &records {
                if let Some((_, set)) = r.fields.iter().find(|f| &f.0 == name) {
                    out = out.union(set);
                }
                // A record with parts left out may hold it.
                if r.open {
                    out = any.clone();
                }
            }
            let has = |r: &&crate::ast::Record| r.fields.iter().any(|f| &f.0 == name) || r.open;
            if !records.iter().any(|r| has(&r)) {
                let text = crate::print::expr(a);
                return Err(format!(
                    "`{text}` is {}, which has no `.{name}`",
                    show(m, &got)
                ));
            }
            out
        }
        Expr::List(items) => {
            let mut all = Set::empty();
            for i in items {
                all = all.union(&of(m, i, env)?);
            }
            match items.is_empty() {
                true => Set::list(any),
                false => Set::list(all),
            }
        }
        Expr::Index(a, i) => {
            of(m, a, env)?;
            of(m, i, env)?;
            any
        }
        Expr::Len(a) => {
            of(m, a, env)?;
            Set::int()
        }
        Expr::Match(a, arms) => {
            let got = of(m, a, env)?;
            let mut out = Set::empty();
            for arm in arms {
                let n = env.len();
                if let Some((variant, names)) = &arm.variant {
                    let record = m.records.iter().find(|r| &r.name == variant);
                    if let Some(r) = record.filter(|_| !unknown(&got)) {
                        if got.intersect(&r.set()).is_empty() {
                            let text = crate::print::expr(a);
                            return Err(format!(
                                "`{text}` is {}, never a `{variant}`",
                                show(m, &got)
                            ));
                        }
                    }
                    for (i, x) in names.iter().enumerate() {
                        let field = record.and_then(|r| r.fields.get(i));
                        env.push((x.clone(), field.map_or(any.clone(), |f| f.1.clone())));
                    }
                }
                let body = of(m, &arm.body, env);
                env.truncate(n);
                out = out.union(&body?);
            }
            out
        }
        Expr::Rate(x, set, body) => {
            env.push((x.clone(), set.clone()));
            let got = of(m, body, env);
            env.pop();
            needs(m, body, &got?, &Set::bool(), "`rate`")?;
            Set::float()
        }
        Expr::Score(..) => Set::float(),
        Expr::Each(kind, x, list, body) => {
            let over = of(m, list, env)?;
            env.push((x.clone(), any.clone()));
            let got = of(m, body, env);
            env.pop();
            let got = got?;
            match *kind {
                "all" | "any" => {
                    needs(m, body, &got, &Set::bool(), &format!("`{kind}`"))?;
                    Set::bool()
                }
                "filter" => {
                    needs(m, body, &got, &Set::bool(), "`filter`")?;
                    over
                }
                "map" => Set::list(got),
                _ => any,
            }
        }
    })
}
