//! Text projection: one declaration per line. `#` starts a comment; blank
//! lines are ignored.
//!
//! ```text
//! subject  not(a: bool) -> bool
//! claim    not_correct: not(_)    correct
//! claim    not_on_true: not(true) correct
//!
//! evidence not(true)  correct @ 1
//! evidence not(false) wrong   @ 0
//! ```

use crate::ast::*;
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(String),
    Punct(&'static str),
}

const PUNCT: [&str; 6] = ["->", "(", ")", ":", ",", "@"];

fn lex(line: &str, n: usize) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    let mut rest = line;
    while let Some(c) = rest.chars().next() {
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if c == '#' {
            break;
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            toks.push(Tok::Punct(p));
            rest = &rest[p.len()..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            toks.push(Tok::Ident(rest[..end].into()));
            rest = &rest[end..];
        } else if c.is_ascii_digit() {
            let end = rest
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(rest.len());
            toks.push(Tok::Num(rest[..end].into()));
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
    fn keyword(&mut self, k: &str) -> Result<(), Error> {
        match self.next() {
            Some(Tok::Ident(s)) if s == k => Ok(()),
            _ => self.err(format!("expected `{k}`")),
        }
    }
    fn is_keyword(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if s == k)
    }
    fn end(&self) -> Result<(), Error> {
        if self.pos < self.toks.len() {
            self.err("unexpected text at end of line")
        } else {
            Ok(())
        }
    }
    /// `( item, item, ... )`, possibly empty.
    fn list<T>(
        &mut self,
        mut item: impl FnMut(&mut P) -> Result<T, Error>,
    ) -> Result<Vec<T>, Error> {
        self.expect("(")?;
        let mut out = Vec::new();
        if self.eat(")") {
            return Ok(out);
        }
        loop {
            out.push(item(self)?);
            if self.eat(")") {
                return Ok(out);
            }
            self.expect(",")?;
        }
    }
    /// `name: bool`
    fn typed(&mut self) -> Result<String, Error> {
        let name = self.ident()?;
        self.expect(":")?;
        self.keyword("bool")?;
        Ok(name)
    }
    fn bool(&mut self) -> Result<bool, Error> {
        match self.next() {
            Some(Tok::Ident(s)) if s == "true" => Ok(true),
            Some(Tok::Ident(s)) if s == "false" => Ok(false),
            _ => self.err("expected `true` or `false`"),
        }
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
            })),
            Err(e) => Some(Err(e)),
        })
}

/// The model file: `subject` and `claim` lines.
pub fn parse_model(src: &str) -> Result<(Vec<Subject>, Vec<Claim>), Error> {
    let (mut subjects, mut claims) = (Vec::new(), Vec::new());
    for p in lines(src) {
        let mut p = p?;
        if p.is_keyword("subject") {
            // subject name(a: bool, ...) -> bool
            p.next();
            let name = p.ident()?;
            let params = p.list(|p| p.typed())?;
            p.expect("->")?;
            p.keyword("bool")?;
            p.end()?;
            subjects.push(Subject {
                name,
                params,
                line: p.line,
            });
        } else {
            // claim name: subject(_ | true | false, ...) correct
            p.keyword("claim")?;
            let name = p.ident()?;
            p.expect(":")?;
            let subject = p.ident()?;
            let pattern = p.list(|p| {
                if p.is_keyword("_") {
                    p.next();
                    Ok(Pattern::Any)
                } else {
                    Ok(Pattern::Value(p.bool()?))
                }
            })?;
            p.keyword("correct")?;
            p.end()?;
            claims.push(Claim {
                name,
                subject,
                pattern,
                line: p.line,
            });
        }
    }
    Ok((subjects, claims))
}

/// The evidence file: `evidence subject(inputs) correct|wrong @ confidence`.
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    lines(src)
        .map(|p| {
            let mut p = p?;
            p.keyword("evidence")?;
            let subject = p.ident()?;
            let inputs = p.list(|p| p.bool())?;
            let correct = match p.ident()?.as_str() {
                "correct" => true,
                "wrong" => false,
                o => return p.err(format!("expected `correct` or `wrong`, got `{o}`")),
            };
            p.expect("@")?;
            let confidence = match p.next() {
                Some(Tok::Num(s)) => match s.parse::<f64>() {
                    Ok(c) if c == 0.0 || c == 1.0 => c,
                    Ok(c) if (0.0..=1.0).contains(&c) => {
                        return p.err(format!(
                            "confidence {s}: only 0 and 1 are supported for now"
                        ))
                    }
                    _ => return p.err(format!("confidence `{s}` must be a number from 0 to 1")),
                },
                _ => return p.err("expected a confidence from 0 to 1 after `@`"),
            };
            p.end()?;
            Ok(Evidence {
                subject,
                inputs,
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
    fn parses_the_seed_model() {
        let (s, c) = parse_model(
            "subject  not(a: bool) -> bool\n\
             claim    not_correct: not(_)    correct  # every input\n\
             claim    not_on_true: not(true) correct",
        )
        .unwrap();
        assert_eq!(s[0].params, vec!["a".to_string()]);
        assert_eq!(c[0].pattern, vec![Pattern::Any]);
        assert_eq!(c[0].inputs(), vec![vec![false], vec![true]]);
        assert_eq!(c[1].pattern, vec![Pattern::Value(true)]);
    }

    #[test]
    fn parses_evidence() {
        let e = parse_evidence("\nevidence not(false) wrong @ 1.0").unwrap();
        assert_eq!(e[0].inputs, vec![false]);
        assert!(!e[0].correct);
        assert_eq!(e[0].confidence, 1.0);
        assert_eq!(e[0].line, 2);
    }

    #[test]
    fn rejects_intermediate_confidence_for_now() {
        let e = parse_evidence("evidence not(true) correct @ 0.9").unwrap_err();
        assert!(e.message.contains("only 0 and 1"), "{e}");
    }
}
