//! Model to text (`docs/design/syntax.md`): what `src/parse.rs` reads.
//!
//! One way to write each model: four spaces a level, a part's requires,
//! then records, functions and ensures, each tie in the block of what it
//! ties, a path as short as it reads back the same. Comments are not
//! kept: they are not in the model.
//!
//! A model whose names are words (M001) and whose expressions are made
//! as their operations take them is written so that `parse` gives it
//! back, ties aside: a tie of a part, record or function is written in
//! its block, any other at the end, `tie a::b at "…";`. A name a proof
//! gives a proposition that is no path, as a tie's `stone::drop at
//! lab/tower`, cannot be written; nor an operation applied to other
//! arguments than it takes, written `__malformed__`.

use crate::expression::{Arg, Expression};
use crate::judgement::{self, Prop, Set};
use crate::location::Location;
use crate::model::{
    Contract, Ensure, Function, Implements, Kind, Model, ModelExample, Part, Path, Record, Require,
    Rule, Source, Term, Tie, Use,
};
use crate::parse::Calling;
use crate::simulation::{Bound, Op, Rational, Type, Value};
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use std::collections::{BTreeMap, BTreeSet};

/// The text of a model.
pub fn print(model: &Model) -> String {
    let items = items(model);
    let calling = Calling::of(model);
    let mut placed = vec![false; model.ties.len()];
    let mut blocks: Vec<String> = vec![];
    for c in &model.contracts {
        let w = Writer {
            part: None,
            items: &items,
            calling: &calling,
            contract: Some(&c.name),
        };
        blocks.push(w.contract(c));
    }
    let mut seen_parts = BTreeSet::new();
    let nested: BTreeSet<&String> = model.parts.iter().flat_map(|p| &p.inner).collect();
    for part in &model.parts {
        // A nested part is written inside the part that holds it.
        if nested.contains(&part.name) {
            continue;
        }
        let first = seen_parts.insert(part.name.clone());
        let w = Writer {
            part: Some(&part.name),
            items: &items,
            calling: &calling,
            contract: None,
        };
        let mut out = String::new();
        w.part(part, model, &mut placed, first, &mut out);
        // A part read from another file stays there: its ties too.
        if part.file.is_none() {
            blocks.push(out);
        }
    }
    let loose: Vec<&ModelExample> = model
        .examples
        .iter()
        .filter(|e| e.within.is_none())
        .collect();
    if !loose.is_empty() {
        let w = Writer {
            part: None,
            items: &items,
            calling: &calling,
            contract: None,
        };
        let rows: Vec<String> = loose.iter().map(|e| w.model_example(e)).collect();
        blocks.push(format!(
            "examples {{\n{}}}\n",
            rows.iter()
                .map(|r| format!("    {r},\n"))
                .collect::<String>()
        ));
    }
    let mut out = blocks.join("\n");
    let loose: Vec<&Tie> = model
        .ties
        .iter()
        .zip(&placed)
        .filter(|(_, p)| !**p)
        .map(|(t, _)| t)
        .collect();
    if !loose.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        for tie in loose {
            let item = tie.item.names.join("::");
            out.push_str(&format!("tie {item} at {};\n", location(&tie.at)));
        }
    }
    out
}

/// A proposition, its paths whole, as a store of reviews writes it.
pub fn print_prop(prop: &Prop) -> String {
    let items = BTreeMap::new();
    let calling = Calling::default();
    Writer {
        part: None,
        items: &items,
        calling: &calling,
        contract: None,
    }
    .prop(prop)
}

/// A proof, its paths whole, as an error that lists proofs writes it.
pub fn print_proof(proof: &crate::expression::Expression<judgement::Op>) -> String {
    let items = BTreeMap::new();
    let calling = Calling::default();
    Writer {
        part: None,
        items: &items,
        calling: &calling,
        contract: None,
    }
    .proof(proof, &mut vec![])
    .0
}

/// A row, `(6, 2) -> 3`.
pub fn print_row(takes: &[Value], gives: &Value) -> String {
    format!("{} -> {}", values(takes), value(gives))
}

/// The names of each part: its records, functions, requires and ensures.
fn items(model: &Model) -> BTreeMap<String, BTreeSet<String>> {
    let mut items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for part in &model.parts {
        let names = items.entry(part.name.clone()).or_default();
        names.extend(part.records.iter().map(|r| r.name.clone()));
        names.extend(part.functions.iter().map(|f| f.name.clone()));
        names.extend(part.requires.iter().map(|q| q.name.clone()));
        names.extend(part.ensures.iter().map(|e| e.name.clone()));
    }
    items
}

/// `"lab/tower/stone"`, escaped as Rust escapes a string.
fn location(at: &Location) -> String {
    string(&at.path.join("/"))
}

fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What a part reads paths from.
struct Writer<'a> {
    part: Option<&'a str>,
    items: &'a BTreeMap<String, BTreeSet<String>>,
    calling: &'a Calling,
    contract: Option<&'a str>,
}

const MALFORMED: &str = "__malformed__";

// Precedence, from the loosest.
const LET: u8 = 0;
const OR: u8 = 1;
const AND: u8 = 2;
const CMP: u8 = 3;
const SUM: u8 = 4;
const PRODUCT: u8 = 5;
const UNARY: u8 = 6;
const POSTFIX: u8 = 7;
const ATOM: u8 = 8;

fn wrap((s, p): (String, u8), at_least: u8) -> String {
    match p < at_least {
        true => format!("({s})"),
        false => s,
    }
}

impl Writer<'_> {
    fn cx(&self) -> Cx<'_> {
        Cx {
            calling: self.calling,
            part: self.part,
            contract: self.contract,
        }
    }

    fn contract(&self, c: &Contract) -> String {
        let params: Vec<String> = c
            .params
            .iter()
            .filter(|(n, _)| n != "Self")
            .map(|(n, b)| match b {
                Bound::Any => n.clone(),
                Bound::Number => format!("{n}: Number"),
            })
            .collect();
        let params = match params.is_empty() {
            true => String::new(),
            false => format!("<{}>", params.join(", ")),
        };
        let mut s = format!("contract {}{params} {{\n", c.name);
        for f in &c.functions {
            s.push_str(&format!(
                "    fn {}({}) -> {}: {};\n",
                f.name,
                takes(&f.takes),
                f.gives.0,
                type_(&f.gives.1)
            ));
        }
        for l in &c.claims {
            s.push_str(&format!(
                "    claim {}({}): {};\n",
                l.name,
                takes(&l.over),
                sim_in(&self.cx(), &l.holds)
            ));
        }
        s.push_str("}\n");
        s
    }

    /// `(sun::light::w = 850.0, stone::speed::h = 5.0) -> stone::speed::v2 = 98.1`.
    fn model_example(&self, e: &ModelExample) -> String {
        let port = |(p, v): &(Path, Value)| format!("{} = {}", self.path_of(p), value(v));
        let inputs: Vec<String> = e.inputs.iter().map(port).collect();
        let outputs: Vec<String> = e.outputs.iter().map(port).collect();
        let outputs = match outputs.len() {
            1 => outputs[0].clone(),
            _ => format!("({})", outputs.join(", ")),
        };
        let given = match e.given.0.is_empty() {
            true => String::new(),
            false => format!("{} ", givens(&e.given).trim_start()),
        };
        match &e.when {
            Some((f, args, gives)) => format!(
                "{given}when {}{} then {}",
                self.path_of(f),
                values(args),
                value(gives)
            ),
            None => format!("{given}({}) -> {outputs}", inputs.join(", ")),
        }
    }

    fn local(&self, name: &str) -> bool {
        self.part
            .and_then(|p| self.items.get(p))
            .is_some_and(|items| items.contains(name))
    }

    /// As short as it reads back the same: from the part when it is in
    /// it, `crate::` first when a whole path would read as one from the part.
    fn path(&self, names: &[String], bound: &[String]) -> String {
        if names.is_empty() {
            return MALFORMED.to_string();
        }
        if let Some(part) = self.part {
            if names.len() >= 2 && names[0] == part && self.local(&names[1]) {
                let short = &names[1..];
                if !(short.len() == 1 && bound.contains(&short[0])) {
                    return short.join("::");
                }
            }
        }
        let clash = self.local(&names[0])
            || names[0] == "crate"
            || (names.len() == 1 && bound.contains(&names[0]));
        match clash {
            true => format!("crate::{}", names.join("::")),
            false => names.join("::"),
        }
    }
    fn path_of(&self, p: &Path) -> String {
        self.path(&p.names, &[])
    }
    /// A name as a proof or proposition writes it, `stone::drop::falls`.
    fn named(&self, name: &str, bound: &[String]) -> String {
        if bound.iter().any(|b| b == name) {
            return name.to_string();
        }
        let names: Vec<String> = name.split("::").map(str::to_string).collect();
        self.path(&names, bound)
    }

    fn ties(&self, item: &Path, model: &Model, placed: &mut [bool]) -> String {
        let mut at = vec![];
        for (i, tie) in model.ties.iter().enumerate() {
            if !placed[i] && tie.item == *item {
                placed[i] = true;
                at.push(location(&tie.at));
            }
        }
        match at.is_empty() {
            true => String::new(),
            false => format!(" at {}", at.join(", ")),
        }
    }

    fn part(&self, part: &Part, model: &Model, placed: &mut [bool], first: bool, out: &mut String) {
        let kind = match part.kind {
            Kind::Subject => "subject",
            Kind::Environment => "environment",
        };
        let here = Path::to(&[&part.name]);
        let tied = |item: &Path, placed: &mut [bool]| match first {
            true => self.ties(item, model, placed),
            false => String::new(),
        };
        // A part that is its data only: `subject record Reply { … }`.
        let data_only = part.uses.is_empty()
            && part.functions.is_empty()
            && part.implements.is_empty()
            && part.requires.is_empty()
            && part.ensures.is_empty()
            && part.inner.is_empty()
            && model.ties.iter().all(|t| t.item != here);
        match (&part.records[..], &part.enums[..]) {
            ([r], []) if data_only && r.name == part.name => {
                let ties = tied(&here.then(&r.name), placed);
                let s = self.record(r, &ties, false);
                out.push_str(&format!("{kind} {}", s.trim_start()));
                return;
            }
            ([], [e]) if data_only && e.name == part.name => {
                let ties = tied(&here.then(&e.name), placed);
                out.push_str(&format!("{kind} {}", enum_(e, &ties).trim_start()));
                return;
            }
            _ => {}
        }
        out.push_str(&format!("{kind} {}{} {{\n", part.name, tied(&here, placed)));
        let mut blocks: Vec<String> = vec![];
        let uses: String = part.uses.iter().map(use_).collect();
        if !uses.is_empty() {
            blocks.push(uses);
        }
        let requires: String = part.requires.iter().map(|q| self.require(q)).collect();
        if !requires.is_empty() {
            blocks.push(requires);
        }
        let mut seen = BTreeSet::new();
        for r in &part.records {
            let ties = match seen.insert(r.name.clone()) {
                true => tied(&here.then(&r.name), placed),
                false => String::new(),
            };
            let own = r.name == part.name && r.params.is_empty();
            blocks.push(self.record(r, &ties, own));
        }
        for e in &part.enums {
            let ties = match seen.insert(e.name.clone()) {
                true => tied(&here.then(&e.name), placed),
                false => String::new(),
            };
            blocks.push(enum_(e, &ties));
        }
        for f in &part.functions {
            let ties = match seen.insert(f.name.clone()) {
                true => tied(&here.then(&f.name), placed),
                false => String::new(),
            };
            blocks.push(self.function(f, &ties));
        }
        let impls: String = part
            .implements
            .iter()
            .map(|i| implements(i, part, model))
            .collect();
        if !impls.is_empty() {
            blocks.push(impls);
        }
        let ensures: String = part.ensures.iter().map(|e| self.ensure(e)).collect();
        if !ensures.is_empty() {
            blocks.push(ensures);
        }
        // Its examples of the whole, then its parts, one level in.
        let rows: Vec<String> = model
            .examples
            .iter()
            .filter(|e| e.within.as_deref() == Some(part.name.as_str()))
            .map(|e| format!("        {},\n", self.model_example(e)))
            .collect();
        if !rows.is_empty() {
            blocks.push(format!("    examples {{\n{}    }}\n", rows.concat()));
        }
        for name in &part.inner {
            if let Some(inner) = model.parts.iter().find(|p| &p.name == name) {
                let w = Writer {
                    part: Some(&inner.name),
                    ..*self
                };
                let mut s = String::new();
                w.part(inner, model, placed, first, &mut s);
                let indented: String = s
                    .lines()
                    .map(|l| match l.is_empty() {
                        true => "\n".to_string(),
                        false => format!("    {l}\n"),
                    })
                    .collect();
                blocks.push(indented);
            }
        }
        out.push_str(&blocks.join("\n"));
        out.push_str("}\n");
    }

    fn require(&self, q: &Require) -> String {
        let from = match &q.from {
            Source::Prose(s) => string(s),
            Source::Ensure(p) => self.path_of(p),
        };
        format!("    require {}: {from};\n", q.name)
    }

    fn ensure(&self, e: &Ensure) -> String {
        let mut s = format!("    ensure {}: {}", e.name, string(&e.prose));
        if !e.from.is_empty() {
            let lists: Vec<String> = e
                .from
                .iter()
                .map(|l| {
                    format!(
                        "({})",
                        l.iter()
                            .map(|p| self.path_of(p))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();
            s.push_str(&format!(" from {}", lists.join(" | ")));
        }
        s.push_str(";\n");
        s
    }

    fn record(&self, r: &Record, ties: &str, own: bool) -> String {
        // The part's own record has no name of its own.
        let mut s = match own {
            true => format!("    record{ties}"),
            false => format!("    record {}{}{ties}", r.name, params(&r.params)),
        };
        if let Some(fields) = &r.fields {
            let fields: Vec<String> = fields
                .iter()
                .map(|(n, t)| match n == crate::model::REST {
                    true => n.clone(),
                    false => format!("{n}: {}", type_(t)),
                })
                .collect();
            match fields.is_empty() {
                true => s.push_str(" {}"),
                false => s.push_str(&format!(" {{ {} }}", fields.join(", "))),
            }
        }
        let said = !r.claims.is_empty() || !r.rules.is_empty();
        match (said, r.fields.is_some()) {
            (false, true) => s.push('\n'),
            (false, false) => s.push_str(";\n"),
            (true, _) => s.push_str(&self.where_(&r.claims, &r.rules, &[])),
        }
        s
    }

    fn function(&self, f: &Function, ties: &str) -> String {
        let takes: Vec<String> = f
            .takes
            .iter()
            .enumerate()
            .map(|(k, (n, t))| match f.ranges.takes.get(n) {
                _ if k == 0 && n == "self" => "self".to_string(),
                Some(range) => format!("{n}: {range}"),
                None => format!("{n}: {}", type_(t)),
            })
            .collect();
        let gives = match &f.ranges.gives {
            Some((_, range)) => range.to_string(),
            None => type_(&f.gives.1),
        };
        let mut s = format!(
            "    fn {}({}) -> {}: {gives}{ties}",
            f.name,
            takes.join(", "),
            f.gives.0,
        );
        if let Some(body) = &f.body {
            s.push_str(&format!(" {{ {} }}", sim_in(&self.cx(), body)));
        }
        let said = !f.claims.is_empty() || !f.rules.is_empty() || !f.examples.is_empty();
        let rows: Vec<String> = f
            .examples
            .iter()
            .map(|e| match e.given.0.is_empty() {
                true => print_row(&e.takes, &e.gives),
                false => format!(
                    "{} when {} then {}",
                    givens(&e.given).trim_start(),
                    values(&e.takes),
                    value(&e.gives)
                ),
            })
            .collect();
        match (said, f.body.is_some()) {
            (false, true) => s.push('\n'),
            (false, false) => s.push_str(";\n"),
            (true, _) => s.push_str(&self.where_(&f.claims, &f.rules, &rows)),
        }
        s
    }

    fn where_(&self, claims: &[crate::model::Claim], rules: &[Rule], rows: &[String]) -> String {
        let mut s = String::from(" where {\n");
        for c in claims {
            let by = match (&c.induction, &c.invariant) {
                (Some(t), _) => format!(" by induction {t}"),
                (None, Some(crate::model::Invariant { turns: Some(n) })) => {
                    format!(" by invariant {n}")
                }
                (None, Some(_)) => " by invariant".to_string(),
                (None, None) => String::new(),
            };
            s.push_str(&format!(
                "        claim {}: {}{by};\n",
                c.name,
                sim_in(&self.cx(), c.kept())
            ));
        }
        for r in rules {
            s.push_str(&self.rule(r));
        }
        if !rows.is_empty() {
            let one = format!("        examples {{ {} }}\n", rows.join(", "));
            match one.len() <= 100 {
                true => s.push_str(&one),
                false => {
                    s.push_str("        examples {\n");
                    for row in rows {
                        s.push_str(&format!("            {row},\n"));
                    }
                    s.push_str("        }\n");
                }
            }
        }
        s.push_str("    }\n");
        s
    }

    fn term(&self, t: &Term) -> String {
        match t {
            Term::At(p) => self.path_of(p),
            Term::Holds(e) => format!("{{ {} }}", sim_in(&self.cx(), e)),
            // A proposition that would read as a path or a block is in parentheses.
            Term::Prop(p @ (Prop::Named(_) | Prop::Implies { .. })) => {
                format!("({})", self.prop(p))
            }
            Term::Prop(p) => self.prop(p),
        }
    }

    fn rule(&self, r: &Rule) -> String {
        let premises: Vec<String> = r.premises.iter().map(|t| self.term(t)).collect();
        let premises = match premises.is_empty() {
            true => String::new(),
            false => format!("{} ", premises.join(", ")),
        };
        let mut s = format!(
            "        rule {}: {premises}|- {}",
            r.name,
            self.term(&r.conclusion)
        );
        if let Some(proof) = &r.proof {
            s.push_str(&format!(
                "\n            by {}",
                self.proof(proof, &mut vec![]).0
            ));
        } else if let Some(step) = r.short {
            s.push_str(&format!(" by {}", step.word()));
        }
        s.push_str(";\n");
        s
    }

    // ------------------------------------------------ proofs

    fn proof(&self, e: &Expression<judgement::Op>, bound: &mut Vec<String>) -> (String, u8) {
        use judgement::Op as O;
        let (op, args) = match e {
            Expression::Name(n) => {
                let written = match n.parse::<usize>() {
                    Ok(_) => n.clone(),
                    Err(_) => self.named(n, bound),
                };
                return (written, ATOM);
            }
            Expression::Apply(op, args) => (op, args),
        };
        if let O::Let = op {
            return match &args[..] {
                [Arg { binds: b0, body: a }, Arg { binds: b1, body: b }]
                    if b0.is_empty() && b1.len() == 1 =>
                {
                    let a = self.proof(a, bound).0;
                    bound.push(b1[0].clone());
                    let b = self.proof(b, bound).0;
                    bound.pop();
                    (format!("let {} = {a} in {b}", b1[0]), LET)
                }
                _ => (MALFORMED.to_string(), ATOM),
            };
        }
        // `intro[q](|x| p)`.
        if let O::Intro { q } = op {
            return match &args[..] {
                [Arg { binds, body }] if binds.len() == 1 => {
                    bound.push(binds[0].clone());
                    let b = self.proof(body, bound).0;
                    bound.pop();
                    (format!("intro[{}](|{}| {b})", self.prop(q), binds[0]), ATOM)
                }
                _ => (MALFORMED.to_string(), ATOM),
            };
        }
        if args.iter().any(|a| !a.binds.is_empty()) {
            return (MALFORMED.to_string(), ATOM);
        }
        let mut each = |at_least: u8| -> Vec<String> {
            args.iter()
                .map(|a| wrap(self.proof(&a.body, bound), at_least))
                .collect()
        };
        match op {
            O::Not if args.len() == 1 => (format!("!{}", each(UNARY)[0]), UNARY),
            // `state(…)` and `&&` on states have no text (finding 27).
            O::Not | O::Let | O::And | O::StateOf => (MALFORMED.to_string(), ATOM),
            O::Join => (format!("join({})", each(LET).join(", ")), ATOM),
            O::Ponens { t, post } => {
                let sugar = matches!((t, args.first()), (Prop::Named(r), Some(Arg { body: Expression::Name(n), .. })) if r == n)
                    && *post == 0;
                let args = each(LET).join(", ");
                match (sugar, post) {
                    (true, _) => (format!("ponens({args})"), ATOM),
                    (false, 0) => (format!("ponens[{}]({args})", self.prop(t)), ATOM),
                    (false, _) => (format!("ponens[{}, {post}]({args})", self.prop(t)), ATOM),
                }
            }
            O::Tollens { t, post, blame } => {
                let args = each(LET).join(", ");
                (
                    format!("tollens[{}, {post}, {blame}]({args})", self.prop(t)),
                    ATOM,
                )
            }
            O::Reductio { of } => (
                format!("reductio[{}]({})", self.prop(of), each(LET).join(", ")),
                ATOM,
            ),
            O::Instance { q } => (
                format!("instance[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
            O::Witness { q } => (
                format!("witness[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
            O::Point(at) if args.is_empty() => (format!("point{}", values(at)), ATOM),
            O::Given(at, w) if args.is_empty() => {
                (format!("point{}{}", values(at), world(w)), ATOM)
            }
            O::Point(_) | O::Given(..) | O::Intro { .. } => (MALFORMED.to_string(), ATOM),
            O::Cases { q } => (
                format!("cases[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
        }
    }

    // ------------------------------------------------ propositions

    fn prop(&self, p: &Prop) -> String {
        self.prop_at(p).0
    }
    fn prop_at(&self, p: &Prop) -> (String, u8) {
        match p {
            Prop::Named(n) => (self.named(n, &[]), ATOM),
            Prop::Not(x) => (format!("!{}", wrap(self.prop_at(x), ATOM)), UNARY),
            Prop::Implies { pre, subject, post } => {
                let list = |ps: &[Prop]| {
                    ps.iter()
                        .map(|p| self.prop(p))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                (
                    format!(
                        "{{{}}} {} {{{}}}",
                        list(pre),
                        self.named(subject, &[]),
                        list(post)
                    ),
                    ATOM,
                )
            }
            Prop::Correct { function, at } => {
                let at = match at {
                    judgement::Term::Point(vs) => values(vs),
                    judgement::Term::Given { point, world: w } => {
                        format!("{}{}", values(point), world(w))
                    }
                    judgement::Term::Var(x) => x.clone(),
                    judgement::Term::Arbitrary { name, over } => {
                        format!("{name} in {}", set(&self.cx(), over))
                    }
                };
                (
                    format!("correct({}, {at})", self.named(function, &[])),
                    ATOM,
                )
            }
            Prop::All { name, over, body } => (
                format!(
                    "all {name} in {}: {}",
                    set(&self.cx(), over),
                    self.prop(body)
                ),
                LET,
            ),
            Prop::Any { name, over, body } => (
                format!(
                    "any {name} in {}: {}",
                    set(&self.cx(), over),
                    self.prop(body)
                ),
                LET,
            ),
        }
    }
}

/// `use sun::{light, Ray};`, `use sun::light;`, `use sun::*;`.
/// `enum Option<T> { None, Some(T) }`.
fn enum_(e: &crate::model::Enum, ties: &str) -> String {
    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|(v, ts)| match ts.is_empty() {
            true => v.clone(),
            false => format!(
                "{v}({})",
                ts.iter().map(type_).collect::<Vec<_>>().join(", ")
            ),
        })
        .collect();
    format!(
        "    enum {}{}{ties} {{ {} }}\n",
        e.name,
        params(&e.params),
        variants.join(", ")
    )
}

/// A use, with the path its `from` names as written.
fn use_(u: &Use) -> String {
    let from = match u.from() {
        Some(path) => format!(" from {}", string(path)),
        None => String::new(),
    };
    match u {
        Use::Glob { part, .. } => format!("    use {part}::*{from};\n"),
        Use::Names { part, names, .. } if names.len() == 1 && names[0] == *part => {
            format!("    use {part}{from};\n")
        }
        Use::Names { part, names, .. } if names.len() == 1 => {
            format!("    use {part}::{}{from};\n", names[0])
        }
        Use::Names { part, names, .. } => {
            format!("    use {part}::{{{}}}{from};\n", names.join(", "))
        }
    }
}

/// `impl Monotone<Real> for speed;`, `impl Pair<Real> for (f, g);`.
fn implements(i: &Implements, part: &Part, model: &Model) -> String {
    // `impl Callable;`: the part is `Self`, its functions the contract's.
    let own = model
        .contracts
        .iter()
        .find(|c| c.name == i.contract)
        .is_some_and(|c| {
            c.params.first().is_some_and(|(n, _)| n == "Self")
                && i.types
                    .first()
                    .is_some_and(|t| matches!(t, Type::Record(r) if r.name == part.name))
                && c.functions.iter().map(|f| &f.name).eq(i.functions.iter())
        });
    if own {
        let rest = &i.types[1..];
        let types = match rest.is_empty() {
            true => String::new(),
            false => format!(
                "<{}>",
                rest.iter().map(type_).collect::<Vec<_>>().join(", ")
            ),
        };
        return format!("    impl {}{types};\n", i.contract);
    }
    let types = match i.types.is_empty() {
        true => String::new(),
        false => format!(
            "<{}>",
            i.types.iter().map(type_).collect::<Vec<_>>().join(", ")
        ),
    };
    let functions = match &i.functions[..] {
        [one] => one.clone(),
        many => format!("({})", many.join(", ")),
    };
    format!("    impl {}{types} for {functions};\n", i.contract)
}

/// `x: Real, y: Int`.
fn takes(ts: &[(String, Type)]) -> String {
    ts.iter()
        .enumerate()
        .map(|(k, (n, t))| match (k, n.as_str()) {
            (0, "self") => "self".to_string(),
            _ => format!("{n}: {}", type_(t)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// ` given sun::light() -> 2.0, …`: what the environment gave, in
/// order; nothing when it gave nothing.
pub fn world(w: &judgement::World) -> String {
    if w.0.is_empty() {
        return String::new();
    }
    let rows: Vec<String> =
        w.0.iter()
            .map(|(f, args, v)| format!("{f}{} -> {}", values(args), value(v)))
            .collect();
    format!(" given {}", rows.join(", "))
}

/// ` given llm::inference(_) -> "hello", …`: what the environment gave,
/// as an example says it; nothing when it gave nothing.
pub fn givens(w: &crate::model::Given) -> String {
    if w.0.is_empty() {
        return String::new();
    }
    let rows: Vec<String> =
        w.0.iter()
            .map(|(f, args, v)| {
                let args: Vec<String> = args
                    .iter()
                    .map(|a| a.as_ref().map_or("_".to_string(), value))
                    .collect();
                format!("{f}({}) -> {}", args.join(", "), value(v))
            })
            .collect();
    format!(" given {}", rows.join(", "))
}

/// `(a: 0..10) where a > 5`, as written.
pub fn print_set(s: &Set) -> String {
    let calling = Calling::default();
    let cx = Cx {
        calling: &calling,
        part: None,
        contract: None,
    };
    set(&cx, s)
}

fn set(cx: &Cx, s: &Set) -> String {
    let ranges: Vec<String> = s
        .ranges
        .iter()
        .map(|(n, lo, hi)| format!("{n}: {}..{}", value(lo), value(hi)))
        .collect();
    let mut out = format!("({})", ranges.join(", "));
    if let Some(such) = &s.such {
        out.push_str(&format!(" where {}", sim_in(cx, such)));
    }
    out
}

// ---------------------------------------------------------------- types and values

/// `bool`, `integer`, `rational`, `(A, B)`, or a record's name.
pub fn type_(t: &Type) -> String {
    match t {
        Type::Truth => "bool".to_string(),
        Type::Int => "integer".to_string(),
        Type::Real => "rational".to_string(),
        Type::Record(r) if r.labelled() => {
            let types: Vec<String> = r
                .fields
                .iter()
                .map(|(n, t)| format!("{n}: {}", type_(t)))
                .collect();
            format!("({})", types.join(", "))
        }
        Type::Record(r) if r.name.is_empty() => {
            let types: Vec<String> = r.fields.iter().map(|(_, t)| type_(t)).collect();
            format!("({})", types.join(", "))
        }
        Type::Record(r) => generic(&r.name, &r.args),
        Type::Enum(e) => generic(&e.name, &e.args),
        Type::Seq(t) => format!("[{}]", type_(t)),
        Type::Encoded(t) => format!("[~{}]", type_(t)),
        Type::Char => "char".to_string(),
        Type::Byte => "byte".to_string(),
        Type::Param { name, .. } => name.clone(),
    }
}

/// `Pair`, or `Pair<Int, Real>`.
fn generic(name: &str, args: &[Type]) -> String {
    match args.is_empty() {
        true => name.to_string(),
        false => format!(
            "{name}<{}>",
            args.iter().map(type_).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// `<T>`, `<T: Number, U>`: the parameters a declaration names.
fn params(ps: &[(String, crate::simulation::Bound)]) -> String {
    if ps.is_empty() {
        return String::new();
    }
    let ps: Vec<String> = ps
        .iter()
        .map(|(n, b)| match b {
            crate::simulation::Bound::Number => format!("{n}: Number"),
            crate::simulation::Bound::Any => n.clone(),
        })
        .collect();
    format!("<{}>", ps.join(", "))
}

/// `9.81`, `2.0`, `-0.5`; a ratio no decimal writes, `1/3`.
pub fn real(r: &Rational) -> String {
    let (numer, denom) = (r.numer(), r.denom());
    let (mut twos, mut fives, mut rest) = (0u32, 0u32, denom.clone());
    let (two, five) = (BigInt::from(2), BigInt::from(5));
    while !rest.is_zero() && (&rest % &two).is_zero() {
        rest /= &two;
        twos += 1;
    }
    while !rest.is_zero() && (&rest % &five).is_zero() {
        rest /= &five;
        fives += 1;
    }
    if !rest.is_one() {
        return format!("{numer}/{denom}");
    }
    let k = twos.max(fives);
    let scaled = numer.abs() * BigInt::from(10).pow(k) / denom;
    let mut digits = scaled.to_string();
    while digits.len() <= k as usize {
        digits.insert(0, '0');
    }
    let point = digits.len() - k as usize;
    let (whole, frac) = digits.split_at(point);
    let frac = match frac.is_empty() {
        true => "0",
        false => frac,
    };
    let sign = match numer.is_negative() {
        true => "-",
        false => "",
    };
    format!("{sign}{whole}.{frac}")
}

pub fn value(v: &Value) -> String {
    match v {
        Value::Truth(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Real(r) => real(r),
        Value::Record(r, vs) if r.labelled() => {
            let fields: Vec<String> = r
                .fields
                .iter()
                .zip(vs)
                .map(|((n, _), v)| format!("{n}: {}", value(v)))
                .collect();
            format!("({})", fields.join(", "))
        }
        Value::Record(r, vs) if r.name.is_empty() => values(vs),
        Value::Record(r, vs) => {
            let fields: Vec<String> = r
                .fields
                .iter()
                .zip(vs)
                .map(|((n, _), v)| format!("{n}: {}", value(v)))
                .collect();
            format!("{} {{ {} }}", r.name, fields.join(", "))
        }
        // Rust's struct pattern: the fields not known left to `..`.
        Value::Partial(r, vs) => {
            let mut fields: Vec<String> = r
                .fields
                .iter()
                .zip(vs)
                .filter_map(|((n, _), v)| v.as_ref().map(|v| format!("{n}: {}", value(v))))
                .collect();
            fields.push("..".to_string());
            format!("{} {{ {} }}", r.name, fields.join(", "))
        }
        Value::Seq(_, holds) => {
            let holds: Vec<String> = holds.iter().map(value).collect();
            format!("[{}]", holds.join(", "))
        }
        Value::Text(s) => string(s),
        // Rust's byte literal, `b'A'`, `b'\xff'`: shown only, none is
        // written in a model yet.
        Value::Byte(b) => format!("b'{}'", std::ascii::escape_default(*b)),
        Value::Variant(e, i, holds) => {
            let name = e.variants.get(*i).map(|(n, _)| n.as_str()).unwrap_or("?");
            match holds.is_empty() {
                true => name.to_string(),
                false => format!("{name}{}", values(holds)),
            }
        }
    }
}

/// `(6, 2)`: a list of values, in parentheses.
pub fn values(vs: &[Value]) -> String {
    format!("({})", vs.iter().map(value).collect::<Vec<_>>().join(", "))
}

// ---------------------------------------------------------------- simulation

/// An expression of simulation, as Rust writes it, `let x = a in b` aside.
pub fn sim(e: &Expression<Op>) -> String {
    let calling = Calling::default();
    sim_in(
        &Cx {
            calling: &calling,
            part: None,
            contract: None,
        },
        e,
    )
}

/// An expression of simulation as written in `part` or `contract`: a
/// call by its name alone when that reads back the same.
pub(crate) fn sim_from(
    calling: &Calling,
    part: Option<&str>,
    contract: Option<&str>,
    e: &Expression<Op>,
) -> String {
    sim_in(
        &Cx {
            calling,
            part,
            contract,
        },
        e,
    )
}

fn sim_in(cx: &Cx, e: &Expression<Op>) -> String {
    sim_at(cx, e).0
}

/// What a call reads names from.
struct Cx<'a> {
    calling: &'a Calling,
    part: Option<&'a str>,
    contract: Option<&'a str>,
}

impl Cx<'_> {
    /// A call's function by its name alone when that reads back the
    /// same, else whole.
    fn call(&self, full: &str) -> String {
        if let Some((_, last)) = full.rsplit_once("::") {
            if self
                .calling
                .resolve(self.part, self.contract, last)
                .as_deref()
                == Some(full)
            {
                return last.to_string();
            }
        }
        full.to_string()
    }
}

/// Whether `e` makes a named record anywhere: then it is put in
/// parentheses as the condition of an `if`, as Rust asks.
fn makes_record(e: &Expression<Op>) -> bool {
    match e {
        Expression::Name(_) => false,
        Expression::Apply(op, args) => {
            matches!(op, Op::Make(r) if !r.name.is_empty())
                || args.iter().any(|a| makes_record(&a.body))
        }
    }
}

/// What a loop starts from, as written: `sum: 0` for one value, else
/// the labelled tuple, its labels those the loop binds.
fn carried(cx: &Cx, labels: &[String], start: &Expression<Op>) -> Option<String> {
    match (labels, start) {
        ([label], _) => Some(format!("{label}: {}", sim_in(cx, start))),
        (_, Expression::Apply(Op::Make(r), _))
            if r.labelled() && r.fields.iter().map(|(f, _)| f).eq(labels.iter()) =>
        {
            Some(sim_in(cx, start))
        }
        _ => None,
    }
}

fn sim_at(cx: &Cx, e: &Expression<Op>) -> (String, u8) {
    let (op, args) = match e {
        Expression::Name(n) => return (n.clone(), ATOM),
        Expression::Apply(op, args) => (op, args),
    };
    let bad = (MALFORMED.to_string(), ATOM);
    let plain = args.iter().all(|a| a.binds.is_empty());
    let body = |i: usize| &args[i].body;
    let binary = |symbol: &str, p: u8, left: u8, right: u8| {
        (
            format!(
                "{} {symbol} {}",
                wrap(sim_at(cx, body(0)), left),
                wrap(sim_at(cx, body(1)), right)
            ),
            p,
        )
    };
    match (op, args.len(), plain) {
        (Op::Let, 2, _) => match (&args[0].binds[..], &args[1].binds[..]) {
            ([], [x]) => (
                format!(
                    "let {x} = {} in {}",
                    sim_in(cx, body(0)),
                    sim_in(cx, body(1))
                ),
                LET,
            ),
            _ => bad,
        },
        // `xs.fold(sum: 0) { x -> sum + x }`, `xs.fold((sum: 0, n: 0)) { … }`.
        (Op::Fold, 3, _) => match (&args[0].binds[..], &args[1].binds[..], &args[2].binds[..]) {
            ([], [], [x, labels @ ..]) if !labels.is_empty() => {
                let Some(start) = carried(cx, labels, body(1)) else {
                    return bad;
                };
                (
                    format!(
                        "{}.fold({start}) {{ {x} -> {} }}",
                        wrap(sim_at(cx, body(0)), POSTFIX),
                        sim_in(cx, body(2))
                    ),
                    POSTFIX,
                )
            }
            _ => bad,
        },
        // `until(r: g, r.stop) { next(r) }`.
        (Op::Until, 3, _) => match (&args[0].binds[..], &args[1].binds[..], &args[2].binds[..]) {
            ([], labels, again) if !labels.is_empty() && labels == again => {
                let Some(start) = carried(cx, labels, body(0)) else {
                    return bad;
                };
                (
                    format!(
                        "until({start}, {}) {{ {} }}",
                        sim_in(cx, body(1)),
                        sim_in(cx, body(2))
                    ),
                    ATOM,
                )
            }
            _ => bad,
        },
        (Op::All, 3, _) => match (&args[0].binds[..], &args[1].binds[..], &args[2].binds[..]) {
            ([], [], [x]) => (
                format!(
                    "({}..{}).all(|{x}| {})",
                    sim_in(cx, body(0)),
                    sim_in(cx, body(1)),
                    sim_in(cx, body(2))
                ),
                POSTFIX,
            ),
            _ => bad,
        },
        // `match e { None => a, Some(x) => b }`: every variant, in order.
        (Op::Match(en), n, _) if n == en.variants.len() + 1 && args[0].binds.is_empty() => {
            let e = body(0);
            let e = match makes_record(e) {
                true => format!("({})", sim_in(cx, e)),
                false => wrap(sim_at(cx, e), OR),
            };
            let mut arms = Vec::new();
            for ((v, ts), arm) in en.variants.iter().zip(&args[1..]) {
                if arm.binds.len() != ts.len() {
                    return bad;
                }
                let pattern = match ts.is_empty() {
                    true => v.clone(),
                    false => format!("{v}({})", arm.binds.join(", ")),
                };
                arms.push(format!("{pattern} => {}", sim_in(cx, &arm.body)));
            }
            (format!("match {e} {{ {} }}", arms.join(", ")), LET)
        }
        (_, _, false) => bad,
        // `None`, `Some(x)`.
        (Op::Variant(en, i), n, _) if en.variants.get(*i).is_some_and(|(_, ts)| ts.len() == n) => {
            let name = &en.variants[*i].0;
            match n {
                0 => (name.clone(), ATOM),
                _ => {
                    let items: Vec<String> = (0..n).map(|k| sim_in(cx, body(k))).collect();
                    (format!("{name}({})", items.join(", ")), POSTFIX)
                }
            }
        }
        // `t.call(x)`: a method.
        (Op::Call(f), n, true) if n > 0 && cx.calling.methods.contains(f) => {
            let name = f.rsplit_once("::").map_or(f.as_str(), |(_, n)| n);
            let rest: Vec<String> = args[1..].iter().map(|a| sim_in(cx, &a.body)).collect();
            (
                format!(
                    "{}.{name}({})",
                    wrap(sim_at(cx, body(0)), POSTFIX),
                    rest.join(", ")
                ),
                POSTFIX,
            )
        }
        // `light(h)`, `sun::light(h)`.
        (Op::Call(f), _, _) => {
            let args: Vec<String> = args.iter().map(|a| sim_in(cx, &a.body)).collect();
            (format!("{}({})", cx.call(f), args.join(", ")), POSTFIX)
        }
        (Op::Truth(b), 0, _) => (b.to_string(), ATOM),
        (Op::Int(n), 0, _) => (n.to_string(), if n.is_negative() { UNARY } else { ATOM }),
        (Op::Text(s), 0, _) => (string(s), ATOM),
        (Op::Real(r), 0, _) => (real(r), if r.numer().is_negative() { UNARY } else { ATOM }),
        (Op::Not, 1, _) => match body(0) {
            Expression::Apply(Op::Equal, inner)
                if inner.len() == 2 && inner.iter().all(|a| a.binds.is_empty()) =>
            {
                let (a, b) = (&inner[0].body, &inner[1].body);
                (
                    format!(
                        "{} != {}",
                        wrap(sim_at(cx, a), CMP + 1),
                        wrap(sim_at(cx, b), CMP + 1)
                    ),
                    CMP,
                )
            }
            x => (format!("!{}", wrap(sim_at(cx, x), UNARY)), UNARY),
        },
        (Op::Or, 2, _) => binary("||", OR, OR, OR + 1),
        (Op::And, 2, _) => binary("&&", AND, AND, AND + 1),
        (Op::Equal, 2, _) => binary("==", CMP, CMP + 1, CMP + 1),
        (Op::Less, 2, _) => binary("<", CMP, CMP + 1, CMP + 1),
        (Op::AtMost, 2, _) => binary("<=", CMP, CMP + 1, CMP + 1),
        (Op::Add, 2, _) => binary("+", SUM, SUM, SUM + 1),
        (Op::Sub, 2, _) => binary("-", SUM, SUM, SUM + 1),
        (Op::Join, 2, _) => binary("++", SUM, SUM, SUM + 1),
        (Op::Mul, 2, _) => binary("*", PRODUCT, PRODUCT, PRODUCT + 1),
        (Op::Div, 2, _) => binary("/", PRODUCT, PRODUCT, PRODUCT + 1),
        (Op::Rem, 2, _) => binary("%", PRODUCT, PRODUCT, PRODUCT + 1),
        (Op::Within, 3, _) => (
            format!(
                "within({}, {}, {})",
                sim_in(cx, body(0)),
                sim_in(cx, body(1)),
                sim_in(cx, body(2))
            ),
            ATOM,
        ),
        (Op::If, 3, _) => {
            let c = body(0);
            let c = match makes_record(c) {
                true => format!("({})", sim_in(cx, c)),
                false => wrap(sim_at(cx, c), OR),
            };
            let otherwise = match body(2) {
                e @ Expression::Apply(Op::If, a)
                    if a.len() == 3 && a.iter().all(|a| a.binds.is_empty()) =>
                {
                    sim_in(cx, e)
                }
                e => format!("{{ {} }}", sim_in(cx, e)),
            };
            (
                format!("if {c} {{ {} }} else {otherwise}", sim_in(cx, body(1))),
                LET,
            )
        }
        (Op::Make(r), n, _) if n == r.fields.len() => {
            let items: Vec<String> = (0..n).map(|i| sim_in(cx, body(i))).collect();
            match (r.name.is_empty(), n) {
                (true, _) if r.labelled() => {
                    let fields: Vec<String> = r
                        .fields
                        .iter()
                        .zip(&items)
                        .map(|((f, _), x)| format!("{f}: {x}"))
                        .collect();
                    (format!("({})", fields.join(", ")), ATOM)
                }
                (true, _) => (format!("({})", items.join(", ")), ATOM),
                (false, _) => {
                    // The fields a record with `..` does not state come
                    // from another, `..base`, last.
                    let fields: Vec<String> = r
                        .fields
                        .iter()
                        .enumerate()
                        .zip(&items)
                        .map(
                            |((i, (f, _)), x)| match (f == crate::model::REST, body(i)) {
                                (true, Expression::Apply(Op::Field(g), base))
                                    if g == crate::model::REST =>
                                {
                                    format!("..{}", wrap(sim_at(cx, &base[0].body), POSTFIX))
                                }
                                (true, _) => MALFORMED.to_string(),
                                (false, _) => format!("{f}: {x}"),
                            },
                        )
                        .collect();
                    (format!("{} {{ {} }}", r.name, fields.join(", ")), ATOM)
                }
            }
        }
        (Op::Field(f), 1, _) => (
            format!("{}.{f}", wrap(sim_at(cx, body(0)), POSTFIX)),
            POSTFIX,
        ),
        // `[1, 2]`, `xs.len()`, `xs[i]`.
        (Op::Seq(_), n, _) => {
            let items: Vec<String> = (0..n).map(|i| sim_in(cx, body(i))).collect();
            (format!("[{}]", items.join(", ")), ATOM)
        }
        (Op::Len, 1, _) => (
            format!("{}.len()", wrap(sim_at(cx, body(0)), POSTFIX)),
            POSTFIX,
        ),
        (Op::Index, 2, _) => (
            format!(
                "{}[{}]",
                wrap(sim_at(cx, body(0)), POSTFIX),
                sim_in(cx, body(1))
            ),
            POSTFIX,
        ),
        // `s.starts_with(b)`, `s.decoded()`.
        (Op::StartsWith, 2, _) => (
            format!(
                "{}.starts_with({})",
                wrap(sim_at(cx, body(0)), POSTFIX),
                sim_in(cx, body(1))
            ),
            POSTFIX,
        ),
        (Op::Decoded, 1, _) => (
            format!("{}.decoded()", wrap(sim_at(cx, body(0)), POSTFIX)),
            POSTFIX,
        ),
        _ => bad,
    }
}
