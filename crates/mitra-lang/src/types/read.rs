//! Reading a type: values, ranges, sets of values, collections, the
//! variants of an enum.

use crate::reader::prelude::*;

impl P {
    /// A weight or a confidence: `1`, `0.5`.
    pub(crate) fn number(&mut self) -> R<f64> {
        match self.old() {
            Some(Old::Num(s)) => {
                self.pos += 1;
                match s.parse::<f64>() {
                    Ok(x) if x.is_finite() => Ok(x),
                    _ => self.err(format!("`{s}` is not a number Mitra can use")),
                }
            }
            _ => self.err("expected a number"),
        }
    }

    /// A natural number, negated when `negative`: down to `i64::MIN`.
    pub(crate) fn signed(&mut self, negative: bool) -> R<i64> {
        match self.old() {
            Some(Old::Num(s)) if !s.contains('.') => {
                let n: Option<i128> = s.parse().ok();
                let n = n.map(|n| if negative { -n } else { n });
                match n.and_then(|n| i64::try_from(n).ok()) {
                    Some(n) => {
                        self.pos += 1;
                        Ok(n)
                    }
                    None => self.err(format!("`{s}` is outside the 64-bit integers")),
                }
            }
            _ => self.err("expected an integer"),
        }
    }

    /// An integer or a float literal, negated when `negative`.
    pub(crate) fn numeral(&mut self, negative: bool) -> R<Value> {
        match self.old() {
            Some(Old::Num(s)) if s.contains('.') => match s.parse::<f64>() {
                Ok(x) if x.is_finite() => {
                    self.pos += 1;
                    Ok(Value::Float(if negative { -x } else { x }))
                }
                _ => self.err(format!("`{s}` is outside the 64-bit floats")),
            },
            _ => Ok(Value::Int(self.signed(negative)?)),
        }
    }

    /// `( value {, value} )`, after the `(`.
    pub(crate) fn values(&mut self) -> R<Vec<Value>> {
        let mut parts = vec![self.value()?];
        while self.eat(",") {
            parts.push(self.value()?);
        }
        self.expect(")")?;
        Ok(parts)
    }

    /// value := "true" | "false" | ["-"] number | string
    ///        | "(" value "," value {"," value} ")" | name "(" value {"," value} ")"
    pub(crate) fn value(&mut self) -> R<Value> {
        // A value inside a value, and so on: not without end.
        const DEEPEST: usize = 64;
        if self.nested == DEEPEST {
            return self.err(format!("a value nested more than {DEEPEST} deep"));
        }
        self.nested += 1;
        let v = self.value_inner();
        self.nested -= 1;
        v
    }

    pub(crate) fn value_inner(&mut self) -> R<Value> {
        if self.eat("-") {
            return self.numeral(true);
        }
        if self.eat("(") {
            if self.eat(")") {
                // `()`: the one value of the unit type.
                return Ok(Value::Tuple(Vec::new()));
            }
            let parts = self.values()?;
            if parts.len() < 2 {
                return self.err("a tuple has two values or more: `(1, 2)`");
            }
            return Ok(Value::Tuple(parts));
        }
        if self.is_name() {
            // `Some(1)`, or a variant without fields: `None`.
            let name = self.path()?;
            let scoped = self.qualified(&name);
            let name = match self.all.contains(&scoped) {
                true => scoped,
                false => name,
            };
            if self.records.iter().any(|r| r.name == name && r.open) {
                return self.err(unwritten(&name));
            }
            let parts = match self.eat("(") {
                true => self.values()?,
                false => Vec::new(),
            };
            return Ok(Value::Record(name, parts));
        }
        if self.eat("[") {
            let mut parts = Vec::new();
            while !self.eat("]") {
                if !parts.is_empty() {
                    self.expect(",")?;
                }
                parts.push(self.value()?);
            }
            return Ok(Value::List(parts));
        }
        if self.eat("{") {
            // `{}`, `{1, 2}`: a set. `{:}`, `{"a": 1}`: a map.
            let (mut items, mut pairs) = (Vec::new(), Vec::new());
            let mut map = self.eat(":");
            while !self.eat("}") {
                if map && pairs.is_empty() && self.is("}") {
                    continue;
                }
                if !(items.is_empty() && pairs.is_empty()) {
                    self.expect(",")?;
                }
                let v = self.value()?;
                if items.is_empty() && pairs.is_empty() && !map {
                    map = self.is(":");
                }
                match map {
                    true => {
                        self.expect(":")?;
                        pairs.push((v, self.value()?));
                    }
                    false => items.push(v),
                }
            }
            return match map {
                true => match Value::map_of(pairs) {
                    Some(m) => Ok(m),
                    None => self.err("a key comes twice in this map"),
                },
                false => Ok(Value::set_of(items)),
            };
        }
        match self.old() {
            Some(Old::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Old::Num(_)) => self.numeral(false),
            Some(Old::Str(s)) => {
                self.pos += 1;
                Ok(Value::Str(s))
            }
            _ => self.err(
                "expected a value: true, false, a number, a string, a tuple `(1, 2)`, \
                 a record `Name(1, 2)` or a list `[1, 2]`",
            ),
        }
    }

    /// A range bound: `5` or `-5`, if one is there. `Int` is unbounded, so
    /// bounds may go past 64 bits, up to 2^126 either way.
    pub(crate) fn bound(&mut self) -> R<Option<i128>> {
        let negative = self.eat("-");
        match self.old() {
            Some(Old::Num(n)) if n.contains('.') => {
                self.err("ranges are over integers: write floats as a set, `{1.5, 2.5}`")
            }
            Some(Old::Num(n)) => {
                const LIMIT: i128 = 1 << 126;
                match n.parse::<i128>().ok().filter(|&x| x <= LIMIT) {
                    Some(x) => {
                        self.pos += 1;
                        Ok(Some(if negative { -x } else { x }))
                    }
                    None => self.err(format!("`{n}` is too large for a range bound")),
                }
            }
            _ if negative => self.err("expected an integer"),
            _ => Ok(None),
        }
    }

    /// set := inter { "|" inter }
    pub(crate) fn set(&mut self) -> R<Set> {
        let mut s = self.set_inter()?;
        while self.eat("|") || self.eat("∪") {
            s = s.union(&self.set_inter()?);
        }
        Ok(s)
    }

    /// inter := atom { ("&" | "\\") atom }, left to right.
    pub(crate) fn set_inter(&mut self) -> R<Set> {
        let mut s = self.set_atom()?;
        loop {
            s = if self.eat("&") || self.eat("∩") {
                s.intersect(&self.set_atom()?)
            } else if self.eat("\\") {
                s.minus(&self.set_atom()?)
            } else {
                return Ok(s);
            };
        }
    }

    /// `()`, `(S)`, `(S, T)`: the sets between parentheses.
    pub(crate) fn sets_in_parentheses(&mut self) -> R<Vec<Set>> {
        self.expect("(")?;
        let mut parts = Vec::new();
        if self.eat(")") {
            return Ok(parts);
        }
        parts.push(self.set()?);
        while self.eat(",") {
            parts.push(self.set()?);
        }
        self.expect(")")?;
        Ok(parts)
    }

    pub(crate) fn set_atom(&mut self) -> R<Set> {
        if self.is("(") {
            // `()` is the unit type; `(S)` groups; `(S, T)` is the tuples
            // of one value from each.
            let mut parts = self.sets_in_parentheses()?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Set::tuple(parts),
            });
        }
        if self.eat_keyword("fn") {
            // `fn(A, B) -> C`: the functions declared above that take
            // an A and a B, or more, and give a C. A type parameter
            // is any value in a result, and no value at all in a
            // parameter, so that every function fits.
            let start = self.pos;
            let mut parts = self.sets_in_parentheses()?;
            if !self.generics.is_empty() {
                self.pos = start;
                self.contra = !self.contra;
                let again = self.sets_in_parentheses();
                self.contra = !self.contra;
                parts = again?;
            }
            self.expect("->")?;
            let result = self.set_atom()?;
            let fits = |f: &&Function| {
                f.params.len() == parts.len()
                    && f.params.iter().zip(&parts).all(|(p, a)| a.is_subset(&p.1))
                    && f.range.is_subset(&result)
            };
            let named = |f: &Function| Set::record(&f.name, Vec::new());
            let all = self.functions.iter().filter(fits).map(named);
            let set = all.fold(Set::empty(), |s, f| s.union(&f));
            let each: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
            let written = format!("fn({}) -> {}", each.join(", "), crate::set::paren(&result));
            if !set.is_empty() {
                self.aliases.push((set.to_string(), written));
            }
            return Ok(set);
        }
        if self.peek() == "Vec" && self.kind() == Some(Kind::Name) {
            return self.err("`Vec` is Rust's name for `List`: `import std::rust::{Vec};`");
        }
        // Rust's names for the same types.
        let rust = match self.peek() {
            "bool" => "Bool",
            "i8" => "Int8",
            "i16" => "Int16",
            "i32" => "Int32",
            "i64" => "Int64",
            "u8" => "UInt8",
            "u16" => "UInt16",
            "u32" => "UInt32",
            "f64" => "Float64",
            "str" => "String",
            "HashSet" | "BTreeSet" => "Set",
            "HashMap" | "BTreeMap" => "Map",
            _ => "",
        };
        if !rust.is_empty() && self.kind() == Some(Kind::Name) {
            self.toks[self.pos].text = rust.to_string();
        }
        if self.eat_keyword("u64") || self.eat_keyword("usize") {
            return Ok(Set::range(Some(0), None));
        }
        if self.eat_keyword("List") {
            // `List<T>`: the lists whose elements are all in T.
            self.expect("<")?;
            let elements = self.set()?;
            self.expect(">")?;
            return Ok(Set::list(elements));
        }
        if self.eat_keyword("Set") {
            self.expect("<")?;
            let elements = self.set()?;
            self.expect(">")?;
            return Ok(Set::sets(elements));
        }
        if self.eat_keyword("Map") {
            self.expect("<")?;
            // `Map<K, V>`; with one set, its `(key, value)` pairs.
            let keys = self.set()?;
            if self.eat(">") {
                return Ok(Set::record(MAP, vec![Set::list(keys)]));
            }
            self.expect(",")?;
            let values = self.set()?;
            self.expect(">")?;
            return Ok(Set::maps(keys, values));
        }
        if self.is_name() {
            // `Number`: every value of the record. `Number(S, T)`: those
            // with one field in each set.
            let line = self.line();
            let name = self.ident()?;
            if let Some((_, set)) = self.generics.iter().find(|p| p.0 == name) {
                return Ok(match self.contra {
                    true => Set::empty(),
                    false => set.clone(),
                });
            }
            let mut args = Vec::new();
            if self.eat("<") {
                args.push(self.set()?);
                while self.eat(",") {
                    args.push(self.set()?);
                }
                self.expect(">")?;
            }
            let written = |args: &[Set]| {
                let each: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                match args.is_empty() {
                    true => name.clone(),
                    false => format!("{name}<{}>", each.join(", ")),
                }
            };
            // A record's name with sets, `Dir(String, S)`, is not the
            // record itself: only an enum takes parameters.
            let own = |me: &(String, String)| me.0 == name && (self.in_enum || args.is_empty());
            if let Some((_, me)) = self.me.clone().filter(own) {
                // The type inside its own declaration.
                if written(&args) != me {
                    return Err(Error::new(
                        line,
                        format!("inside its declaration, `{name}` is written `{me}`"),
                    ));
                }
                self.met = true;
                return Ok(match self.deep {
                    true => Set::record(&format!("${me}"), Vec::new()),
                    false => Set::any(),
                });
            }
            // `Option(Int)`: an enum, with a set for each of its parameters.
            if let Some(e) = self.enums.iter().find(|e| e.name == name).cloned() {
                if args.len() != e.params.len() {
                    return Err(Error::new(
                        line,
                        format!(
                            "`{name}` takes {} type(s), got {}",
                            e.params.len(),
                            args.len()
                        ),
                    ));
                }
                // The same enum with the same sets, met again: a record
                // of records is long to print, so it is worked out once.
                let again = |i: &&(String, bool, Vec<Set>, Set)| {
                    i.0 == name && i.1 == self.deep && i.2 == args
                };
                if let Some(met) = self.instances.iter().find(again) {
                    return Ok(met.3.clone());
                }
                let written = written(&args);
                let key = (name.clone(), self.deep, args.clone());
                let params = e.params.iter().cloned().zip(args).collect();
                let (variants, _, recursive) = self.variants(&e, params, line)?;
                let set = variants.iter().fold(Set::empty(), |s, v| s.union(&v.set()));
                self.instances.push((key.0, key.1, key.2, set.clone()));
                self.aliases.push((set.to_string(), written.clone()));
                // Under its plain name too: the printer names it.
                if let Some(recursive) = recursive {
                    self.recursive.extend(recursive);
                    self.recursive.insert(written, set.clone());
                }
                return Ok(set);
            }
            if !args.is_empty() {
                self.used.push(name.clone());
                return Ok(Set::record(&name, args));
            }
            if let Some(set) = self.recursive.get(&name).filter(|_| self.deep) {
                return Ok(set.clone());
            }
            return match self.records.iter().find(|r| r.name == name) {
                Some(r) => Ok(r.set()),
                None => Err(Error::new(
                    line,
                    format!(
                        "unknown type `{name}` (Bool, Int, Int8 to Int64, UInt8 to UInt32, \
                         Float64, String, Any, or a record or an enum declared above)"
                    ),
                )),
            };
        }
        if self.eat("{") {
            let mut s = Set::empty();
            if self.eat("}") {
                return Ok(s);
            }
            loop {
                s = s.union(&Set::point(&self.value()?));
                if self.eat("}") {
                    return Ok(s);
                }
                self.expect(",")?;
            }
        }
        if self.kind() == Some(Kind::Num) || self.is("-") || self.is("..") || self.is("..=") {
            // A range: `a..=b`, `a..b`, `a..`, `..=b`, `..b`.
            let lo = self.bound()?;
            let hi = if self.eat("..=") {
                match self.bound()? {
                    Some(hi) => Some(hi),
                    None => return self.err("`..=` needs an upper bound"),
                }
            } else if self.eat("..") {
                match self.bound()? {
                    // `a..b` leaves `b` out.
                    Some(hi) => Some(hi - 1),
                    None if lo.is_none() => return self.err("`..` alone: write `Int`"),
                    None => None,
                }
            } else {
                return self.err("expected `..=` or `..` after the lower bound");
            };
            return Ok(Set::range(lo, hi));
        }
        let set = match self.old() {
            Some(Old::Ident(t)) => match t.as_str() {
                "Bool" => Set::bool(),
                "Int" => Set::int(),
                "Float64" => Set::float(),
                "String" => Set::string(),
                "Any" => Set::any(),
                "Real" => return self.err("`Real` is not supported yet: use `Float64`"),
                t => match Set::int_type(t) {
                    Some(s) => s,
                    None => {
                        return self.err(format!(
                            "unknown type `{t}` (Bool, Int, Int8 to Int64, UInt8 to UInt32, \
                             Float64, String, Any)"
                        ))
                    }
                },
            },
            _ => return self.err("expected a set"),
        };
        self.pos += 1;
        Ok(set)
    }

    /// The variants of an enum, from its body, with a set for each
    /// parameter.
    pub(crate) fn variants(
        &self,
        of: &Enum,
        params: Vec<(String, Set)>,
        line: usize,
    ) -> R<(Vec<Record>, Vec<String>, Option<BTreeMap<String, Set>>)> {
        let at = |e: Error| Error::new(line, e.message);
        let mut p = P {
            toks: merge_types(lex(&of.body).map_err(at)?, &self.scoped),
            ..P::default()
        };
        let each: Vec<String> = params.iter().map(|p| p.1.to_string()).collect();
        let written = match each.is_empty() {
            true => of.name.clone(),
            false => format!("{}<{}>", of.name, each.join(", ")),
        };
        p.me = Some((of.name.clone(), written));
        p.in_enum = true;
        p.deep = self.deep;
        p.recursive = self.recursive.clone();
        let (mut closed, mut holds_itself) = (false, false);
        // The enums declared before this one.
        p.records = self.records.clone();
        p.functions = self.functions.clone();
        p.contra = self.contra;
        let before = self.enums.iter().take_while(|e| e.name != of.name);
        p.enums = before.cloned().collect();
        p.generics = params;
        let mut out = Vec::new();
        loop {
            let name = p.ident().map_err(at)?;
            p.met = false;
            p.direct = false;
            let fields = match p.is("(") {
                true => p.binders("(", ")").map_err(at)?,
                false => Vec::new(),
            };
            closed |= !p.direct;
            holds_itself |= p.met;
            if fields.is_empty()
                && p.toks
                    .get(p.pos.wrapping_sub(1))
                    .is_some_and(|t| t.text == ")")
            {
                return Err(at(Error::new(
                    0,
                    format!("a variant without fields is written `{name}`, without `()`"),
                )));
            }
            out.push(Record {
                name,
                fields,
                of: None,
                open: false,
                line,
            });
            if p.at_end() {
                return match closed {
                    false => Err(Error::new(
                        line,
                        format!(
                            "every variant of `{0}` holds a `{0}`: no value can be built",
                            of.name
                        ),
                    )),
                    true => Ok((out, p.used, holds_itself.then_some(p.recursive))),
                };
            }
            p.expect(",").map_err(at)?;
        }
    }

    /// Reads the same types twice: plainly, then deep.
    pub(crate) fn both(
        &mut self,
        read: impl Fn(&mut P) -> R<Vec<(String, Set)>>,
    ) -> R<(Vec<(String, Set)>, Vec<Set>)> {
        let start = self.pos;
        let plain = read(self)?;
        self.pos = start;
        self.deep = true;
        let deep = read(self);
        self.deep = false;
        Ok((plain, deep?.into_iter().map(|p| p.1).collect()))
    }

    /// A set read plainly, then deep.
    pub(crate) fn set_both(&mut self) -> R<(Set, Set)> {
        let both = self.both(|p| Ok(vec![(String::new(), p.set()?)]))?;
        Ok((both.0[0].1.clone(), both.1[0].clone()))
    }

    /// `(a: Int, c: Bool)`, or `()`; a record's fields between braces.
    pub(crate) fn binders(&mut self, open: &str, close: &str) -> R<Vec<(String, Set)>> {
        let mut params = Vec::new();
        self.expect(open)?;
        if self.eat(close) {
            return Ok(params);
        }
        loop {
            let name = match self.peek_at(1) == ":" {
                true => self.word()?,
                false => self.ident()?,
            };
            if !self.is(":") {
                return self.err(format!("the input `{name}` needs a type"));
            }
            self.expect(":")?;
            // `o: &Mass`, as Rust writes it: a borrowed value is the same
            // value here.
            if self.eat("&") {
                self.eat("mut");
            }
            let before = std::mem::take(&mut self.met);
            let set = self.set()?;
            // The type itself, alone: a value would need a value.
            let mark = self
                .me
                .as_ref()
                .map(|me| Set::record(&format!("${}", me.1), Vec::new()));
            self.direct |= self.met && (set == Set::any() || Some(&set) == mark.as_ref());
            self.met |= before;
            params.push((name, set));
            if !self.eat(",") || self.is(close) {
                self.expect(close)?;
                return Ok(params);
            }
        }
    }

    /// An enum, from its name, its parameters and its variants as
    /// `Name(field: T, …), …`.
    pub(crate) fn enumeration(
        &mut self,
        name: String,
        params: Vec<String>,
        body: String,
        line: usize,
    ) -> R<()> {
        if self.used.contains(&name) {
            return Err(Error::new(
                line,
                format!("`{name}` is used above: declare an enum before its first use"),
            ));
        }
        // Checked once with every parameter as `Value`.
        let any = params.iter().map(|x| (x.clone(), Set::any())).collect();
        let mut e = Enum {
            name: name.clone(),
            params,
            body,
            variants: Vec::new(),
            line,
        };
        let (variants, used, _) = self.variants(&e, any, line)?;
        self.used.extend(used);
        e.variants = variants.iter().map(|v| v.name.clone()).collect();
        for mut v in variants {
            v.of = Some(name.clone());
            self.variants.insert(v.name.clone(), v.fields.len());
            self.records.push(v.clone());
            self.m.records.push(v);
        }
        self.m.declared.push(Decl::Enum(self.m.enums.len()));
        self.enums.push(e.clone());
        self.m.enums.push(e);
        Ok(())
    }
}
