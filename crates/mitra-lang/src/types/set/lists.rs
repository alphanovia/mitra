//! Sets of lists.

use super::*;

/// A set of lists: those of the terms of `base`, without the lists of
/// `minus`, with the lists of `plus`. Every term holds infinitely many
/// lists; `plus` holds no list of `base`, and `minus` only lists of `base`.
#[derive(Debug, Clone)]
pub(super) struct Lists {
    pub(super) base: Vec<Term>,
    pub(super) plus: BTreeSet<Vec<Value>>,
    pub(super) minus: BTreeSet<Vec<Value>>,
}

impl Lists {
    pub(super) fn none() -> Lists {
        Lists::build(Vec::new(), BTreeSet::new(), |_| false)
    }
    /// The lists of `terms`, corrected on the `points` so that each is in
    /// the set exactly when `member` says so.
    pub(super) fn build(
        terms: Vec<Term>,
        mut points: BTreeSet<Vec<Value>>,
        member: impl Fn(&[Value]) -> bool,
    ) -> Lists {
        let mut base: Vec<Term> = Vec::new();
        for t in terms {
            let some: Vec<Set> = match &t.all {
                Some(all) => t.some.iter().map(|f| f.intersect(all)).collect(),
                None => t.some.clone(),
            };
            if some.iter().any(Set::is_empty) {
                continue;
            }
            if t.all.as_ref().is_some_and(|all| all.is_empty()) {
                // Only the empty list.
                continue;
            }
            let mut kept: Vec<Set> = Vec::new();
            for f in some {
                if !kept.contains(&f) {
                    kept.push(f);
                }
            }
            // The same term twice says nothing more.
            if !base.iter().any(|b| b.all == t.all && b.some == kept) {
                base.push(Term {
                    all: t.all,
                    some: kept,
                });
            }
        }
        // The empty list is where terms differ from what they came from.
        points.insert(Vec::new());
        let (mut plus, mut minus) = (BTreeSet::new(), BTreeSet::new());
        for p in points {
            let have = base.iter().any(|t| t.contains(&p));
            match (member(&p), have) {
                (true, false) => plus.insert(p),
                (false, true) => minus.insert(p),
                _ => false,
            };
        }
        Lists { base, plus, minus }
    }
    pub(super) fn points(&self) -> BTreeSet<Vec<Value>> {
        self.plus.union(&self.minus).cloned().collect()
    }
    pub(super) fn contains(&self, list: &[Value]) -> bool {
        self.plus.contains(list)
            || (!self.minus.contains(list) && self.base.iter().any(|t| t.contains(list)))
    }
    pub(super) fn union(&self, o: &Lists) -> Lists {
        let points = self.points().union(&o.points()).cloned().collect();
        Lists::build([self.base.clone(), o.base.clone()].concat(), points, |p| {
            self.contains(p) || o.contains(p)
        })
    }
    pub(super) fn intersect(&self, o: &Lists) -> Lists {
        let terms = self
            .base
            .iter()
            .flat_map(|a| o.base.iter().map(|b| a.intersect(b)))
            .collect();
        let points = self.points().union(&o.points()).cloned().collect();
        Lists::build(terms, points, |p| self.contains(p) && o.contains(p))
    }
    pub(super) fn complement(&self) -> Lists {
        let mut terms = vec![Term {
            all: None,
            some: vec![],
        }];
        for t in &self.base {
            let outside = t.complement();
            terms = terms
                .iter()
                .flat_map(|a| outside.iter().map(|b| a.intersect(b)))
                .collect();
            // Drop what holds nothing, so the product stays small.
            terms = Lists::build(terms, BTreeSet::new(), |_| false).base;
        }
        Lists::build(terms, self.points(), |p| !self.contains(p))
    }
    pub(super) fn is_empty(&self) -> bool {
        self.base.is_empty() && self.plus.is_empty()
    }

    /// The lists as text: single lists into `points`, the rest into
    /// `parts`. `whole(E)` writes the lists with every element in E, and
    /// `value` one list; sets and maps are printed through their lists.
    pub(super) fn show(
        &self,
        whole: &dyn Fn(&Set) -> String,
        value: &dyn Fn(&Vec<Value>) -> String,
        points: &mut Vec<String>,
        parts: &mut Vec<String>,
    ) {
        points.extend(self.plus.iter().map(value));
        if self.base.is_empty() {
            return;
        }
        // `List(E)`, and for each set F it needs an element of:
        // `\ List(E \ F)`.
        let terms: Vec<String> = self
            .base
            .iter()
            .map(|t| {
                let all = t.all();
                let mut text = whole(&all);
                for f in &t.some {
                    // Needing any element at all: not the empty list.
                    let without = all.minus(f);
                    text += &match without.is_empty() {
                        true => format!(" \\ {{{}}}", value(&Vec::new())),
                        false => format!(" \\ {}", whole(&without)),
                    };
                }
                match t.some.is_empty() {
                    true => text,
                    false => format!("\u{1}{text}"),
                }
            })
            .collect();
        if self.minus.is_empty() {
            parts.extend(terms);
            return;
        }
        let but: Vec<String> = self.minus.iter().map(value).collect();
        let terms: Vec<String> = terms
            .iter()
            .map(|t| match t.strip_prefix('\u{1}') {
                Some(t) => format!("({t})"),
                None => t.clone(),
            })
            .collect();
        let terms = match &terms[..] {
            [one] => one.clone(),
            many => format!("({})", many.join(" | ")),
        };
        parts.push(format!("\u{1}{terms} \\ {{{}}}", but.join(", ")));
    }
}

/// The entries of a map type: `K, V` when they are any key with any
/// value, otherwise the set of `(key, value)` pairs.
pub(super) fn entries(pairs: &Set) -> String {
    let only_pairs = Set {
        composites: pairs.composites.clone(),
        ..Set::empty()
    } == *pairs;
    let product = match pairs.composites.shapes.get(&Shape::Tuple(2)) {
        Some(t) if only_pairs && !pairs.composites.rest && pairs.composites.shapes.len() == 1 => {
            match t.boxes() {
                [b] => Some(format!("{}, {}", b[0], b[1])),
                _ => None,
            }
        }
        _ => None,
    };
    product.unwrap_or_else(|| pairs.to_string())
}

impl PartialEq for Lists {
    fn eq(&self, o: &Lists) -> bool {
        let minus = |a: &Lists, b: &Lists| a.complement().union(b).complement();
        minus(self, o).is_empty() && minus(o, self).is_empty()
    }
}
