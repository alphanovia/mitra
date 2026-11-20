//! Sets of input tuples: what a claim or a piece of evidence is about when a
//! function has several parameters.
//!
//! A set of tuples is a union of boxes; a box is one set per parameter (a
//! cartesian product). With one parameter this is just a [`Set`].

use crate::types::set::{Set, Value};
use std::collections::BTreeSet;

/// The most tuples Mitra enumerates; beyond it a set counts as unbounded.
const MAX_TUPLES: f64 = 1_000_000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Tuples {
    arity: usize,
    /// No box has an empty component.
    boxes: Vec<Vec<Set>>,
}

impl Tuples {
    pub fn empty(arity: usize) -> Tuples {
        Tuples {
            arity,
            boxes: vec![],
        }
    }

    /// The product of one set per parameter.
    pub fn product(sets: Vec<Set>) -> Tuples {
        let arity = sets.len();
        let boxes = if sets.iter().any(Set::is_empty) {
            vec![]
        } else {
            vec![sets]
        };
        Tuples { arity, boxes }
    }

    pub fn point(values: &[Value]) -> Tuples {
        Tuples::product(values.iter().map(Set::point).collect())
    }

    /// The products of sets the set is made of.
    pub fn boxes(&self) -> &[Vec<Set>] {
        &self.boxes
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn union(&self, o: &Tuples) -> Tuples {
        Tuples {
            arity: self.arity,
            boxes: [self.boxes.clone(), o.boxes.clone()].concat(),
        }
    }

    pub fn intersect(&self, o: &Tuples) -> Tuples {
        let mut boxes = Vec::new();
        for a in &self.boxes {
            for b in &o.boxes {
                let meet: Vec<Set> = a.iter().zip(b).map(|(x, y)| x.intersect(y)).collect();
                if !meet.iter().any(Set::is_empty) {
                    boxes.push(meet);
                }
            }
        }
        Tuples {
            arity: self.arity,
            boxes,
        }
    }

    pub fn minus(&self, o: &Tuples) -> Tuples {
        let mut boxes = self.boxes.clone();
        for b in &o.boxes {
            boxes = boxes.iter().flat_map(|a| box_minus(a, b)).collect();
        }
        Tuples {
            arity: self.arity,
            boxes,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    pub fn is_subset(&self, o: &Tuples) -> bool {
        self.minus(o).is_empty()
    }

    /// Same elements, whatever the boxes.
    pub fn same(&self, o: &Tuples) -> bool {
        self.is_subset(o) && o.is_subset(self)
    }

    pub fn contains(&self, t: &[Value]) -> bool {
        let inside = |b: &Vec<Set>| b.iter().zip(t).all(|(s, v)| s.contains(v));
        t.len() == self.arity && self.boxes.iter().any(inside)
    }

    /// With one parameter: the set of values.
    pub fn as_set(&self) -> Set {
        assert_eq!(self.arity, 1);
        self.boxes.iter().fold(Set::empty(), |s, b| s.union(&b[0]))
    }

    /// Every tuple, sorted, if the set is finite and not too large.
    pub fn values(&self) -> Option<Vec<Vec<Value>>> {
        let mut out = BTreeSet::new();
        for b in &self.boxes {
            let size: f64 = b.iter().map(|s| s.count()).product::<Option<f64>>()?;
            if size + out.len() as f64 > MAX_TUPLES {
                return None;
            }
            let mut tuples = vec![vec![]];
            for s in b {
                let vs = s.values()?;
                tuples = tuples
                    .into_iter()
                    .flat_map(|t: Vec<Value>| {
                        vs.iter()
                            .map(move |v| [t.clone(), vec![v.clone()]].concat())
                    })
                    .collect();
            }
            out.extend(tuples);
        }
        Some(out.into_iter().collect())
    }

    /// The sum of the boxes' sizes, without listing them, if every box is
    /// finite. Exact when the boxes are disjoint, as `minus` leaves the
    /// pieces of one box; an upper bound otherwise.
    pub fn size_bound(&self) -> Option<f64> {
        self.boxes
            .iter()
            .map(|b| b.iter().map(Set::count).product::<Option<f64>>())
            .sum()
    }

    pub fn count(&self) -> Option<f64> {
        // One box is counted without listing it: a value nested deep
        // would otherwise be listed again at every level.
        match &self.boxes[..] {
            [] => Some(0.0),
            [_] => self.size_bound(),
            _ => self.values().map(|v| v.len() as f64),
        }
    }

    /// Readable form. One parameter: the set. Otherwise the domain's own
    /// form when equal to it, a list of tuples when finite, or boxes.
    pub fn show(&self, domain: &Tuples) -> String {
        if self.arity == 1 {
            // A type that contains itself has `Value` inside: without a
            // few values it reads `Expr \ {Lit(1)}`, not piece by piece.
            let (set, all) = (self.as_set(), domain.as_set());
            let inside = all.shapes() >= 1 && all.to_string().contains("Any");
            let gone = all.minus(&set).values().filter(|vs| !vs.is_empty());
            return match gone {
                Some(vs) if inside && set.count().is_none() => {
                    let vs: Vec<String> = vs.iter().map(|v| v.to_string()).collect();
                    format!(
                        "{} \\ {{{}}}",
                        crate::types::set::paren(&all),
                        vs.join(", ")
                    )
                }
                _ => set.to_string(),
            };
        }
        if self.is_empty() {
            return "{}".into();
        }
        if self.same(domain) && domain.boxes.len() == 1 {
            return show_box(&domain.boxes[0]);
        }
        if let Some(vs) = self.values() {
            let ts: Vec<String> = vs
                .iter()
                .map(|t| {
                    format!(
                        "({})",
                        t.iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();
            return format!("{{{}}}", ts.join(", "));
        }
        let boxes: Vec<String> = self.boxes.iter().map(|b| show_box(b)).collect();
        boxes.join(" | ")
    }
}

/// A product of sets, as a tuple of sets: `(0..=2, -2..=0)`.
fn show_box(b: &[Set]) -> String {
    let sets: Vec<String> = b.iter().map(|s| s.to_string()).collect();
    format!("({})", sets.join(", "))
}

/// `a \ b` as disjoint boxes: for each position i, the part of `a` that
/// agrees with `b` before i and leaves it at i.
fn box_minus(a: &[Set], b: &[Set]) -> Vec<Vec<Set>> {
    let mut out = Vec::new();
    for i in 0..a.len() {
        let mut piece: Vec<Set> = Vec::with_capacity(a.len());
        for (j, s) in a.iter().enumerate() {
            piece.push(match j.cmp(&i) {
                std::cmp::Ordering::Less => s.intersect(&b[j]),
                std::cmp::Ordering::Equal => s.minus(&b[j]),
                std::cmp::Ordering::Greater => s.clone(),
            });
        }
        if !piece.iter().any(Set::is_empty) {
            out.push(piece);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(i: i64) -> Value {
        Value::Int(i)
    }

    #[test]
    fn algebra_and_display() {
        let d = Tuples::product(vec![Set::interval(0, 2), Set::interval(-2, 0)]);
        assert_eq!(d.count(), Some(9.0));
        let hole = d.minus(&Tuples::point(&[int(1), int(-1)]));
        assert_eq!(hole.count(), Some(8.0));
        assert!(!hole.contains(&[int(1), int(-1)]));
        assert!(hole.union(&Tuples::point(&[int(1), int(-1)])).same(&d));
        assert_eq!(d.show(&d), "(0..=2, -2..=0)");
        assert_eq!(d.minus(&hole).show(&d), "{(1, -1)}");
        let wide = Tuples::product(vec![Set::int(), Set::point(&int(0))]);
        assert_eq!(wide.count(), None);
        assert_eq!(
            wide.show(&Tuples::product(vec![Set::int(), Set::int()])),
            "(Int, {0})"
        );
    }
}
