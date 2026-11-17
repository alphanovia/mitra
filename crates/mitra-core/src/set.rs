//! Sets of values: the types and every subset of inputs Mitra talks about.
//!
//! The universe `any` splits into booleans, integers, strings and everything
//! else. A set is stored per part, in a normal form, so equal sets compare
//! equal.

use std::collections::BTreeSet;
use std::fmt;

/// A single value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::Str(s) => write!(f, "\"{s}\""),
        }
    }
}

const NEG_INF: i128 = i128::MIN;
const POS_INF: i128 = i128::MAX;

/// Strings: a finite set, or every string except a finite set.
#[derive(Debug, Clone, PartialEq)]
enum Strs {
    Finite(BTreeSet<String>),
    Cofinite(BTreeSet<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    /// Membership of `false` and `true`.
    bools: [bool; 2],
    /// Sorted, disjoint, non-adjacent inclusive intervals; `NEG_INF` and
    /// `POS_INF` stand for unbounded ends.
    ints: Vec<(i128, i128)>,
    strs: Strs,
    /// Values that are not booleans, integers or strings.
    other: bool,
}

impl Set {
    pub fn empty() -> Set {
        Set {
            bools: [false; 2],
            ints: vec![],
            strs: Strs::Finite(BTreeSet::new()),
            other: false,
        }
    }
    pub fn bool() -> Set {
        Set {
            bools: [true; 2],
            ..Set::empty()
        }
    }
    pub fn int() -> Set {
        Set {
            ints: vec![(NEG_INF, POS_INF)],
            ..Set::empty()
        }
    }
    pub fn string() -> Set {
        Set {
            strs: Strs::Cofinite(BTreeSet::new()),
            ..Set::empty()
        }
    }
    pub fn any() -> Set {
        Set {
            other: true,
            ..Set::bool().union(&Set::int()).union(&Set::string())
        }
    }
    pub fn point(v: &Value) -> Set {
        match v {
            Value::Bool(b) => {
                let mut s = Set::empty();
                s.bools[*b as usize] = true;
                s
            }
            Value::Int(i) => Set::interval(*i, *i),
            Value::Str(x) => Set {
                strs: Strs::Finite([x.clone()].into()),
                ..Set::empty()
            },
        }
    }
    /// The integers from `lo` to `hi`, both included.
    pub fn interval(lo: i64, hi: i64) -> Set {
        let ints = if lo <= hi {
            vec![(lo as i128, hi as i128)]
        } else {
            vec![]
        };
        Set {
            ints,
            ..Set::empty()
        }
    }

    pub fn union(&self, o: &Set) -> Set {
        let mut ints: Vec<(i128, i128)> = self.ints.iter().chain(&o.ints).copied().collect();
        ints.sort();
        let mut merged: Vec<(i128, i128)> = Vec::new();
        for (lo, hi) in ints {
            match merged.last_mut() {
                Some(last) if last.1 == POS_INF || lo <= last.1 + 1 => last.1 = last.1.max(hi),
                _ => merged.push((lo, hi)),
            }
        }
        use Strs::*;
        let strs = match (&self.strs, &o.strs) {
            (Finite(a), Finite(b)) => Finite(a | b),
            (Cofinite(a), Cofinite(b)) => Cofinite(a & b),
            (Finite(f), Cofinite(c)) | (Cofinite(c), Finite(f)) => Cofinite(c - f),
        };
        Set {
            bools: [self.bools[0] || o.bools[0], self.bools[1] || o.bools[1]],
            ints: merged,
            strs,
            other: self.other || o.other,
        }
    }

    /// Everything in `any` that is not in this set.
    pub fn complement(&self) -> Set {
        let mut ints = Vec::new();
        let mut start = Some(NEG_INF);
        for &(lo, hi) in &self.ints {
            if let Some(s) = start {
                if lo > s {
                    ints.push((s, lo - 1));
                }
            }
            start = if hi == POS_INF { None } else { Some(hi + 1) };
        }
        if let Some(s) = start {
            ints.push((s, POS_INF));
        }
        let strs = match &self.strs {
            Strs::Finite(a) => Strs::Cofinite(a.clone()),
            Strs::Cofinite(a) => Strs::Finite(a.clone()),
        };
        Set {
            bools: [!self.bools[0], !self.bools[1]],
            ints,
            strs,
            other: !self.other,
        }
    }

    pub fn intersect(&self, o: &Set) -> Set {
        self.complement().union(&o.complement()).complement()
    }

    pub fn minus(&self, o: &Set) -> Set {
        self.intersect(&o.complement())
    }

    pub fn is_empty(&self) -> bool {
        *self == Set::empty()
    }

    pub fn is_subset(&self, o: &Set) -> bool {
        self.minus(o).is_empty()
    }

    pub fn contains(&self, v: &Value) -> bool {
        Set::point(v).is_subset(self)
    }

    /// Every value, if the set is finite (and small enough to list).
    pub fn values(&self) -> Option<Vec<Value>> {
        if self.count()? > 1_000_000.0 {
            return None;
        }
        let mut out = Vec::new();
        for b in [false, true] {
            if self.bools[b as usize] {
                out.push(Value::Bool(b));
            }
        }
        for &(lo, hi) in &self.ints {
            out.extend((lo..=hi).map(|i| Value::Int(i as i64)));
        }
        if let Strs::Finite(s) = &self.strs {
            out.extend(s.iter().map(|x| Value::Str(x.clone())));
        }
        Some(out)
    }

    /// The value, if the set has exactly one.
    pub fn single(&self) -> Option<Value> {
        match self.values()?.as_slice() {
            [v] => Some(v.clone()),
            _ => None,
        }
    }

    /// The number of values, if finite.
    pub fn count(&self) -> Option<f64> {
        let unbounded = self
            .ints
            .iter()
            .any(|&(lo, hi)| lo == NEG_INF || hi == POS_INF);
        match &self.strs {
            Strs::Finite(s) if !unbounded && !self.other => {
                let ints: i128 = self.ints.iter().map(|(lo, hi)| hi - lo + 1).sum();
                Some(
                    self.bools.iter().filter(|b| **b).count() as f64 + ints as f64 + s.len() as f64,
                )
            }
            _ => None,
        }
    }
}

impl fmt::Display for Set {
    /// The shortest readable form: `Value`, `Bool`, `{1, "a"}`, `Icc 0 100`,
    /// `Int \ {5}`, joined by ` ∪ `.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.other {
            let rest = self.complement();
            return if rest.is_empty() {
                write!(f, "Value")
            } else {
                write!(f, "Value \\ {}", paren(&rest))
            };
        }
        let mut points: Vec<String> = Vec::new();
        let mut parts: Vec<String> = Vec::new();
        if self.bools == [true; 2] {
            parts.push("Bool".into());
        } else {
            for b in [false, true] {
                if self.bools[b as usize] {
                    points.push(b.to_string());
                }
            }
        }
        let unbounded_both = self.ints.first().is_some_and(|i| i.0 == NEG_INF)
            && self.ints.last().is_some_and(|i| i.1 == POS_INF);
        if unbounded_both {
            let rest = Set::int().minus(self);
            parts.push(if rest.is_empty() {
                "Int".into()
            } else {
                format!("(Int \\ {})", paren(&rest))
            });
        } else {
            for &(lo, hi) in &self.ints {
                if lo == hi {
                    points.push(lo.to_string());
                    continue;
                }
                let b = |x: i128| {
                    if x < 0 {
                        format!("({x})")
                    } else {
                        x.to_string()
                    }
                };
                // Half-unbounded intervals can't be written, only computed.
                parts.push(match (lo, hi) {
                    (NEG_INF, hi) => format!("Iic {}", b(hi)),
                    (lo, POS_INF) => format!("Ici {}", b(lo)),
                    (lo, hi) => format!("Icc {} {}", b(lo), b(hi)),
                });
            }
        }
        match &self.strs {
            Strs::Finite(s) => points.extend(s.iter().map(|x| format!("\"{x}\""))),
            Strs::Cofinite(s) if s.is_empty() => parts.push("String".into()),
            Strs::Cofinite(s) => {
                let s: Vec<String> = s.iter().map(|x| format!("\"{x}\"")).collect();
                parts.push(format!("(String \\ {{{}}})", s.join(", ")));
            }
        }
        if !points.is_empty() {
            parts.insert(0, format!("{{{}}}", points.join(", ")));
        }
        if parts.is_empty() {
            return write!(f, "{{}}");
        }
        if parts.len() == 1 {
            // A lone `(T \ X)` needs no parentheses.
            let p = &parts[0];
            let bare = p.starts_with("(Int") || p.starts_with("(String");
            return write!(f, "{}", if bare { &p[1..p.len() - 1] } else { p });
        }
        write!(f, "{}", parts.join(" ∪ "))
    }
}

/// A set printed so it can follow `\` unambiguously.
fn paren(s: &Set) -> String {
    let t = s.to_string();
    if t.contains(" ∪ ") || t.contains(" \\ ") {
        format!("({t})")
    } else {
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebra() {
        let small = Set::interval(0, 100);
        assert!(small.is_subset(&Set::int()));
        assert_eq!(
            Set::int().union(&Set::bool()).complement(),
            Set::string().union(&Set {
                other: true,
                ..Set::empty()
            })
        );
        assert_eq!(
            small.minus(&Set::point(&Value::Int(5))).count(),
            Some(100.0)
        );
        assert_eq!(
            Set::interval(1, 1).union(&Set::interval(2, 3)),
            Set::interval(1, 3)
        );
        assert_eq!(Set::bool().count(), Some(2.0));
        assert_eq!(Set::string().count(), None);
    }

    #[test]
    fn display() {
        let show = |s: Set| s.to_string();
        assert_eq!(show(Set::any()), "Value");
        assert_eq!(show(Set::empty()), "{}");
        assert_eq!(show(Set::bool()), "Bool");
        assert_eq!(show(Set::point(&Value::Bool(true))), "{true}");
        assert_eq!(
            show(
                Set::interval(0, 4)
                    .union(&Set::interval(6, 100))
                    .union(&Set::string())
            ),
            "Icc 0 4 ∪ Icc 6 100 ∪ String"
        );
        assert_eq!(
            show(Set::point(&Value::Int(1)).union(&Set::point(&Value::Int(42)))),
            "{1, 42}"
        );
        assert_eq!(
            show(Set::int().minus(&Set::point(&Value::Int(5)))),
            "Int \\ {5}"
        );
        assert_eq!(
            show(Set::any().minus(&Set::point(&Value::Int(5)))),
            "Value \\ {5}"
        );
        // Found by the grammar fuzzer: the part after `\` needs parentheses.
        assert_eq!(
            show(Set::any().minus(&Set::interval(-3, 2).union(&Set::string()))),
            "Value \\ (Icc (-3) 2 ∪ String)"
        );
    }
}
