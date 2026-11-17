//! Text projection. One statement per line, each led by a keyword; the
//! regions of a `profile` follow on indented lines. `#` starts a comment and
//! blank lines are ignored. See docs/reference/language.md.

use crate::ast::*;
use crate::set::{Set, Value};
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(String),
    Str(String),
    Punct(&'static str),
}

/// Longest first, so `->` wins over `-` and `||` over `|`.
const PUNCT: [&str; 26] = [
    "->", "==", "!=", "<=", ">=", "&&", "||", "(", ")", ":", ",", "@", "=", "{", "}", "[", "]",
    "|", "&", "\\", "!", "<", ">", "+", "-", "*",
];

fn lex(line: &str, n: usize) -> Result<Vec<Tok>, Error> {
    let mut toks: Vec<Tok> = Vec::new();
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        // `-` before a digit starts a number, unless it follows a value.
        let after_value = matches!(
            toks.last(),
            Some(Tok::Ident(_) | Tok::Num(_) | Tok::Str(_) | Tok::Punct(")" | "]"))
        );
        let negative = !after_value
            && rest
                .strip_prefix('-')
                .is_some_and(|r| r.starts_with(|d: char| d.is_ascii_digit()));
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if c == '#' {
            break;
        } else if c == '"' {
            let Some(end) = rest[1..].find('"') else {
                return Err(Error::new(n, "unterminated string"));
            };
            toks.push(Tok::Str(rest[1..1 + end].into()));
            rest = &rest[end + 2..];
        } else if c.is_ascii_digit() || negative {
            let end = 1 + rest[1..]
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(rest.len() - 1);
            toks.push(Tok::Num(rest[..end].into()));
            rest = &rest[end..];
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            toks.push(Tok::Punct(p));
            rest = &rest[p.len()..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            toks.push(Tok::Ident(rest[..end].into()));
            rest = &rest[end..];
        } else {
            return Err(Error::new(n, format!("unexpected character `{c}`")));
        }
    }
    Ok(toks)
}

struct P {
    toks: Vec<Tok>,
    pos: usize,
    line: usize,
    indented: bool,
}

impl P {
    fn err<T>(&self, m: impl Into<String>) -> Result<T, Error> {
        Err(Error::new(self.line, m))
    }
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
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
    fn ident(&mut self) -> Result<String, Error> {
        match self.next() {
            Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()) => Ok(s),
            Some(Tok::Ident(s)) => self.err(format!("`{s}` is a keyword, expected a name")),
            _ => self.err("expected a name"),
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
    fn end(&self) -> Result<(), Error> {
        if self.pos < self.toks.len() {
            self.err("unexpected text at end of line")
        } else {
            Ok(())
        }
    }
    fn number(&mut self) -> Result<f64, Error> {
        match self.next() {
            Some(Tok::Num(s)) => s
                .parse()
                .or_else(|_| self.err(format!("`{s}` is not a number"))),
            _ => self.err("expected a number"),
        }
    }
    fn int(&mut self) -> Result<i64, Error> {
        match self.next() {
            Some(Tok::Num(s)) => s
                .parse()
                .or_else(|_| self.err(format!("`{s}` is not an integer"))),
            _ => self.err("expected an integer"),
        }
    }
    fn is_literal(&self) -> bool {
        match self.peek() {
            Some(Tok::Ident(s)) => s == "true" || s == "false",
            Some(Tok::Num(_)) | Some(Tok::Str(_)) => true,
            _ => false,
        }
    }
    /// `true`, `false`, an integer or a string.
    fn literal(&mut self) -> Result<Value, Error> {
        match self.peek().cloned() {
            Some(Tok::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Tok::Num(_)) => Ok(Value::Int(self.int()?)),
            Some(Tok::Str(s)) => {
                self.pos += 1;
                Ok(Value::Str(s))
            }
            _ => self.err("expected a value: true, false, an integer or a string"),
        }
    }

    // ---- sets ----

    /// set := atom { ("|" | "&" | "\") atom }, left to right.
    fn set(&mut self) -> Result<Set, Error> {
        let mut s = self.set_atom()?;
        loop {
            s = if self.eat("|") {
                s.union(&self.set_atom()?)
            } else if self.eat("&") {
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
                s = s.union(&Set::point(&self.literal()?));
                if self.eat("}") {
                    return Ok(s);
                }
                self.expect(",")?;
            }
        }
        if self.eat("[") {
            let lo = self.int()?;
            self.expect(",")?;
            let hi = self.int()?;
            self.expect("]")?;
            return Ok(Set::interval(lo, hi));
        }
        match self.next() {
            Some(Tok::Ident(t)) => match t.as_str() {
                "bool" => Ok(Set::bool()),
                "int" => Ok(Set::int()),
                "string" => Ok(Set::string()),
                "any" => Ok(Set::any()),
                t => self.err(format!("unknown type `{t}` (bool, int, string, any)")),
            },
            _ => self.err("expected a set"),
        }
    }

    /// `( _ )`, `( value )`, `( x )` or `( x in set )`.
    fn pattern(&mut self) -> Result<Pattern, Error> {
        self.expect("(")?;
        let p = if self.eat_keyword("_") {
            Pattern {
                var: None,
                set: None,
            }
        } else if self.is_literal() {
            Pattern {
                var: None,
                set: Some(Set::point(&self.literal()?)),
            }
        } else {
            let var = Some(self.ident()?);
            let set = if self.eat_keyword("in") {
                Some(self.set()?)
            } else {
                None
            };
            Pattern { var, set }
        };
        self.expect(")")?;
        Ok(p)
    }

    // ---- expressions ----

    /// expr := "if" expr "then" expr "else" expr | "let" name "=" expr "in" expr | or
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
            self.expect("=")?;
            let v = self.expr()?;
            self.keyword("in")?;
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
    /// unary := ("!" | "-") unary | value | name | name "(" expr ")" | "(" expr ")"
    fn unary(&mut self) -> Result<Expr, Error> {
        for op in ["!", "-"] {
            if self.eat(op) {
                return Ok(Expr::Unary(op, Box::new(self.unary()?)));
            }
        }
        if self.eat("(") {
            let e = self.expr()?;
            self.expect(")")?;
            return Ok(e);
        }
        if self.is_literal() {
            return Ok(Expr::Lit(self.literal()?));
        }
        let name = self.ident()?;
        if self.eat("(") {
            let arg = self.expr()?;
            self.expect(")")?;
            return Ok(Expr::Call(name, Box::new(arg)));
        }
        Ok(Expr::Var(name))
    }
}

const KEYWORDS: [&str; 26] = [
    "subject",
    "spec",
    "claim",
    "evidence",
    "profile",
    "policy",
    "correct",
    "wrong",
    "matches",
    "from",
    "holds",
    "fails",
    "if",
    "then",
    "else",
    "let",
    "in",
    "true",
    "false",
    "bool",
    "int",
    "string",
    "any",
    "otherwise",
    "weight",
    "_",
];

fn lines(src: &str) -> impl Iterator<Item = Result<P, Error>> + '_ {
    src.lines()
        .enumerate()
        .filter_map(|(i, l)| match lex(l, i + 1) {
            Ok(toks) if toks.is_empty() => None,
            Ok(toks) => Some(Ok(P {
                toks,
                pos: 0,
                line: i + 1,
                indented: l.starts_with(char::is_whitespace),
            })),
            Err(e) => Some(Err(e)),
        })
}

/// The model file: `subject`, `spec`, `claim`, `profile` and `policy`.
pub fn parse_model(src: &str) -> Result<Model, Error> {
    let mut m = Model::default();
    let mut in_profile = false;
    for p in lines(src) {
        let mut p = p?;
        if p.indented && in_profile {
            // x in <set> weight w   |   otherwise weight w
            let region = if p.eat_keyword("otherwise") {
                None
            } else {
                p.ident()?;
                p.keyword("in")?;
                Some(p.set()?)
            };
            p.keyword("weight")?;
            let w = p.number()?;
            p.end()?;
            m.profiles.last_mut().unwrap().regions.push((region, w));
            continue;
        }
        in_profile = false;
        let Some(Tok::Ident(kw)) = p.next() else {
            return p.err("expected `subject`, `spec`, `claim`, `profile` or `policy`");
        };
        match kw.as_str() {
            "subject" | "spec" => {
                // subject name(x: set) -> set   |   spec name(x: set) -> set = expr
                let name = p.ident()?;
                p.expect("(")?;
                let param = p.ident()?;
                p.expect(":")?;
                let domain = p.set()?;
                if !p.eat(")") {
                    return p.err("a function takes exactly one parameter for now");
                }
                p.expect("->")?;
                let range = p.set()?;
                let body = if kw == "spec" {
                    p.expect("=")?;
                    Some(p.expr()?)
                } else {
                    None
                };
                p.end()?;
                m.functions.push(Function {
                    name,
                    param,
                    domain,
                    range,
                    body,
                    line: p.line,
                });
            }
            "claim" => {
                // claim name: f(pattern) correct|matches expr [from c1, c2]
                let name = p.ident()?;
                p.expect(":")?;
                let func = p.ident()?;
                let pattern = p.pattern()?;
                let target = if p.eat_keyword("correct") {
                    Target::Correct
                } else if p.eat_keyword("matches") {
                    Target::Matches(p.expr()?)
                } else {
                    return p.err("expected `correct` or `matches`");
                };
                let mut from = Vec::new();
                if p.eat_keyword("from") {
                    loop {
                        from.push(p.ident()?);
                        if !p.eat(",") {
                            break;
                        }
                    }
                }
                p.end()?;
                m.claims.push(Claim {
                    name,
                    func,
                    pattern,
                    target,
                    from,
                    line: p.line,
                });
            }
            "profile" => {
                // profile name for subject:
                let name = p.ident()?;
                p.keyword("for")?;
                let subject = p.ident()?;
                p.expect(":")?;
                p.end()?;
                m.profiles.push(Profile {
                    name,
                    subject,
                    regions: vec![],
                    line: p.line,
                });
                in_profile = true;
            }
            "policy" => {
                // policy strict   |   policy name = strict | coverage(profile)
                let kind = if p.eat_keyword("strict") {
                    m.policies.push(Policy {
                        name: "strict".into(),
                        kind: PolicyKind::Strict,
                        line: p.line,
                    });
                    p.end()?;
                    continue;
                } else {
                    p.ident()?
                };
                p.expect("=")?;
                let k = if p.eat_keyword("strict") {
                    PolicyKind::Strict
                } else if p.eat_keyword("coverage") {
                    p.expect("(")?;
                    let profile = p.ident()?;
                    p.expect(")")?;
                    PolicyKind::Coverage(profile)
                } else {
                    return p.err("expected a policy kind: `strict` or `coverage(profile)`");
                };
                p.end()?;
                m.policies.push(Policy {
                    name: kind,
                    kind: k,
                    line: p.line,
                });
            }
            k => {
                return p.err(format!(
                    "expected `subject`, `spec`, `claim`, `profile` or `policy`, got `{k}`"
                ))
            }
        }
    }
    Ok(m)
}

/// The evidence file:
/// `evidence name(pattern) correct|wrong|holds|fails|-> value @ confidence`.
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    lines(src)
        .map(|p| {
            let mut p = p?;
            p.keyword("evidence")?;
            let about = p.ident()?;
            let pattern = p.pattern()?;
            let judgment = if p.eat("->") {
                Judgment::Output(p.literal()?)
            } else {
                match p.next() {
                    Some(Tok::Ident(s)) if s == "correct" || s == "wrong" => {
                        Judgment::Correct(s == "correct")
                    }
                    Some(Tok::Ident(s)) if s == "holds" || s == "fails" => {
                        Judgment::Holds(s == "holds")
                    }
                    _ => {
                        return p.err("expected `correct`, `wrong`, `holds`, `fails` or `-> value`")
                    }
                }
            };
            p.expect("@")?;
            let confidence = p.number()?;
            if confidence != 0.0 && confidence != 1.0 {
                return p.err(format!(
                    "confidence {confidence}: only 0 and 1 are supported for now"
                ));
            }
            p.end()?;
            Ok(Evidence {
                about,
                pattern,
                judgment,
                confidence,
                line: p.line,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_model() {
        let m = parse_model(
            "subject  id(x: any) -> any\n\
             claim    id_small: id(x in [0, 100]) correct  # a subset\n\
             profile  usage for id:\n\
             \x20 x in [0, 100] weight 0.8\n\
             \x20 otherwise     weight 0.2\n\
             policy   strict\n\
             policy   by_usage = coverage(usage)",
        )
        .unwrap();
        assert_eq!(m.functions[0].domain, Set::any());
        assert_eq!(m.claims[0].pattern.set, Some(Set::interval(0, 100)));
        assert_eq!(
            m.profiles[0].regions,
            vec![(Some(Set::interval(0, 100)), 0.8), (None, 0.2)]
        );
        assert_eq!(m.policies[1].kind, PolicyKind::Coverage("usage".into()));
    }

    #[test]
    fn parses_specs_and_derived_claims() {
        let m = parse_model(
            "spec  neg(a: bool) -> bool = !a\n\
             spec  dec(x: int) -> int = if x > 0 then x-1 else let y = -x in y * 2\n\
             claim twice: neg(a) matches neg(neg(a)) from c1, c2",
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
        assert_eq!(m.claims[0].pattern.var.as_deref(), Some("a"));
        assert_eq!(m.claims[0].from, vec!["c1".to_string(), "c2".to_string()]);
    }

    #[test]
    fn parses_evidence() {
        let e = parse_evidence(
            "evidence id(x in {1, -2, \"a\"} | [5, 6] \\ {6}) wrong @ 1\n\
             evidence not(true) -> false @ 1\n\
             evidence shape(_) holds @ 0",
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
        assert_eq!(e[0].pattern.set, Some(expected));
        assert_eq!(e[1].judgment, Judgment::Output(Value::Bool(false)));
        assert_eq!(e[2].judgment, Judgment::Holds(true));
    }

    #[test]
    fn rejects_intermediate_confidence_for_now() {
        let e = parse_evidence("evidence not(true) correct @ 0.9").unwrap_err();
        assert!(e.message.contains("only 0 and 1"), "{e}");
    }
}
