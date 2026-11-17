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

const PUNCT: [&str; 14] = [
    "->", "(", ")", ":", ",", "@", "=", "{", "}", "[", "]", "|", "&", "\\",
];

fn lex(line: &str, n: usize) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        let negative = rest
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
    fn eat(&mut self, p: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Punct(q)) if *q == p) {
            self.pos += 1;
            true
        } else {
            false
        }
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
            Some(Tok::Ident(s)) => Ok(s),
            _ => self.err("expected a name"),
        }
    }
    fn is_keyword(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if s == k)
    }
    fn keyword(&mut self, k: &str) -> Result<(), Error> {
        if self.is_keyword(k) {
            self.pos += 1;
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
    fn is_literal(&self) -> bool {
        match self.peek() {
            Some(Tok::Ident(s)) => s == "true" || s == "false",
            Some(Tok::Num(_)) | Some(Tok::Str(_)) => true,
            _ => false,
        }
    }
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
        match self.ident()?.as_str() {
            "bool" => Ok(Set::bool()),
            "int" => Ok(Set::int()),
            "string" => Ok(Set::string()),
            "any" => Ok(Set::any()),
            t => self.err(format!("unknown type `{t}` (bool, int, string, any)")),
        }
    }
    /// `( _ )`, `( value )` or `( x in set )`.
    fn pattern(&mut self) -> Result<Pattern, Error> {
        self.expect("(")?;
        let p = if self.is_keyword("_") {
            self.pos += 1;
            Pattern::Any
        } else if self.is_literal() {
            Pattern::In(Set::point(&self.literal()?))
        } else {
            self.ident()?;
            self.keyword("in")?;
            Pattern::In(self.set()?)
        };
        self.expect(")")?;
        Ok(p)
    }
}

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

/// The model file: `subject`, `claim`, `profile` and `policy` statements.
pub fn parse_model(src: &str) -> Result<Model, Error> {
    let mut m = Model::default();
    let mut in_profile = false;
    for p in lines(src) {
        let mut p = p?;
        if p.indented && in_profile {
            // x in <set> weight w   |   otherwise weight w
            let region = if p.is_keyword("otherwise") {
                p.pos += 1;
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
        match p.ident()?.as_str() {
            "subject" => {
                // subject name(x: set) -> set
                let name = p.ident()?;
                p.expect("(")?;
                let param = p.ident()?;
                p.expect(":")?;
                let domain = p.set()?;
                if !p.eat(")") {
                    return p.err("a subject takes exactly one parameter for now");
                }
                p.expect("->")?;
                p.set()?;
                p.end()?;
                m.subjects.push(Subject {
                    name,
                    param,
                    domain,
                    line: p.line,
                });
            }
            "claim" => {
                // claim name: subject(pattern) correct
                let name = p.ident()?;
                p.expect(":")?;
                let subject = p.ident()?;
                let pattern = p.pattern()?;
                p.keyword("correct")?;
                p.end()?;
                m.claims.push(Claim {
                    name,
                    subject,
                    pattern,
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
                let name = p.ident()?;
                let kind = if p.eat("=") {
                    match p.ident()?.as_str() {
                        "strict" => PolicyKind::Strict,
                        "coverage" => {
                            p.expect("(")?;
                            let profile = p.ident()?;
                            p.expect(")")?;
                            PolicyKind::Coverage(profile)
                        }
                        k => return p.err(format!("unknown policy kind `{k}` (strict, coverage)")),
                    }
                } else if name == "strict" {
                    PolicyKind::Strict
                } else {
                    return p.err("expected `=` and a policy kind (strict, coverage)");
                };
                p.end()?;
                m.policies.push(Policy {
                    name,
                    kind,
                    line: p.line,
                });
            }
            k => {
                return p.err(format!(
                    "expected `subject`, `claim`, `profile` or `policy`, got `{k}`"
                ))
            }
        }
    }
    Ok(m)
}

/// The evidence file: `evidence subject(pattern) correct|wrong @ confidence`.
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    lines(src)
        .map(|p| {
            let mut p = p?;
            p.keyword("evidence")?;
            let subject = p.ident()?;
            let pattern = p.pattern()?;
            let correct = match p.ident()?.as_str() {
                "correct" => true,
                "wrong" => false,
                o => return p.err(format!("expected `correct` or `wrong`, got `{o}`")),
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
                subject,
                pattern,
                correct,
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
        assert_eq!(m.subjects[0].domain, Set::any());
        assert_eq!(m.claims[0].pattern, Pattern::In(Set::interval(0, 100)));
        assert_eq!(
            m.profiles[0].regions,
            vec![(Some(Set::interval(0, 100)), 0.8), (None, 0.2)]
        );
        assert_eq!(m.policies[1].kind, PolicyKind::Coverage("usage".into()));
    }

    #[test]
    fn parses_set_expressions() {
        let e =
            parse_evidence(r#"evidence id(x in {1, -2, "a"} | [5, 6] \ {6}) wrong @ 1"#).unwrap();
        let expected = [
            Value::Int(1),
            Value::Int(-2),
            Value::Str("a".into()),
            Value::Int(5),
        ]
        .iter()
        .fold(Set::empty(), |s, v| s.union(&Set::point(v)));
        assert_eq!(e[0].pattern, Pattern::In(expected));
        assert!(!e[0].correct);
    }

    #[test]
    fn rejects_intermediate_confidence_for_now() {
        let e = parse_evidence("evidence not(true) correct @ 0.9").unwrap_err();
        assert!(e.message.contains("only 0 and 1"), "{e}");
    }
}
