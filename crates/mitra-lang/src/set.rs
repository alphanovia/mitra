//! Sets of values: the types and every subset of inputs Mitra talks about.
//!
//! The universe `Any` splits into booleans, integers, 64-bit floats,
//! strings and everything else. A set is stored per part, in a normal form,
//! so equal sets compare equal.

use crate::tuples::Tuples;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
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
    /// `(1, "a")`: two values or more.
    Tuple(Vec<Value>),
    /// `Number(1, "m")`: a record's name and the values of its fields.
    Record(String, Vec<Value>),
    /// `[1, 2, 3]`: any number of values, in order.
    List(Vec<Value>),
}

/// What kind of composite value: a tuple of n values, or a named record of
/// n fields.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    Tuple(usize),
    Record(String, usize),
}

/// A set value `{1, 2}` is the record `Set` of one list, sorted and
/// without repeats; a map value `{"a": 1}` is the record `Map` of one list
/// of `(key, value)` tuples, sorted by key. Both names are keywords, so no
/// declared record takes them.
pub const SET: &str = "Set";
pub const MAP: &str = "Map";

impl Value {
    /// The set value of `items`.
    pub fn set_of(mut items: Vec<Value>) -> Value {
        items.sort();
        items.dedup();
        Value::Record(SET.into(), vec![Value::List(items)])
    }
    /// The map value of `pairs`; `None` when a key comes twice.
    pub fn map_of(mut pairs: Vec<(Value, Value)>) -> Option<Value> {
        pairs.sort();
        if pairs.windows(2).any(|w| w[0].0 == w[1].0) {
            return None;
        }
        let pairs = pairs.into_iter().map(|(k, v)| Value::Tuple(vec![k, v]));
        Some(Value::Record(
            MAP.into(),
            vec![Value::List(pairs.collect())],
        ))
    }
    /// The elements of a set value.
    pub fn as_set(&self) -> Option<&[Value]> {
        match self {
            Value::Record(name, parts) if name == SET => match &parts[..] {
                [Value::List(items)] => Some(items),
                _ => None,
            },
            _ => None,
        }
    }
    /// The keys and values of a map value.
    pub fn as_map(&self) -> Option<Vec<(&Value, &Value)>> {
        let Value::Record(name, parts) = self else {
            return None;
        };
        let [Value::List(pairs)] = &parts[..] else {
            return None;
        };
        if name != MAP {
            return None;
        }
        pairs
            .iter()
            .map(|p| match p {
                Value::Tuple(kv) if kv.len() == 2 => Some((&kv[0], &kv[1])),
                _ => None,
            })
            .collect()
    }
}

impl Shape {
    fn arity(&self) -> usize {
        match self {
            Shape::Tuple(n) | Shape::Record(_, n) => *n,
        }
    }
    fn value(&self, parts: Vec<Value>) -> Value {
        match self {
            Shape::Tuple(_) => Value::Tuple(parts),
            Shape::Record(name, _) => Value::Record(name.clone(), parts),
        }
    }
    /// `(a, b)` or `Name(a, b)`.
    fn show(&self, parts: &[String]) -> String {
        match self {
            Shape::Tuple(_) => format!("({})", parts.join(", ")),
            // A variant without fields is its name alone.
            Shape::Record(name, 0) => name.clone(),
            Shape::Record(name, _) => format!("{name}({})", parts.join(", ")),
        }
    }
}

impl Value {
    fn rank(&self) -> u8 {
        match self {
            Value::Bool(_) => 0,
            Value::Int(_) => 1,
            Value::Float(_) => 2,
            Value::Str(_) => 3,
            Value::Tuple(_) => 4,
            Value::Record(..) => 5,
            Value::List(_) => 6,
        }
    }
    /// The shape and the parts of a tuple or a record.
    pub fn parts(&self) -> Option<(Shape, &[Value])> {
        match self {
            Value::Tuple(vs) => Some((Shape::Tuple(vs.len()), vs)),
            Value::Record(name, vs) => Some((Shape::Record(name.clone(), vs.len()), vs)),
            _ => None,
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
            (Value::Tuple(a), Value::Tuple(b)) => a.cmp(b),
            (Value::Record(n, a), Value::Record(m, b)) => (n, a).cmp(&(m, b)),
            (Value::List(a), Value::List(b)) => a.cmp(b),
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
            Value::Str(s) => write!(f, "{}", quote(s)),
            Value::Record(..) if self.as_set().is_some() => {
                let items = self.as_set().expect("checked");
                let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "{{{}}}", parts.join(", "))
            }
            Value::Record(..) if self.as_map().is_some() => {
                let pairs = self.as_map().expect("checked");
                let parts: Vec<String> = pairs.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                match parts.is_empty() {
                    true => write!(f, "{{:}}"),
                    false => write!(f, "{{{}}}", parts.join(", ")),
                }
            }
            Value::Tuple(vs) | Value::Record(_, vs) => {
                let (shape, _) = self.parts().expect("composite");
                let parts: Vec<String> = vs.iter().map(|v| v.to_string()).collect();
                write!(f, "{}", shape.show(&parts))
            }
            Value::List(vs) => {
                let parts: Vec<String> = vs.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", parts.join(", "))
            }
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

/// Tuples and records, by shape. `rest` says whether the shapes not listed
/// are wholly in the set, with the values that are of no kind Mitra knows.
/// No shape is listed with what `rest` already says: none of its values
/// when `rest` is false, all of them when it is true.
#[derive(Debug, Clone)]
struct Composites {
    rest: bool,
    shapes: BTreeMap<Shape, Tuples>,
}

impl Composites {
    fn none() -> Composites {
        Composites {
            rest: false,
            shapes: BTreeMap::new(),
        }
    }
    fn full(shape: &Shape) -> Tuples {
        Tuples::product(vec![Set::any(); shape.arity()])
    }
    fn of(&self, shape: &Shape) -> Tuples {
        match self.shapes.get(shape) {
            Some(t) => t.clone(),
            None if self.rest => Composites::full(shape),
            None => Tuples::empty(shape.arity()),
        }
    }
    fn build(rest: bool, shapes: impl Iterator<Item = (Shape, Tuples)>) -> Composites {
        let said = |shape: &Shape, t: &Tuples| match rest {
            true => Composites::full(shape).is_subset(t),
            false => t.is_empty(),
        };
        Composites {
            rest,
            shapes: shapes.filter(|(s, t)| !said(s, t)).collect(),
        }
    }
    fn union(&self, o: &Composites) -> Composites {
        let shapes: BTreeSet<&Shape> = self.shapes.keys().chain(o.shapes.keys()).collect();
        Composites::build(
            self.rest || o.rest,
            shapes
                .into_iter()
                .map(|s| (s.clone(), self.of(s).union(&o.of(s)))),
        )
    }
    fn intersect(&self, o: &Composites) -> Composites {
        let shapes: BTreeSet<&Shape> = self.shapes.keys().chain(o.shapes.keys()).collect();
        Composites::build(
            self.rest && o.rest,
            shapes.into_iter().map(|s| {
                // What `rest` says of a shape need not be spelled out.
                let t = match (self.shapes.get(s), o.shapes.get(s)) {
                    (Some(a), Some(b)) => a.intersect(b),
                    (Some(a), None) if o.rest => a.clone(),
                    (None, Some(b)) if self.rest => b.clone(),
                    _ => Tuples::empty(s.arity()),
                };
                (s.clone(), t)
            }),
        )
    }
    fn complement(&self) -> Composites {
        Composites::build(
            !self.rest,
            self.shapes
                .iter()
                .map(|(s, t)| (s.clone(), Composites::full(s).minus(t))),
        )
    }
}

impl PartialEq for Composites {
    fn eq(&self, o: &Composites) -> bool {
        self.rest == o.rest
            && self.shapes.len() == o.shapes.len()
            && self
                .shapes
                .iter()
                .all(|(s, t)| o.shapes.get(s).is_some_and(|u| t.same(u)))
    }
}

/// The lists whose elements are all in `all` (`None`: any value), with at
/// least one element in each set of `some`.
#[derive(Debug, Clone)]
struct Term {
    all: Option<Box<Set>>,
    some: Vec<Set>,
}

impl Term {
    fn all(&self) -> Set {
        self.all.as_deref().cloned().unwrap_or_else(Set::any)
    }
    fn contains(&self, list: &[Value]) -> bool {
        list.iter()
            .all(|v| self.all.as_ref().is_none_or(|all| all.contains(v)))
            && self.some.iter().all(|f| list.iter().any(|v| f.contains(v)))
    }
    fn intersect(&self, o: &Term) -> Term {
        let all = match (&self.all, &o.all) {
            (Some(a), Some(b)) => Some(Box::new(a.intersect(b))),
            (a, b) => a.clone().or(b.clone()),
        };
        Term {
            all,
            some: [self.some.clone(), o.some.clone()].concat(),
        }
    }
    /// The lists outside the term, as terms.
    fn complement(&self) -> Vec<Term> {
        // Nothing is outside "any value".
        let mut out: Vec<Term> = self
            .all
            .iter()
            .map(|all| Term {
                all: None,
                some: vec![all.complement()],
            })
            .collect();
        out.extend(self.some.iter().map(|f| Term {
            all: Some(Box::new(f.complement())),
            some: vec![],
        }));
        out
    }
}

/// A set of lists: those of the terms of `base`, without the lists of
/// `minus`, with the lists of `plus`. Every term holds infinitely many
/// lists; `plus` holds no list of `base`, and `minus` only lists of `base`.
#[derive(Debug, Clone)]
struct Lists {
    base: Vec<Term>,
    plus: BTreeSet<Vec<Value>>,
    minus: BTreeSet<Vec<Value>>,
}

impl Lists {
    fn none() -> Lists {
        Lists::build(Vec::new(), BTreeSet::new(), |_| false)
    }
    /// The lists of `terms`, corrected on the `points` so that each is in
    /// the set exactly when `member` says so.
    fn build(
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
    fn points(&self) -> BTreeSet<Vec<Value>> {
        self.plus.union(&self.minus).cloned().collect()
    }
    fn contains(&self, list: &[Value]) -> bool {
        self.plus.contains(list)
            || (!self.minus.contains(list) && self.base.iter().any(|t| t.contains(list)))
    }
    fn union(&self, o: &Lists) -> Lists {
        let points = self.points().union(&o.points()).cloned().collect();
        Lists::build([self.base.clone(), o.base.clone()].concat(), points, |p| {
            self.contains(p) || o.contains(p)
        })
    }
    fn intersect(&self, o: &Lists) -> Lists {
        let terms = self
            .base
            .iter()
            .flat_map(|a| o.base.iter().map(|b| a.intersect(b)))
            .collect();
        let points = self.points().union(&o.points()).cloned().collect();
        Lists::build(terms, points, |p| self.contains(p) && o.contains(p))
    }
    fn complement(&self) -> Lists {
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
    fn is_empty(&self) -> bool {
        self.base.is_empty() && self.plus.is_empty()
    }

    /// The lists as text: single lists into `points`, the rest into
    /// `parts`. `whole(E)` writes the lists with every element in E, and
    /// `value` one list; sets and maps are printed through their lists.
    fn show(
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
fn entries(pairs: &Set) -> String {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Set {
    /// Membership of `false` and `true`.
    bools: [bool; 2],
    /// Sorted, disjoint, non-adjacent inclusive intervals; `NEG_INF` and
    /// `POS_INF` stand for unbounded ends.
    ints: Vec<(i128, i128)>,
    floats: Part<F>,
    strs: Part<String>,
    lists: Lists,
    /// Tuples, records, and the values that are none of the above.
    composites: Composites,
}

impl Set {
    pub fn empty() -> Set {
        Set {
            bools: [false; 2],
            ints: vec![],
            floats: Part::none(),
            strs: Part::none(),
            lists: Lists::none(),
            composites: Composites::none(),
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
            lists: Lists {
                base: vec![Term {
                    all: None,
                    some: vec![],
                }],
                plus: BTreeSet::new(),
                minus: BTreeSet::new(),
            },
            composites: Composites {
                rest: true,
                shapes: BTreeMap::new(),
            },
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
            Value::Tuple(vs) | Value::Record(_, vs) => {
                let (shape, _) = v.parts().expect("composite");
                Set::composite(shape, Tuples::point(vs))
            }
            Value::List(vs) => Set {
                lists: Lists::build(Vec::new(), [vs.clone()].into(), |p| p == vs),
                ..Set::empty()
            },
        }
    }
    /// `List(T)`: the lists whose elements are all in `elements`.
    pub fn list(elements: Set) -> Set {
        let term = Term {
            all: Some(Box::new(elements)),
            some: vec![],
        };
        Set {
            lists: Lists::build(vec![term], BTreeSet::new(), |_| true),
            ..Set::empty()
        }
    }
    /// `Set(T)`: the set values whose elements are all in T.
    pub fn sets(elements: Set) -> Set {
        Set::record(SET, vec![Set::list(elements)])
    }
    /// `Map(K, V)`: the map values with keys in K and values in V.
    pub fn maps(keys: Set, values: Set) -> Set {
        Set::record(MAP, vec![Set::list(Set::tuple(vec![keys, values]))])
    }
    /// The tuples or records of one shape whose parts are in `parts`.
    pub fn composite(shape: Shape, parts: Tuples) -> Set {
        Set {
            composites: Composites::build(false, [(shape, parts)].into_iter()),
            ..Set::empty()
        }
    }
    /// `(A, B)`: the tuples with one part in each set.
    pub fn tuple(parts: Vec<Set>) -> Set {
        Set::composite(Shape::Tuple(parts.len()), Tuples::product(parts))
    }
    /// `Name(A, B)`: the records with one field in each set.
    pub fn record(name: &str, fields: Vec<Set>) -> Set {
        let shape = Shape::Record(name.to_string(), fields.len());
        Set::composite(shape, Tuples::product(fields))
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
            lists: self.lists.union(&o.lists),
            composites: self.composites.union(&o.composites),
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
            lists: self.lists.complement(),
            composites: self.composites.complement(),
        }
    }

    pub fn intersect(&self, o: &Set) -> Set {
        // The scalar parts by De Morgan; lists and composites directly,
        // since their complements are costly.
        let scalars = |s: &Set| Set {
            lists: Lists::none(),
            composites: Composites::none(),
            ..s.clone()
        };
        let (a, b) = (scalars(self), scalars(o));
        Set {
            lists: self.lists.intersect(&o.lists),
            composites: self.composites.intersect(&o.composites),
            ..a.complement().union(&b.complement()).complement_scalars()
        }
    }

    /// The complement of the scalar parts alone.
    fn complement_scalars(&self) -> Set {
        let scalars = Set {
            lists: Lists::none(),
            composites: Composites::none(),
            ..self.clone()
        };
        Set {
            lists: Lists::none(),
            composites: Composites::none(),
            ..scalars.complement()
        }
    }

    pub fn minus(&self, o: &Set) -> Set {
        // A set with no tuple and no record loses none to those of `o`:
        // their complement is long to work out, and changes nothing here.
        if self.composites.shapes.is_empty() && !self.composites.rest {
            let light = Set {
                bools: o.bools,
                ints: o.ints.clone(),
                floats: o.floats.clone(),
                strs: o.strs.clone(),
                lists: o.lists.clone(),
                composites: Composites::none(),
            };
            return self.intersect(&light.complement());
        }
        self.intersect(&o.complement())
    }

    pub fn is_empty(&self) -> bool {
        self.bools == [false; 2]
            && self.ints.is_empty()
            && self.floats == Part::none()
            && self.strs == Part::none()
            && self.lists.is_empty()
            && !self.composites.rest
            && self.composites.shapes.is_empty()
    }

    pub fn is_subset(&self, o: &Set) -> bool {
        self.minus(o).is_empty()
    }

    pub fn contains(&self, v: &Value) -> bool {
        match v {
            Value::List(vs) => self.lists.contains(vs),
            Value::Tuple(vs) | Value::Record(_, vs) => {
                let (shape, _) = v.parts().expect("composite");
                match self.composites.shapes.get(&shape) {
                    Some(t) => t.contains(vs),
                    None => self.composites.rest,
                }
            }
            _ => Set::point(v).is_subset(self),
        }
    }

    /// How many kinds of tuples and records the set names.
    pub fn shapes(&self) -> usize {
        self.composites.shapes.len()
    }
    /// Like `contains`, for a set with marks: a mark `$T`, a record
    /// without fields, stands for the set `types` gives for `T`.
    pub fn conforms(&self, v: &Value, types: &BTreeMap<String, Set>) -> bool {
        let marks = self.composites.shapes.keys().filter_map(|s| match s {
            Shape::Record(name, 0) => types.get(name.strip_prefix('$')?),
            _ => None,
        });
        if marks.into_iter().any(|t| t.conforms(v, types)) {
            return true;
        }
        match v {
            Value::List(vs) => {
                let term = |t: &Term| {
                    let all = |v| t.all.as_ref().is_none_or(|all| all.conforms(v, types));
                    vs.iter().all(all)
                        && t.some
                            .iter()
                            .all(|f| vs.iter().any(|v| f.conforms(v, types)))
                };
                self.lists.plus.contains(vs)
                    || (!self.lists.minus.contains(vs) && self.lists.base.iter().any(term))
            }
            Value::Tuple(vs) | Value::Record(_, vs) => {
                let (shape, _) = v.parts().expect("composite");
                match self.composites.shapes.get(&shape) {
                    Some(t) => t
                        .boxes()
                        .iter()
                        .any(|sets| sets.iter().zip(vs).all(|(s, v)| s.conforms(v, types))),
                    None => self.composites.rest,
                }
            }
            _ => self.contains(v),
        }
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
        for (shape, t) in &self.composites.shapes {
            out.extend(t.values()?.into_iter().map(|parts| shape.value(parts)));
        }
        out.extend(self.lists.plus.iter().map(|l| Value::List(l.clone())));
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
            (Part::Finite(s), Part::Finite(fl))
                if !unbounded && !self.composites.rest && self.lists.base.is_empty() =>
            {
                let mut composites = 0.0;
                for t in self.composites.shapes.values() {
                    composites += t.count()?;
                }
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
                        + s.len() as f64
                        + self.lists.plus.len() as f64
                        + composites,
                )
            }
            _ => None,
        }
    }
}

impl fmt::Display for Set {
    /// The shortest readable form: `Any`, `Bool`, `{1, "a"}`, `0..=100`,
    /// `Int \ {5}`, joined by ` | `.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.composites.rest {
            let rest = self.complement();
            return if rest.is_empty() {
                write!(f, "Any")
            } else {
                write!(f, "Any \\ {}", paren(&rest))
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
                format!("\u{1}Int \\ {}", paren(&rest))
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
                parts.push(format!("\u{1}{name} \\ {{{}}}", but.join(", ")));
            } else {
                points.extend(but);
            }
        };
        let show_f = |s: &BTreeSet<F>| s.iter().map(|x| Value::Float(x.0).to_string()).collect();
        match &self.floats {
            Part::Finite(s) => cofinite("Float64", false, show_f(s), &mut points),
            Part::Cofinite(s) => cofinite("Float64", true, show_f(s), &mut points),
        }
        let show_s = |s: &BTreeSet<String>| s.iter().map(|x| quote(x)).collect();
        match &self.strs {
            Part::Finite(s) => cofinite("String", false, show_s(s), &mut points),
            Part::Cofinite(s) => cofinite("String", true, show_s(s), &mut points),
        }
        for (shape, t) in &self.composites.shapes {
            // Sets and maps are printed from the lists inside: `Set(T)`,
            // `Map(K, V)`, `Set(T) \ {{1, 2}}`.
            if matches!(shape, Shape::Record(n, 1) if n == SET || n == MAP) {
                let inner = t.boxes().iter().fold(Set::empty(), |s, b| s.union(&b[0]));
                let only_lists = Set {
                    lists: inner.lists.clone(),
                    ..Set::empty()
                } == inner;
                if only_lists {
                    let value =
                        |l: &Vec<Value>| shape.value(vec![Value::List(l.clone())]).to_string();
                    let whole = |all: &Set| match n_is_set(shape) {
                        true => format!("Set<{all}>"),
                        false => format!("Map<{}>", entries(all)),
                    };
                    inner.lists.show(&whole, &value, &mut points, &mut parts);
                    continue;
                }
            }
            // Few enough: each value. Otherwise the products of sets.
            let show = |b: &[Set]| {
                let sets: Vec<String> = b.iter().map(|s| s.to_string()).collect();
                shape.show(&sets)
            };
            // Listed only when few, without counting a large set out.
            let few = |t: &Tuples| match t.size_bound() {
                Some(n) if n <= 64.0 => t.values().filter(|vs| vs.len() <= 3),
                _ => None,
            };
            if let Some(vs) = few(t) {
                points.extend(vs.into_iter().map(|v| shape.value(v).to_string()));
                continue;
            }
            // Single values, whatever their size: each as a value.
            let single = |b: &Vec<Set>| b.iter().map(Set::single).collect::<Option<Vec<Value>>>();
            if let Some(vs) = t.boxes().iter().map(single).collect::<Option<Vec<_>>>() {
                points.extend(vs.into_iter().map(|v| shape.value(v).to_string()));
                continue;
            }
            // One product of sets is its own hull: no value is missing.
            if let [only] = t.boxes() {
                parts.push(show(only));
                continue;
            }
            // One product of sets without a few values: `Number(..) \ {..}`.
            let hull: Vec<Set> = (0..shape.arity())
                .map(|i| t.boxes().iter().fold(Set::empty(), |s, b| s.union(&b[i])))
                .collect();
            let holes = Tuples::product(hull.clone()).minus(t);
            match few(&holes) {
                Some(vs) if vs.is_empty() => parts.push(show(&hull)),
                Some(vs) => {
                    let but: Vec<String> =
                        vs.into_iter().map(|v| shape.value(v).to_string()).collect();
                    parts.push(format!("\u{1}{} \\ {{{}}}", show(&hull), but.join(", ")));
                }
                None => parts.extend(t.boxes().iter().map(|b| show(b))),
            }
        }
        let show_l = |l: &Vec<Value>| Value::List(l.clone()).to_string();
        let whole = |all: &Set| format!("List<{all}>");
        self.lists.show(&whole, &show_l, &mut points, &mut parts);
        if !points.is_empty() {
            parts.insert(0, format!("{{{}}}", points.join(", ")));
        }
        if parts.is_empty() {
            return write!(f, "{{}}");
        }
        // A difference `T \ X` needs parentheses, unless it is alone.
        let lone = parts.len() == 1;
        let parts: Vec<String> = parts
            .iter()
            .map(|p| match p.strip_prefix('\u{1}') {
                Some(p) if lone => p.to_string(),
                Some(p) => format!("({p})"),
                None => p.clone(),
            })
            .collect();
        write!(f, "{}", parts.join(" | "))
    }
}

fn n_is_set(shape: &Shape) -> bool {
    matches!(shape, Shape::Record(n, _) if n == SET)
}

/// A string as written in Mitra, with its escapes.
// mitra: implements print::quote
pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\n' => out += "\\n",
            '\t' => out += "\\t",
            '"' => out += "\\\"",
            '\\' => out += "\\\\",
            c => out.push(c),
        }
    }
    out + "\""
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

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.

    // mitra: evidence for t_quote_plain
    #[test]
    fn t_quote_plain() {
        assert_eq!(quote("a b"), "\"a b\"");
    }

    // mitra: evidence for t_quote_empty
    #[test]
    fn t_quote_empty() {
        assert_eq!(quote(""), "\"\"");
    }

    // mitra: evidence for t_quote_escapes
    #[test]
    fn t_quote_escapes() {
        assert_eq!(quote("a\"b\\c\nd\te"), "\"a\\\"b\\\\c\\nd\\te\"");
    }

    #[test]
    fn algebra() {
        let small = Set::interval(0, 100);
        assert!(small.is_subset(&Set::int()));
        assert_eq!(
            Set::int().union(&Set::bool()).complement(),
            Set::string().union(&Set::float()).union(&Set {
                lists: Set::any().lists,
                composites: Composites {
                    rest: true,
                    shapes: BTreeMap::new(),
                },
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
        assert_eq!(show(Set::any()), "Any");
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
            "Any \\ {5}"
        );
        // Found by the grammar fuzzer: the part after `\` needs parentheses.
        assert_eq!(
            show(Set::any().minus(&Set::interval(-3, 2).union(&Set::string()))),
            "Any \\ (-3..=2 | String)"
        );
    }
}
