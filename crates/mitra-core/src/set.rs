//! Sets of values: the types and every subset of inputs Mitra talks about.
//!
//! The universe `Value` splits into booleans, integers, 64-bit floats,
//! strings and everything else. A set is stored per part, in a normal form,
//! so equal sets compare equal.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

/// A single value. Floats compare by their bits' total order: `NaN` equals
/// itself, and `0.0` and `-0.0` differ.
#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    /// An integer. `Int` is every integer, but Mitra computes within 64 bits
    /// for now.
    Int(i64),
    Float(f64),
    Str(String),
}

impl Value {
    fn rank(&self) -> u8 {
        match self {
            Value::Bool(_) => 0,
            Value::Int(_) => 1,
            Value::Float(_) => 2,
            Value::Str(_) => 3,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, o: &Value) -> Ordering {
        match (self, o) {
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.cmp(b),
            _ => self.rank().cmp(&o.rank()),
        }
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, o: &Value) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl PartialEq for Value {
    fn eq(&self, o: &Value) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            // Decimal notation, shortest that reads back the same, and `1.0`
            // rather than `1`: a float stays a float when printed.
            Value::Float(x) => {
                let t = x.to_string();
                if x.is_finite() && !t.contains('.') {
                    write!(f, "{t}.0")
                } else {
                    write!(f, "{t}")
                }
            }
            Value::Str(s) => write!(f, "\"{s}\""),
        }
    }
}

const NEG_INF: i128 = i128::MIN;
const POS_INF: i128 = i128::MAX;

/// A float, ordered by `total_cmp`, to be kept in a set.
#[derive(Debug, Clone, Copy)]
struct F(f64);
impl Ord for F {
    fn cmp(&self, o: &F) -> Ordering {
        self.0.total_cmp(&o.0)
    }
}
impl PartialOrd for F {
    fn partial_cmp(&self, o: &F) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl PartialEq for F {
    fn eq(&self, o: &F) -> bool {
        self.cmp(o) == Ordering::Equal
    }
}
impl Eq for F {}

/// Strings or floats: a finite set, or every one except a finite set.
#[derive(Debug, Clone, PartialEq)]
enum Part<T: Ord + Clone> {
    Finite(BTreeSet<T>),
    Cofinite(BTreeSet<T>),
}

impl<T: Ord + Clone> Part<T> {
    fn none() -> Part<T> {
        Part::Finite(BTreeSet::new())
    }
    fn all() -> Part<T> {
        Part::Cofinite(BTreeSet::new())
    }
    fn union(&self, o: &Part<T>) -> Part<T> {
        use Part::*;
        match (self, o) {
            (Finite(a), Finite(b)) => Finite(a | b),
            (Cofinite(a), Cofinite(b)) => Cofinite(a & b),
            (Finite(f), Cofinite(c)) | (Cofinite(c), Finite(f)) => Cofinite(c - f),
        }
    }
    fn complement(&self) -> Part<T> {
        match self {
            Part::Finite(a) => Part::Cofinite(a.clone()),
            Part::Cofinite(a) => Part::Finite(a.clone()),
        }
    }
}

/// The named integer types: `Int8` to `Int64`, `UInt8` to `UInt32`.
pub const INT_TYPES: [(&str, i128, i128); 7] = [
    ("Int8", i8::MIN as i128, i8::MAX as i128),
    ("Int16", i16::MIN as i128, i16::MAX as i128),
    ("Int32", i32::MIN as i128, i32::MAX as i128),
    ("Int64", i64::MIN as i128, i64::MAX as i128),
    ("UInt8", 0, u8::MAX as i128),
    ("UInt16", 0, u16::MAX as i128),
    ("UInt32", 0, u32::MAX as i128),
];

#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    /// Membership of `false` and `true`.
    bools: [bool; 2],
    /// Sorted, disjoint, non-adjacent inclusive intervals; `NEG_INF` and
    /// `POS_INF` stand for unbounded ends.
    ints: Vec<(i128, i128)>,
    floats: Part<F>,
    strs: Part<String>,
    /// Values that are none of the above.
    other: bool,
}

impl Set {
    pub fn empty() -> Set {
        Set {
            bools: [false; 2],
            ints: vec![],
            floats: Part::none(),
            strs: Part::none(),
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
            strs: Part::all(),
            ..Set::empty()
        }
    }
    pub fn float() -> Set {
        Set {
            floats: Part::all(),
            ..Set::empty()
        }
    }
    pub fn any() -> Set {
        Set {
            other: true,
            ..Set::bool()
                .union(&Set::int())
                .union(&Set::float())
                .union(&Set::string())
        }
    }
    /// A named integer type, such as `Int64`.
    pub fn int_type(name: &str) -> Option<Set> {
        let &(_, lo, hi) = INT_TYPES.iter().find(|t| t.0 == name)?;
        Some(Set {
            ints: vec![(lo, hi)],
            ..Set::empty()
        })
    }
    /// The integers from `lo` to `hi`, both included; `None` is unbounded.
    /// Bounds stay strictly between the sentinels for unbounded ends.
    pub fn range(lo: Option<i128>, hi: Option<i128>) -> Set {
        let lo = lo.map_or(NEG_INF, |x| x.max(NEG_INF + 1));
        let hi = hi.map_or(POS_INF, |x| x.min(POS_INF - 1));
        Set {
            ints: if lo <= hi { vec![(lo, hi)] } else { vec![] },
            ..Set::empty()
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
            Value::Float(x) => Set {
                floats: Part::Finite([F(*x)].into()),
                ..Set::empty()
            },
            Value::Str(x) => Set {
                strs: Part::Finite([x.clone()].into()),
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
        Set {
            bools: [self.bools[0] || o.bools[0], self.bools[1] || o.bools[1]],
            ints: merged,
            floats: self.floats.union(&o.floats),
            strs: self.strs.union(&o.strs),
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
        Set {
            bools: [!self.bools[0], !self.bools[1]],
            ints,
            floats: self.floats.complement(),
            strs: self.strs.complement(),
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

    /// Every value, if the set is finite, small enough to list, and its
    /// integers fit the 64 bits Mitra computes with.
    pub fn values(&self) -> Option<Vec<Value>> {
        if self.count()? > 1_000_000.0 {
            return None;
        }
        let fits = |x: i128| i64::try_from(x).is_ok();
        if self.ints.iter().any(|&(lo, hi)| !fits(lo) || !fits(hi)) {
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
        if let Part::Finite(s) = &self.floats {
            out.extend(s.iter().map(|x| Value::Float(x.0)));
        }
        if let Part::Finite(s) = &self.strs {
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
        match (&self.strs, &self.floats) {
            (Part::Finite(s), Part::Finite(fl)) if !unbounded && !self.other => {
                // In floats: a sum of i128 widths can overflow.
                let ints: f64 = self
                    .ints
                    .iter()
                    .map(|&(lo, hi)| hi as f64 - lo as f64 + 1.0)
                    .sum();
                Some(
                    self.bools.iter().filter(|b| **b).count() as f64
                        + ints
                        + fl.len() as f64
                        + s.len() as f64,
                )
            }
            _ => None,
        }
    }
}

impl fmt::Display for Set {
    /// The shortest readable form: `Value`, `Bool`, `{1, "a"}`, `0..=100`,
    /// `Int \ {5}`, joined by ` | `.
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
                // Past 64 bits a single integer can't be written as a value.
                if lo == hi && i64::try_from(lo).is_ok() {
                    points.push(lo.to_string());
                    continue;
                }
                if let Some(t) = INT_TYPES.iter().find(|t| (t.1, t.2) == (lo, hi)) {
                    parts.push(t.0.into());
                    continue;
                }
                parts.push(match (lo, hi) {
                    (NEG_INF, hi) => format!("..={hi}"),
                    (lo, POS_INF) => format!("{lo}.."),
                    (lo, hi) => format!("{lo}..={hi}"),
                });
            }
        }
        let mut cofinite = |name: &str, all: bool, but: Vec<String>, points: &mut Vec<String>| {
            if all && but.is_empty() {
                parts.push(name.into());
            } else if all {
                parts.push(format!("({name} \\ {{{}}})", but.join(", ")));
            } else {
                points.extend(but);
            }
        };
        let show_f = |s: &BTreeSet<F>| s.iter().map(|x| Value::Float(x.0).to_string()).collect();
        match &self.floats {
            Part::Finite(s) => cofinite("Float64", false, show_f(s), &mut points),
            Part::Cofinite(s) => cofinite("Float64", true, show_f(s), &mut points),
        }
        let show_s = |s: &BTreeSet<String>| s.iter().map(|x| format!("\"{x}\"")).collect();
        match &self.strs {
            Part::Finite(s) => cofinite("String", false, show_s(s), &mut points),
            Part::Cofinite(s) => cofinite("String", true, show_s(s), &mut points),
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
            let bare = p.starts_with('(');
            return write!(f, "{}", if bare { &p[1..p.len() - 1] } else { p });
        }
        write!(f, "{}", parts.join(" | "))
    }
}

/// A set printed so it can follow `\` or `&` unambiguously.
pub fn paren(s: &Set) -> String {
    let t = s.to_string();
    if t.contains(" | ") || t.contains(" \\ ") {
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
            Set::string().union(&Set::float()).union(&Set {
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
            "0..=4 | 6..=100 | String"
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
            "Value \\ (-3..=2 | String)"
        );
    }
}
