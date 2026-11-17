//! Text projection: a small, fixed syntax inspired by Lean 4. Every command
//! starts with a keyword and layout is free. `--` starts a comment. The
//! grammar is LL(1): one token of lookahead decides every step. See
//! docs/reference/language.md.

use crate::ast::*;
use crate::set::{Set, Value};
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    /// A name or a keyword.
    Ident(String),
    Num(String),
    Str(String),
    Punct(&'static str),
}

/// Longest first, so `:=` wins over `:` and `||` over `|`.
const PUNCT: [&str; 32] = [
    "@[", ":=", "=>", "==", "!=", "<=", ">=", "&&", "||", "(", ")", ":", ";", ",", "=", "{", "}",
    "[", "]", "|", "!", "<", ">", "+", "-", "*", "\\", "∀", "∈", "∪", "∩", "¬",
];

/// Reserved words: never names. Kept in step with the reference grammar.
pub const KEYWORDS: [&str; 25] = [
    "opaque", "def", "claim", "evidence", "profile", "policy", "correct", "at", "by", "derive",
    "if", "then", "else", "let", "true", "false", "Bool", "Int", "String", "Value", "Icc", "for",
    "strict", "coverage", "_",
];

/// The whole text as tokens, each with its line.
fn lex(src: &str) -> Result<Vec<(Tok, usize)>, Error> {
    let mut toks = Vec::new();
    let mut line = 1;
    let mut rest = src;
    while let Some(c) = rest.chars().next() {
        if c == '\n' {
            line += 1;
        }
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if rest.starts_with("--") {
            rest = rest.find('\n').map_or("", |n| &rest[n..]);
        } else if c == '"' {
            let Some(end) = rest[1..]
                .find(['"', '\n'])
                .filter(|&e| rest[1 + e..].starts_with('"'))
            else {
                return Err(Error::new(line, "unterminated string"));
            };
            toks.push((Tok::Str(rest[1..1 + end].into()), line));
            rest = &rest[end + 2..];
        } else if c.is_ascii_digit() {
            let digits = |s: &str| s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            let mut end = digits(rest);
            if let Some(frac) = rest[end..].strip_prefix('.').map(digits).filter(|&f| f > 0) {
                end += 1 + frac;
            }
            toks.push((Tok::Num(rest[..end].into()), line));
            rest = &rest[end..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            toks.push((Tok::Ident(rest[..end].into()), line));
            rest = &rest[end..];
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            toks.push((Tok::Punct(p), line));
            rest = &rest[p.len()..];
        } else {
            return Err(Error::new(line, format!("unexpected character `{c}`")));
        }
    }
    Ok(toks)
}

struct P {
    toks: Vec<(Tok, usize)>,
    pos: usize,
}

impl P {
    fn new(src: &str) -> Result<P, Error> {
        Ok(P {
            toks: lex(src)?,
            pos: 0,
        })
    }
    /// The line of the next token (or of the last one, at the end).
    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or(self.toks.last())
            .map_or(1, |t| t.1)
    }
    fn err<T>(&self, m: impl Into<String>) -> Result<T, Error> {
        Err(Error::new(self.line(), m))
    }
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.0)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.peek().cloned();
        self.pos += 1;
        t
    }
    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn is(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::Punct(q)) if *q == p)
    }
    fn eat(&mut self, p: &str) -> bool {
        let yes = self.is(p);
        self.pos += yes as usize;
        yes
    }
    fn expect(&mut self, p: &str) -> Result<(), Error> {
        if self.eat(p) {
            Ok(())
        } else {
            self.err(format!("expected `{p}`"))
        }
    }
    fn is_name(&self) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()))
    }
    fn ident(&mut self) -> Result<String, Error> {
        match self.next() {
            Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()) => Ok(s),
            Some(Tok::Ident(s)) => {
                self.pos -= 1;
                self.err(format!("`{s}` is a keyword, expected a name"))
            }
            _ => {
                self.pos -= 1;
                self.err("expected a name")
            }
        }
    }
    fn is_keyword(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if s == k)
    }
    fn eat_keyword(&mut self, k: &str) -> bool {
        let yes = self.is_keyword(k);
        self.pos += yes as usize;
        yes
    }
    fn keyword(&mut self, k: &str) -> Result<(), Error> {
        if self.eat_keyword(k) {
            Ok(())
        } else {
            self.err(format!("expected `{k}`"))
        }
    }
    /// A weight or a confidence: `1`, `0.5`.
    fn number(&mut self) -> Result<f64, Error> {
        match self.peek().cloned() {
            Some(Tok::Num(s)) => {
                self.pos += 1;
                match s.parse::<f64>() {
                    Ok(x) if x.is_finite() => Ok(x),
                    _ => self.err(format!("`{s}` is not a number Mitra can use")),
                }
            }
            _ => self.err("expected a number"),
        }
    }
    /// A natural number: digits only.
    fn nat(&mut self) -> Result<i64, Error> {
        self.signed(false)
    }
    /// A natural number, negated when `negative`: down to `i64::MIN`.
    fn signed(&mut self, negative: bool) -> Result<i64, Error> {
        match self.peek().cloned() {
            Some(Tok::Num(s)) if !s.contains('.') => {
                let n: Option<i128> = s.parse().ok();
                let n = n.map(|n| if negative { -n } else { n });
                match n.and_then(|n| i64::try_from(n).ok()) {
                    Some(n) => {
                        self.pos += 1;
                        Ok(n)
                    }
                    None => self.err(format!("`{s}` is outside the 64-bit integers")),
                }
            }
            _ => self.err("expected an integer"),
        }
    }
    /// `5` or `(-5)`: an integer where a single token is expected.
    fn int_atom(&mut self) -> Result<i64, Error> {
        if self.eat("(") {
            self.expect("-")?;
            let n = self.signed(true)?;
            self.expect(")")?;
            return Ok(n);
        }
        self.nat()
    }
    /// value := "true" | "false" | ["-"] nat | string
    fn value(&mut self) -> Result<Value, Error> {
        if self.eat("-") {
            return Ok(Value::Int(self.signed(true)?));
        }
        match self.peek().cloned() {
            Some(Tok::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Tok::Num(_)) => Ok(Value::Int(self.nat()?)),
            Some(Tok::Str(s)) => {
                self.pos += 1;
                Ok(Value::Str(s))
            }
            _ => self.err("expected a value: true, false, an integer or a string"),
        }
    }

    // ---- sets ----

    /// set := inter { "∪" inter }
    fn set(&mut self) -> Result<Set, Error> {
        let mut s = self.set_inter()?;
        while self.eat("∪") {
            s = s.union(&self.set_inter()?);
        }
        Ok(s)
    }
    /// inter := atom { ("∩" | "\") atom }, left to right.
    fn set_inter(&mut self) -> Result<Set, Error> {
        let mut s = self.set_atom()?;
        loop {
            s = if self.eat("∩") {
                s.intersect(&self.set_atom()?)
            } else if self.eat("\\") {
                s.minus(&self.set_atom()?)
            } else {
                return Ok(s);
            };
        }
    }
    fn set_atom(&mut self) -> Result<Set, Error> {
        if self.eat("(") {
            let s = self.set()?;
            self.expect(")")?;
            return Ok(s);
        }
        if self.eat("{") {
            let mut s = Set::empty();
            if self.eat("}") {
                return Ok(s);
            }
            loop {
                s = s.union(&Set::point(&self.value()?));
                if self.eat("}") {
                    return Ok(s);
                }
                self.expect(",")?;
            }
        }
        let set = match self.peek() {
            Some(Tok::Ident(t)) => match t.as_str() {
                "Bool" => Set::bool(),
                "Int" => Set::int(),
                "String" => Set::string(),
                "Value" => Set::any(),
                "Icc" => {
                    self.pos += 1;
                    let lo = self.int_atom()?;
                    let hi = self.int_atom()?;
                    return Ok(Set::interval(lo, hi));
                }
                t => return self.err(format!("unknown type `{t}` (Bool, Int, String, Value)")),
            },
            _ => return self.err("expected a set"),
        };
        self.pos += 1;
        Ok(set)
    }

    // ---- binders, arguments, patterns ----

    /// `(a b : Int) (c : Bool)`, at least one.
    fn binders(&mut self) -> Result<Vec<(String, Set)>, Error> {
        let mut params = Vec::new();
        loop {
            self.expect("(")?;
            let mut names = vec![self.ident()?];
            while self.is_name() {
                names.push(self.ident()?);
            }
            self.expect(":")?;
            let ty = self.set()?;
            self.expect(")")?;
            params.extend(names.into_iter().map(|n| (n, ty.clone())));
            if !self.is("(") {
                return Ok(params);
            }
        }
    }
    /// `∀ x, ∀ y ∈ S, …`: each bound name with its set, if any.
    fn foralls(&mut self) -> Result<Vec<(String, Option<Set>, usize)>, Error> {
        let mut out = Vec::new();
        while self.eat("∀") {
            let line = self.line();
            let x = self.ident()?;
            if out.iter().any(|(y, _, _)| y == &x) {
                return Err(Error::new(line, format!("`{x}` is bound twice")));
            }
            let set = if self.eat("∈") {
                Some(self.set()?)
            } else {
                None
            };
            self.expect(",")?;
            out.push((x, set, line));
        }
        Ok(out)
    }
    /// The arguments of a call in a claim or evidence: bound names and
    /// values, as a pattern. Every bound name is used exactly once.
    fn args(&mut self, bound: &[(String, Option<Set>, usize)]) -> Result<Pattern, Error> {
        let mut pattern = Vec::new();
        loop {
            let pos = if self.is_name() {
                let line = self.line();
                let x = self.ident()?;
                let at = |m: String| Err(Error::new(line, m));
                let Some((_, set, _)) = bound.iter().find(|b| b.0 == x) else {
                    return at(format!("`{x}` is not bound: add `∀ {x},` before it"));
                };
                if pattern
                    .iter()
                    .any(|p: &Position| p.var.as_ref() == Some(&x))
                {
                    return at(format!("`{x}` is used twice: one name per input"));
                }
                Position {
                    var: Some(x),
                    set: set.clone(),
                }
            } else if self.is("(") {
                Position {
                    var: None,
                    set: Some(Set::point(&Value::Int(self.int_atom()?))),
                }
            } else if matches!(self.peek(), Some(Tok::Num(_) | Tok::Str(_)))
                || self.is_keyword("true")
                || self.is_keyword("false")
            {
                Position {
                    var: None,
                    set: Some(Set::point(&self.value()?)),
                }
            } else {
                break;
            };
            pattern.push(pos);
        }
        if let Some((x, _, line)) = bound
            .iter()
            .find(|b| !pattern.iter().any(|p| p.var.as_ref() == Some(&b.0)))
        {
            return Err(Error::new(*line, format!("`{x}` is bound but not used")));
        }
        Ok(pattern)
    }
    /// `name args`: what a judgment or claim is about.
    fn call(&mut self, bound: &[(String, Option<Set>, usize)]) -> Result<(String, Pattern), Error> {
        let f = self.ident()?;
        Ok((f, self.args(bound)?))
    }
    /// `correct (f args)`, after `correct`.
    fn correct(
        &mut self,
        bound: &[(String, Option<Set>, usize)],
    ) -> Result<(String, Pattern), Error> {
        self.expect("(")?;
        let c = self.call(bound)?;
        if c.1.is_empty() {
            return self.err(format!("`correct ({})` needs the inputs", c.0));
        }
        self.expect(")")?;
        Ok(c)
    }

    // ---- expressions ----

    /// expr := "if" expr "then" expr "else" expr | "let" name ":=" expr ";" expr | or
    fn expr(&mut self) -> Result<Expr, Error> {
        if self.eat_keyword("if") {
            let c = self.expr()?;
            self.keyword("then")?;
            let t = self.expr()?;
            self.keyword("else")?;
            let e = self.expr()?;
            return Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)));
        }
        if self.eat_keyword("let") {
            let x = self.ident()?;
            self.expect(":=")?;
            let v = self.expr()?;
            self.expect(";")?;
            let body = self.expr()?;
            return Ok(Expr::Let(x, Box::new(v), Box::new(body)));
        }
        self.binary(0)
    }
    /// Binary operators by level, all left-associative:
    /// `||`, `&&`, comparisons, `+ -`, `*`.
    fn binary(&mut self, level: usize) -> Result<Expr, Error> {
        const LEVELS: [&[&str]; 5] = [
            &["||"],
            &["&&"],
            &["==", "!=", "<", "<=", ">", ">="],
            &["+", "-"],
            &["*"],
        ];
        if level == LEVELS.len() {
            return self.unary();
        }
        let mut e = self.binary(level + 1)?;
        while let Some(op) = LEVELS[level].iter().find(|op| self.is(op)) {
            self.pos += 1;
            e = Expr::Binary(op, Box::new(e), Box::new(self.binary(level + 1)?));
        }
        Ok(e)
    }
    /// unary := ("!" | "-") unary | name { atom } | atom.
    /// `-` on an integer literal gives a negative literal.
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.eat("!") {
            return Ok(Expr::Unary("!", Box::new(self.unary()?)));
        }
        if self.eat("-") {
            // `-` right before digits is a negative literal, down to i64::MIN.
            if matches!(self.peek(), Some(Tok::Num(n)) if !n.contains('.') && n.trim_start_matches('0') != "")
            {
                return Ok(Expr::Lit(Value::Int(self.signed(true)?)));
            }
            return Ok(Expr::Unary("-", Box::new(self.unary()?)));
        }
        if self.is_name() {
            let f = self.ident()?;
            let mut args = Vec::new();
            while self.starts_atom() {
                args.push(self.atom()?);
            }
            return Ok(if args.is_empty() {
                Expr::Var(f)
            } else {
                Expr::Call(f, args)
            });
        }
        self.atom()
    }
    fn starts_atom(&self) -> bool {
        self.is_name()
            || self.is("(")
            || matches!(self.peek(), Some(Tok::Num(_) | Tok::Str(_)))
            || self.is_keyword("true")
            || self.is_keyword("false")
    }
    /// atom := value | name | "(" expr ")"
    fn atom(&mut self) -> Result<Expr, Error> {
        if self.eat("(") {
            let e = self.expr()?;
            self.expect(")")?;
            return Ok(e);
        }
        if self.is_name() {
            return Ok(Expr::Var(self.ident()?));
        }
        if self.is("-") {
            return self.err("expected an expression");
        }
        Ok(Expr::Lit(self.value()?))
    }
}

/// The model file: `opaque`, `def`, `claim`, `profile` and `policy`.
pub fn parse_model(src: &str) -> Result<Model, Error> {
    let mut p = P::new(src)?;
    let mut m = Model::default();
    while !p.at_end() {
        let line = p.line();
        let Some(Tok::Ident(kw)) = p.next() else {
            p.pos -= 1;
            return p.err("expected `opaque`, `def`, `claim`, `profile` or `policy`");
        };
        match kw.as_str() {
            "opaque" | "def" => {
                // opaque f (a : T) … : R   |   def f (a : T) … : R := expr
                let name = p.ident()?;
                let params = p.binders()?;
                p.expect(":")?;
                let range = p.set()?;
                let body = if kw == "def" {
                    p.expect(":=")?;
                    Some(p.expr()?)
                } else {
                    None
                };
                m.functions.push(Function {
                    name,
                    params,
                    range,
                    body,
                    line,
                });
            }
            "claim" => {
                // claim n : ∀ …, correct (f args) | f args = expr  [:= by derive [c, …]]
                let name = p.ident()?;
                p.expect(":")?;
                let bound = p.foralls()?;
                let (func, pattern, target) = if p.eat_keyword("correct") {
                    let (f, pat) = p.correct(&bound)?;
                    (f, pat, Target::Correct)
                } else {
                    let (f, pat) = p.call(&bound)?;
                    if pat.is_empty() {
                        return p.err(format!("`{f}` needs its inputs"));
                    }
                    p.expect("=")?;
                    (f, pat, Target::Matches(p.expr()?))
                };
                let mut from = Vec::new();
                if p.eat(":=") {
                    p.keyword("by")?;
                    p.keyword("derive")?;
                    p.expect("[")?;
                    from.push(p.ident()?);
                    while p.eat(",") {
                        from.push(p.ident()?);
                    }
                    p.expect("]")?;
                }
                m.claims.push(Claim {
                    name,
                    func,
                    pattern,
                    target,
                    from,
                    line,
                });
            }
            "profile" => {
                // profile n for f   then   | x ∈ S => w   |   | _ => w
                let name = p.ident()?;
                p.keyword("for")?;
                let subject = p.ident()?;
                let mut regions = Vec::new();
                while p.eat("|") {
                    let region = if p.eat_keyword("_") {
                        None
                    } else {
                        p.ident()?;
                        p.expect("∈")?;
                        Some(p.set()?)
                    };
                    p.expect("=>")?;
                    regions.push((region, p.number()?));
                }
                m.profiles.push(Profile {
                    name,
                    subject,
                    regions,
                    line,
                });
            }
            "policy" => {
                // policy strict   |   policy n := strict | coverage profile
                if p.eat_keyword("strict") {
                    m.policies.push(Policy {
                        name: "strict".into(),
                        kind: PolicyKind::Strict,
                        line,
                    });
                    continue;
                }
                let name = p.ident()?;
                p.expect(":=")?;
                let kind = if p.eat_keyword("strict") {
                    PolicyKind::Strict
                } else if p.eat_keyword("coverage") {
                    PolicyKind::Coverage(p.ident()?)
                } else {
                    return p.err("expected a policy kind: `strict` or `coverage profile`");
                };
                m.policies.push(Policy { name, kind, line });
            }
            k => {
                p.pos -= 1;
                return p.err(format!(
                    "expected `opaque`, `def`, `claim`, `profile` or `policy`, got `{k}`"
                ));
            }
        }
    }
    Ok(m)
}

/// The evidence file: `evidence ∀ …, <judgment> at <confidence>`, where the
/// judgment is `correct (f args)`, `¬ correct (f args)`, `f args = value`,
/// `claim` or `¬ claim`.
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    let mut p = P::new(src)?;
    let mut out = Vec::new();
    while !p.at_end() {
        let line = p.line();
        let (mut source, mut rev) = (None, None);
        if p.eat("@[") {
            loop {
                let attr_line = p.line();
                let name = p.ident()?;
                let value = match p.next() {
                    Some(Tok::Str(s)) => s,
                    _ => {
                        p.pos -= 1;
                        return p.err(format!("`{name}` needs a string"));
                    }
                };
                let slot = match name.as_str() {
                    "source" => &mut source,
                    "rev" => &mut rev,
                    _ => {
                        return Err(Error::new(
                            attr_line,
                            format!("unknown attribute `{name}` (source, rev)"),
                        ))
                    }
                };
                if slot.replace(value).is_some() {
                    return Err(Error::new(attr_line, format!("`{name}` is given twice")));
                }
                if !p.eat(",") {
                    break;
                }
            }
            p.expect("]")?;
        }
        p.keyword("evidence")?;
        let bound = p.foralls()?;
        let (about, pattern, judgment) = if p.eat_keyword("correct") {
            let (f, pat) = p.correct(&bound)?;
            (f, pat, Judgment::Correct(true))
        } else if p.eat("¬") {
            if p.eat_keyword("correct") {
                let (f, pat) = p.correct(&bound)?;
                (f, pat, Judgment::Correct(false))
            } else {
                let (c, pat) = p.call(&bound)?;
                (c, pat, Judgment::Holds(false))
            }
        } else {
            let (f, pat) = p.call(&bound)?;
            if p.eat("=") {
                (f, pat, Judgment::Output(p.value()?))
            } else {
                (f, pat, Judgment::Holds(true))
            }
        };
        p.keyword("at")?;
        let confidence = p.number()?;
        if confidence != 0.0 && confidence != 1.0 {
            return Err(Error::new(
                line,
                format!("confidence {confidence}: only 0 and 1 are supported for now"),
            ));
        }
        out.push(Evidence {
            about,
            pattern,
            judgment,
            confidence,
            source,
            rev,
            line,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_model() {
        let m = parse_model(
            "opaque id (x : Value) : Value\n\
             claim id_small : ∀ x ∈ Icc 0 100, correct (id x)  -- a subset\n\
             profile usage for id\n\
             \x20 | x ∈ Icc 0 100 => 0.8\n\
             \x20 | _ => 0.2\n\
             policy strict\n\
             policy by_usage := coverage usage",
        )
        .unwrap();
        assert_eq!(m.functions[0].params, vec![("x".to_string(), Set::any())]);
        assert_eq!(m.claims[0].pattern[0].set, Some(Set::interval(0, 100)));
        assert_eq!(
            m.profiles[0].regions,
            vec![(Some(Set::interval(0, 100)), 0.8), (None, 0.2)]
        );
        assert_eq!(m.policies[1].kind, PolicyKind::Coverage("usage".into()));
        assert_eq!(m.policies[1].line, 7);
    }

    #[test]
    fn parses_specs_and_derived_claims() {
        let m = parse_model(
            "def neg (a : Bool) : Bool := !a\n\
             def dec (x : Int) : Int := if x > 0 then x-1 else let y := -x; y * 2\n\
             claim twice : ∀ a, neg a = neg (neg a) := by derive [c1, c2]",
        )
        .unwrap();
        assert_eq!(
            m.functions[0].body,
            Some(Expr::Unary("!", Box::new(Expr::Var("a".into()))))
        );
        let Some(Expr::If(_, then, _)) = &m.functions[1].body else {
            panic!()
        };
        assert_eq!(
            **then,
            Expr::Binary(
                "-",
                Box::new(Expr::Var("x".into())),
                Box::new(Expr::Lit(Value::Int(1)))
            )
        );
        assert_eq!(m.claims[0].pattern[0].var.as_deref(), Some("a"));
        assert_eq!(m.claims[0].from, vec!["c1".to_string(), "c2".to_string()]);
    }

    #[test]
    fn parses_tuples() {
        let m = parse_model(
            "opaque minus (a b : Int) : Int\n\
             def difference (a b : Int) : Int := a - b\n\
             claim zero : ∀ x ∈ Icc 0 10, minus x 0 = add x (negate 0)",
        )
        .unwrap();
        assert_eq!(m.functions[0].params.len(), 2);
        assert_eq!(m.claims[0].pattern[1].set, Some(Set::point(&Value::Int(0))));
        let Target::Matches(Expr::Call(f, args)) = &m.claims[0].target else {
            panic!()
        };
        assert_eq!((f.as_str(), args.len()), ("add", 2));
        let e = parse_evidence("evidence minus 3 (-1) = 4 at 1").unwrap();
        assert_eq!(e[0].pattern.len(), 2);
        assert_eq!(e[0].pattern[1].set, Some(Set::point(&Value::Int(-1))));
    }

    #[test]
    fn parses_evidence() {
        let e = parse_evidence(
            "evidence ∀ x ∈ {1, -2, \"a\"} ∪ Icc 5 6 \\ {6}, ¬ correct (id x) at 1\n\
             evidence not true = false at 1\n\
             evidence shape at 0\n\
             evidence ¬ shape at 1",
        )
        .unwrap();
        let expected = [
            Value::Int(1),
            Value::Int(-2),
            Value::Str("a".into()),
            Value::Int(5),
        ]
        .iter()
        .fold(Set::empty(), |s, v| s.union(&Set::point(v)));
        assert_eq!(e[0].pattern[0].set, Some(expected));
        assert_eq!(e[0].judgment, Judgment::Correct(false));
        assert_eq!(e[1].judgment, Judgment::Output(Value::Bool(false)));
        assert_eq!(
            (e[2].judgment.clone(), e[2].pattern.len()),
            (Judgment::Holds(true), 0)
        );
        assert_eq!(e[3].judgment, Judgment::Holds(false));
    }

    #[test]
    fn set_precedence_is_lean_like() {
        // `∩` and `\` bind tighter than `∪`.
        let e = parse_evidence("evidence ∀ x ∈ {1} ∪ {3, 5} \\ {5}, correct (f x) at 1").unwrap();
        let s = e[0].pattern[0].set.clone().unwrap();
        assert_eq!(s.to_string(), "{1, 3}");
    }

    #[test]
    fn rejects_unbound_and_unused_names() {
        let e = parse_evidence("evidence correct (f x) at 1").unwrap_err();
        assert!(e.message.contains("not bound"), "{e}");
        let e = parse_evidence("evidence ∀ x, ∀ y, correct (f x) at 1").unwrap_err();
        assert!(e.message.contains("`y` is bound but not used"), "{e}");
    }

    #[test]
    fn reads_the_64_bit_extremes() {
        let e = parse_evidence(
            "evidence f (-9223372036854775808) = -9223372036854775808 at 1\n\
             evidence ∀ x ∈ Icc (-9223372036854775808) 9223372036854775807, correct (f x) at 1",
        )
        .unwrap();
        assert_eq!(e[0].judgment, Judgment::Output(Value::Int(i64::MIN)));
        let m = parse_model("def f (a : Int) : Int := -9223372036854775808").unwrap();
        assert_eq!(m.functions[0].body, Some(Expr::Lit(Value::Int(i64::MIN))));
        assert!(parse_evidence("evidence f 9223372036854775808 = 1 at 1").is_err());
        let w = format!("profile p for f | _ => 1{}", "0".repeat(400));
        assert!(parse_model(&w)
            .unwrap_err()
            .message
            .contains("not a number"));
    }

    #[test]
    fn errors_point_at_the_name() {
        let e = parse_evidence("evidence correct (f\n x\n) at 1").unwrap_err();
        assert_eq!(e.line, 2);
        let e = parse_evidence("evidence ∀ x,\n ∀ x,\n correct (f x) at 1").unwrap_err();
        assert_eq!((e.line, e.message.contains("twice")), (2, true));
    }

    #[test]
    fn reads_attributes() {
        let e = parse_evidence(
            "@[source \"ci\", rev \"3f2a\"] evidence correct (not true) at 1\n\
             evidence not true = false at 1",
        )
        .unwrap();
        assert_eq!(
            (e[0].source.as_deref(), e[0].rev.as_deref()),
            (Some("ci"), Some("3f2a"))
        );
        assert_eq!((e[1].source.clone(), e[1].rev.clone()), (None, None));
        let err = parse_evidence("@[who \"me\"] evidence s at 1").unwrap_err();
        assert!(err.message.contains("unknown attribute"), "{err}");
        let err = parse_evidence("@[rev \"a\", rev \"b\"] evidence s at 1").unwrap_err();
        assert!(err.message.contains("twice"), "{err}");
    }

    #[test]
    fn rejects_intermediate_confidence_for_now() {
        let e = parse_evidence("evidence correct (not true) at 0.9").unwrap_err();
        assert!(e.message.contains("only 0 and 1"), "{e}");
    }
}
