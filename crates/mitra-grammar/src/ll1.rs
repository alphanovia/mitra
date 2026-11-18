//! An LL(1) check of the left-factored grammar in `mitra.ll1`, and a
//! predictive recognizer driven by it.
//!
//! The grammar is LL(1) when, for every rule, the alternatives start with
//! different tokens (FIRST sets are disjoint) and at most one can be empty,
//! in which case the others don't start with a token that can follow the
//! rule (FIRST ∩ FOLLOW is empty).

use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE: &str = include_str!("mitra.ll1");

/// End of input.
const END: &str = "$";
const CLASSES: [&str; 4] = ["NAT", "DEC", "STR", "NAME"];
const KEYWORDS: [&str; 38] = [
    "opaque", "def", "claim", "test", "evidence", "profile", "policy", "correct", "at", "by",
    "derive", "if", "then", "else", "let", "true", "false", "forall", "in", "not", "Bool", "Int",
    "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "Float64", "String", "Value",
    "for", "strict", "coverage", "_", "Real", "uniform",
];
const PUNCT: [&str; 35] = [
    "@[", "..=", ":=", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "(", ")", ":", ";", ",",
    "=", "{", "}", "[", "]", "|", "&", "!", "<", ">", "+", "-", "*", "\\", "∀", "∈", "∪", "∩", "¬",
];

/// A grammar symbol: a token (`"x"` or a token class) or a rule name.
type Sym = String;

pub struct Grammar {
    rules: BTreeMap<String, Vec<Vec<Sym>>>,
    nullable: BTreeSet<String>,
    first: BTreeMap<String, BTreeSet<String>>,
    follow: BTreeMap<String, BTreeSet<String>>,
}

fn is_token(s: &str) -> bool {
    s.starts_with('"') || CLASSES.contains(&s)
}

impl Grammar {
    pub fn load() -> Grammar {
        Grammar::parse(SOURCE)
    }

    pub fn parse(src: &str) -> Grammar {
        let text: String = src
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join(" ");
        let mut rules: BTreeMap<String, Vec<Vec<Sym>>> = BTreeMap::new();
        let mut words = text.split_whitespace();
        while let Some(name) = words.next() {
            assert_eq!(words.next(), Some("="), "rule {name} without `=`");
            let mut alts = vec![vec![]];
            for w in words.by_ref() {
                match w {
                    ";" => break,
                    "|" => alts.push(vec![]),
                    _ => alts.last_mut().unwrap().push(w.to_string()),
                }
            }
            rules.insert(name.to_string(), alts);
        }
        let mut g = Grammar {
            rules,
            nullable: BTreeSet::new(),
            first: BTreeMap::new(),
            follow: BTreeMap::new(),
        };
        g.compute();
        g
    }

    fn first_of(&self, seq: &[Sym]) -> (BTreeSet<String>, bool) {
        let mut out = BTreeSet::new();
        for s in seq {
            if is_token(s) {
                out.insert(s.clone());
                return (out, false);
            }
            out.extend(self.first.get(s).into_iter().flatten().cloned());
            if !self.nullable.contains(s) {
                return (out, false);
            }
        }
        (out, true)
    }

    fn compute(&mut self) {
        for name in self.rules.keys() {
            self.first.insert(name.clone(), BTreeSet::new());
            self.follow.insert(name.clone(), BTreeSet::new());
        }
        for start in ["Model", "Evidences"] {
            self.follow.get_mut(start).unwrap().insert(END.into());
        }
        let rules = self.rules.clone();
        loop {
            let mut changed = false;
            for (name, alts) in &rules {
                for alt in alts {
                    let (f, empty) = self.first_of(alt);
                    let before = self.first[name].len();
                    self.first.get_mut(name).unwrap().extend(f);
                    changed |= self.first[name].len() != before;
                    if empty {
                        changed |= self.nullable.insert(name.clone());
                    }
                    for (i, s) in alt.iter().enumerate() {
                        if is_token(s) {
                            continue;
                        }
                        let (mut f, empty) = self.first_of(&alt[i + 1..]);
                        if empty {
                            f.extend(self.follow[name].iter().cloned());
                        }
                        let follow = self.follow.entry(s.clone()).or_default();
                        let before = follow.len();
                        follow.extend(f);
                        changed |= follow.len() != before;
                    }
                }
            }
            if !changed {
                break;
            }
        }
    }

    /// Every LL(1) conflict, as text. Empty means the grammar is LL(1).
    pub fn conflicts(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (name, alts) in &self.rules {
            for s in alts.iter().flatten() {
                if !is_token(s) && !self.rules.contains_key(s) {
                    out.push(format!("{name}: unknown rule {s}"));
                }
            }
            let mut seen: BTreeMap<String, usize> = BTreeMap::new();
            for (i, alt) in alts.iter().enumerate() {
                let (mut f, empty) = self.first_of(alt);
                if empty {
                    f.extend(self.follow[name].iter().cloned());
                }
                for t in f {
                    if let Some(j) = seen.insert(t.clone(), i) {
                        out.push(format!(
                            "{name}: alternatives {j} and {i} both start with {t}"
                        ));
                    }
                }
            }
        }
        out
    }

    /// The alternative of `rule` to take on token `t`.
    fn predict(&self, rule: &str, t: &str) -> Option<&Vec<Sym>> {
        self.rules[rule].iter().find(|alt| {
            let (mut f, empty) = self.first_of(alt);
            if empty {
                f.extend(self.follow[rule].iter().cloned());
            }
            f.contains(t)
        })
    }

    /// Recognize `src` from `start` ("Model" or "Evidences"), top-down.
    pub fn accepts(&self, start: &str, src: &str) -> Result<(), String> {
        let toks = tokens(src)?;
        let mut stack: Vec<Sym> = vec![END.into(), start.into()];
        let mut i = 0;
        while let Some(top) = stack.pop() {
            let (class, text) = &toks[i];
            if top == END || is_token(&top) {
                if &top != class {
                    return Err(format!("expected {top}, got `{text}`"));
                }
                i += 1;
                continue;
            }
            let alt = self
                .predict(&top, class)
                .ok_or_else(|| format!("{top}: unexpected `{text}`"))?;
            stack.extend(alt.iter().rev().cloned());
        }
        Ok(())
    }
}

/// Tokens as (grammar symbol, text), ending with `$`. Same rules as the
/// LALRPOP lexer: free whitespace, `--` comments, `-` always an operator.
fn tokens(src: &str) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut rest = src;
    while let Some(c) = rest.chars().next() {
        let take = |n: usize| (rest[..n].to_string(), &rest[n..]);
        let digits = |s: &str| s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
        let (class, text, next) = if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
            continue;
        } else if rest.starts_with("--") {
            rest = rest.find('\n').map_or("", |n| &rest[n..]);
            continue;
        } else if c == '"' {
            let end = rest[1..].find('"').ok_or("unterminated string")? + 2;
            let (t, r) = take(end);
            ("STR".to_string(), t, r)
        } else if c.is_ascii_digit() {
            let n = digits(rest);
            let frac = rest[n..].strip_prefix('.').map_or(0, digits);
            let (class, len) = if frac > 0 {
                ("DEC", n + 1 + frac)
            } else {
                ("NAT", n)
            };
            let (t, r) = take(len);
            (class.to_string(), t, r)
        } else if c.is_ascii_alphabetic() || c == '_' {
            let n = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            let (t, r) = take(n);
            let class = if KEYWORDS.contains(&t.as_str()) {
                format!("\"{t}\"")
            } else {
                "NAME".into()
            };
            (class, t, r)
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            let (t, r) = take(p.len());
            (format!("\"{p}\""), t, r)
        } else {
            return Err(format!("unexpected character `{c}`"));
        };
        out.push((class, text));
        rest = next;
    }
    out.push((END.into(), "end of input".into()));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_is_ll1() {
        let g = Grammar::load();
        assert_eq!(g.conflicts(), Vec::<String>::new());
    }

    #[test]
    fn a_left_recursive_rule_is_not_ll1() {
        let g = Grammar::parse("Model = Model \"x\" | \"y\" ; Evidences = ;");
        assert!(!g.conflicts().is_empty());
    }
}
