//! Text projection: one claim or one piece of evidence per line.
//! `#` starts a comment; blank lines are ignored.
//!
//! ```text
//! claim not_correct(a: bool): not(a) = !a
//! evidence test(t1) : not_correct(true) = pass @ 0.9
//! ```

use crate::ast::*;
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    Num(String),
    Punct(char),
}

fn lex(line: &str, n: usize) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '#' {
            break;
        } else if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while let Some(&c) = chars
                .peek()
                .filter(|c| c.is_ascii_alphanumeric() || **c == '_')
            {
                s.push(c);
                chars.next();
            }
            toks.push(Tok::Ident(s));
        } else if c.is_ascii_digit() || c == '-' {
            let mut s = String::from(c);
            chars.next();
            while let Some(&c) = chars.peek().filter(|c| c.is_ascii_digit() || **c == '.') {
                s.push(c);
                chars.next();
            }
            toks.push(Tok::Num(s));
        } else if "():,=!@".contains(c) {
            toks.push(Tok::Punct(c));
            chars.next();
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
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(&Tok::Punct(c)) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, c: char) -> Result<(), Error> {
        if self.eat(c) {
            Ok(())
        } else {
            self.err(format!("expected `{c}`"))
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
        self.expect('(')?;
        let mut out = Vec::new();
        if self.eat(')') {
            return Ok(out);
        }
        loop {
            out.push(item(self)?);
            if self.eat(')') {
                return Ok(out);
            }
            self.expect(',')?;
        }
    }
    fn value(&mut self) -> Result<Value, Error> {
        match self.next() {
            Some(Tok::Ident(s)) if s == "true" => Ok(Value::Bool(true)),
            Some(Tok::Ident(s)) if s == "false" => Ok(Value::Bool(false)),
            Some(Tok::Num(s)) => match s.parse() {
                Ok(i) => Ok(Value::Int(i)),
                Err(_) => self.err(format!("`{s}` is not an integer")),
            },
            _ => self.err("expected a value (true, false or an integer)"),
        }
    }
    // expr := unary ('=' unary)?
    fn expr(&mut self) -> Result<Expr, Error> {
        let lhs = self.unary()?;
        if self.eat('=') {
            let rhs = self.unary()?;
            return Ok(Expr::Eq(Box::new(lhs), Box::new(rhs)));
        }
        Ok(lhs)
    }
    // unary := '!' unary | '(' expr ')' | name | name(args) | value
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.eat('!') {
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        if self.eat('(') {
            let e = self.expr()?;
            self.expect(')')?;
            return Ok(e);
        }
        match self.peek() {
            Some(Tok::Ident(s)) if s != "true" && s != "false" => {
                let name = self.ident()?;
                if self.peek() == Some(&Tok::Punct('(')) {
                    Ok(Expr::Call(name, self.list(|p| p.expr())?))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            _ => Ok(Expr::Lit(self.value()?)),
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

/// `claim name(a: bool, b): body`
pub fn parse_claims(src: &str) -> Result<Vec<Claim>, Error> {
    lines(src)
        .map(|p| {
            let mut p = p?;
            p.keyword("claim")?;
            let name = p.ident()?;
            let params = p.list(|p| {
                let name = p.ident()?;
                let domain = if p.eat(':') {
                    match p.ident()?.as_str() {
                        "bool" => Domain::Bool,
                        t => return p.err(format!("unknown domain `{t}` (only `bool` for now)")),
                    }
                } else {
                    Domain::Any
                };
                Ok(Param { name, domain })
            })?;
            p.expect(':')?;
            let body = p.expr()?;
            p.end()?;
            Ok(Claim {
                name,
                params,
                body,
                line: p.line,
            })
        })
        .collect()
}

/// `evidence kind(source) : claim(inputs) = pass|fail @ confidence`
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    lines(src)
        .map(|p| {
            let mut p = p?;
            p.keyword("evidence")?;
            let k = p.ident()?;
            let Some(kind) = Kind::from_name(&k) else {
                return p.err(format!(
                    "unknown evidence kind `{k}` (test, proof, usage, review, agent)"
                ));
            };
            p.expect('(')?;
            let source = p.ident()?;
            p.expect(')')?;
            p.expect(':')?;
            let claim = p.ident()?;
            let inputs = p.list(|p| p.value())?;
            p.expect('=')?;
            let outcome = match p.ident()?.as_str() {
                "pass" => Outcome::Pass,
                "fail" => Outcome::Fail,
                o => return p.err(format!("expected `pass` or `fail`, got `{o}`")),
            };
            p.expect('@')?;
            let confidence = match p.next() {
                Some(Tok::Num(s)) => match s.parse::<f64>() {
                    Ok(c) if (0.0..=1.0).contains(&c) => c,
                    _ => return p.err(format!("confidence `{s}` must be a number from 0 to 1")),
                },
                _ => return p.err("expected a confidence from 0 to 1 after `@`"),
            };
            p.end()?;
            Ok(Evidence {
                kind,
                source,
                claim,
                inputs,
                outcome,
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
    fn parses_the_seed_claim() {
        let c = parse_claims("claim not_correct(a: bool): not(a) = !a  # seed").unwrap();
        assert_eq!(
            c[0].params,
            vec![Param {
                name: "a".into(),
                domain: Domain::Bool
            }]
        );
        assert_eq!(
            c[0].body,
            Expr::Eq(
                Box::new(Expr::Call("not".into(), vec![Expr::Var("a".into())])),
                Box::new(Expr::Not(Box::new(Expr::Var("a".into()))))
            )
        );
    }

    #[test]
    fn parses_evidence() {
        let e =
            parse_evidence("\nevidence usage(cli_session_42) : not_correct(false) = fail @ 0.3")
                .unwrap();
        assert_eq!(e[0].kind, Kind::Usage);
        assert_eq!(e[0].inputs, vec![Value::Bool(false)]);
        assert_eq!(e[0].outcome, Outcome::Fail);
        assert_eq!(e[0].confidence, 0.3);
        assert_eq!(e[0].line, 2);
    }

    #[test]
    fn rejects_bad_confidence() {
        let e = parse_evidence("evidence test(t) : c(true) = pass @ 1.5").unwrap_err();
        assert!(e.message.contains("from 0 to 1"), "{e}");
    }
}
