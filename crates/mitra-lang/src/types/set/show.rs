//! A set as text, the shortest readable form.

use super::*;

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

pub(super) fn n_is_set(shape: &Shape) -> bool {
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
