//! Whether a claim follows from the claims on the functions it reaches.
//!
//! The code of a subject is unknown: `add(a, b)` is only a term. Claims
//! above say which terms are equal (`add(a, b) == add(b, a)`), and a
//! body says what a function is built from. A claim follows when
//! supposing it false, on inputs nobody chose, contradicts them.
//!
//! The reasoning is sound and not complete: it says "follows" only when
//! it does, and may miss a consequence that needs arithmetic on unknown
//! values, or more steps than it tries.

use crate::model::*;
use crate::types::set::{Set, Value};
use std::collections::HashMap;

type Id = usize;

#[derive(Clone)]
enum Node {
    Val(Value),
    /// A function of the code, an operator, or an input (`$a`, no
    /// argument).
    App(String, Vec<Id>),
}

/// The terms, each once.
#[derive(Default)]
struct Bank {
    nodes: Vec<Node>,
    index: HashMap<String, Id>,
    /// The set each term is in, where one is known.
    range: Vec<Option<Set>>,
}

impl Bank {
    fn val(&mut self, v: Value) -> Id {
        let key = format!("v {v}");
        if let Some(id) = self.index.get(&key) {
            return *id;
        }
        self.add(key, Node::Val(v), None)
    }
    fn app(&mut self, f: &str, args: Vec<Id>, range: Option<Set>) -> Id {
        let key = format!("{f} {args:?}");
        self.add(key, Node::App(f.to_string(), args), range)
    }
    fn add(&mut self, key: String, node: Node, range: Option<Set>) -> Id {
        if let Some(id) = self.index.get(&key) {
            return *id;
        }
        self.nodes.push(node);
        self.range.push(range);
        self.index.insert(key, self.nodes.len() - 1);
        self.nodes.len() - 1
    }
}

/// What is supposed: which terms are equal, which differ, and of which
/// lists of terms one at least has its value.
#[derive(Clone, Default)]
struct State {
    parent: Vec<Id>,
    value: Vec<Option<Value>>,
    differ: Vec<(Id, Id)>,
    clauses: Vec<Vec<(Id, bool)>>,
    /// The terms whose truth was already taken apart.
    opened: Vec<Option<bool>>,
    conflict: bool,
}

impl State {
    fn grow(&mut self, bank: &Bank) {
        for id in self.parent.len()..bank.nodes.len() {
            self.parent.push(id);
            self.value.push(match &bank.nodes[id] {
                Node::Val(v) => Some(v.clone()),
                Node::App(..) => None,
            });
            self.opened.push(None);
        }
    }
    fn find(&self, mut a: Id) -> Id {
        while self.parent[a] != a {
            a = self.parent[a];
        }
        a
    }
    fn union(&mut self, a: Id, b: Id) -> bool {
        let (a, b) = (self.find(a), self.find(b));
        if a == b {
            return false;
        }
        match (&self.value[a], &self.value[b]) {
            (Some(x), Some(y)) if x != y => self.conflict = true,
            (None, Some(_)) => self.value[a] = self.value[b].clone(),
            _ => {}
        }
        self.parent[b] = a;
        true
    }
    fn truth(&self, t: Id) -> Option<bool> {
        match self.value[self.find(t)] {
            Some(Value::Bool(b)) => Some(b),
            _ => None,
        }
    }
    /// Suppose the term `t` is `b`.
    fn suppose(&mut self, bank: &mut Bank, t: Id, b: bool) -> bool {
        let v = bank.val(Value::Bool(b));
        self.grow(bank);
        self.union(t, v)
    }
    /// What the truth of a term says of its parts.
    fn open(&mut self, bank: &mut Bank, t: Id, b: bool) -> bool {
        if self.opened[t] == Some(b) {
            return false;
        }
        self.opened[t] = Some(b);
        let Node::App(op, args) = bank.nodes[t].clone() else {
            return false;
        };
        match (op.as_str(), &args[..], b) {
            ("!", [a], _) => self.suppose(bank, *a, !b),
            ("&&", [a, c], true) | ("||", [a, c], false) => {
                self.suppose(bank, *a, b) | self.suppose(bank, *c, b)
            }
            ("&&", [a, c], false) | ("||", [a, c], true) => {
                self.clauses.push(vec![(*a, b), (*c, b)]);
                true
            }
            ("if", [c, t, f], _) => {
                let (yes, no) = (bank.val(Value::Bool(true)), bank.val(Value::Bool(false)));
                let _ = (yes, no);
                self.clauses.push(vec![(*c, false), (*t, b)]);
                self.clauses.push(vec![(*c, true), (*f, b)]);
                true
            }
            ("==", [a, c], true) | ("!=", [a, c], false) => self.union(*a, *c),
            ("==", [a, c], false) | ("!=", [a, c], true) => {
                self.differ.push((*a, *c));
                true
            }
            _ => false,
        }
    }
    /// Everything that follows without choosing.
    fn settle(&mut self, bank: &mut Bank) {
        let mut changed = true;
        while changed && !self.conflict {
            changed = false;
            // Two calls of one function on equal inputs are equal.
            let mut seen: HashMap<(String, Vec<Id>), Id> = HashMap::new();
            for t in 0..bank.nodes.len() {
                let Node::App(f, args) = bank.nodes[t].clone() else {
                    continue;
                };
                let roots: Vec<Id> = args.iter().map(|a| self.find(*a)).collect();
                match seen.get(&(f.clone(), roots.clone())) {
                    Some(first) => changed |= self.union(*first, t),
                    None => drop(seen.insert((f.clone(), roots.clone()), t)),
                }
                // An operator on known values is computed.
                let known: Vec<Option<Value>> =
                    roots.iter().map(|r| self.value[*r].clone()).collect();
                let got = match (f.as_str(), &known[..], &roots[..]) {
                    ("==", _, [a, b]) if a == b => Some(Value::Bool(true)),
                    ("!=", _, [a, b]) if a == b => Some(Value::Bool(false)),
                    ("==" | "!=", _, [a, b]) => {
                        let apart = |(x, y): &(Id, Id)| {
                            let (x, y) = (self.find(*x), self.find(*y));
                            (x, y) == (*a, *b) || (y, x) == (*a, *b)
                        };
                        let differ = match (&known[0], &known[1]) {
                            (Some(x), Some(y)) => x != y,
                            _ => self.differ.iter().any(apart),
                        };
                        differ.then_some(Value::Bool(f == "!="))
                    }
                    ("&&", [Some(Value::Bool(false)), _] | [_, Some(Value::Bool(false))], _) => {
                        Some(Value::Bool(false))
                    }
                    ("||", [Some(Value::Bool(true)), _] | [_, Some(Value::Bool(true))], _) => {
                        Some(Value::Bool(true))
                    }
                    ("if", [Some(Value::Bool(c)), _, _], _) => {
                        changed |= self.union(args[if *c { 1 } else { 2 }], t);
                        None
                    }
                    ("!" | "-", [Some(x)], _) => crate::eval::unary(&f, x),
                    (op, [Some(x), Some(y)], _) if operator(op) => crate::eval::binary(op, x, y),
                    _ => None,
                };
                if let Some(v) = got {
                    let v = bank.val(v);
                    self.grow(bank);
                    changed |= self.union(t, v);
                }
                if let Some(b) = self.truth(t) {
                    changed |= self.open(bank, t, b);
                }
            }
            let same = |(a, b): &(Id, Id)| self.find(*a) == self.find(*b);
            self.conflict |= self.differ.iter().any(same);
            // A list where every term but one is known not to have its
            // value: that one has it.
            for i in 0..self.clauses.len() {
                let open: Vec<(Id, bool)> = self.clauses[i]
                    .iter()
                    .filter(|(t, b)| self.truth(*t) != Some(!b))
                    .cloned()
                    .collect();
                match open[..] {
                    [] => self.conflict = true,
                    [(t, b)] => changed |= self.suppose(bank, t, b),
                    _ => {}
                }
            }
        }
    }
    /// Whether what is supposed can't all be true, trying each way where
    /// a choice is left.
    fn impossible(mut self, bank: &mut Bank, budget: &mut usize) -> bool {
        self.settle(bank);
        if self.conflict {
            return true;
        }
        let undecided = |c: &&Vec<(Id, bool)>| c.iter().all(|(t, b)| self.truth(*t) != Some(*b));
        let Some(&(t, b)) = self
            .clauses
            .iter()
            .find(undecided)
            .and_then(|c| c.iter().find(|(t, _)| self.truth(*t).is_none()))
        else {
            return false;
        };
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        [b, !b].into_iter().all(|way| {
            let mut next = self.clone();
            next.suppose(bank, t, way);
            next.impossible(bank, budget)
        })
    }
}

fn operator(op: &str) -> bool {
    [
        "==", "!=", "<", "<=", ">", ">=", "+", "-", "*", "/", "&&", "||", "|", "&", "\\", "in",
    ]
    .contains(&op)
}

struct Prover<'a> {
    model: &'a Model,
    bank: Bank,
    /// The steps left: a body that calls itself is not followed for ever.
    fuel: usize,
}

impl Prover<'_> {
    /// The term an expression is, with a term for each name.
    fn term(&mut self, e: &Expr, env: &[(String, Id)], depth: usize) -> Option<Id> {
        self.fuel = self.fuel.checked_sub(1)?;
        match e {
            Expr::Lit(v) => Some(self.bank.val(v.clone())),
            Expr::Var(x) => match env.iter().rev().find(|n| &n.0 == x) {
                Some((_, t)) => Some(*t),
                None => None,
            },
            Expr::Let(x, v, body) => {
                let v = self.term(v, env, depth)?;
                let mut env = env.to_vec();
                env.push((x.clone(), v));
                self.term(body, &env, depth)
            }
            Expr::Unary(op, a) => {
                let a = self.term(a, env, depth)?;
                Some(self.bank.app(op, vec![a], (*op == "!").then(Set::bool)))
            }
            Expr::Binary(op, a, b) => {
                let args = vec![self.term(a, env, depth)?, self.term(b, env, depth)?];
                let logic = !["+", "-", "*", "/", "|", "&", "\\"].contains(op);
                Some(self.bank.app(op, args, logic.then(Set::bool)))
            }
            Expr::If(c, t, f) => {
                let args = vec![
                    self.term(c, env, depth)?,
                    self.term(t, env, depth)?,
                    self.term(f, env, depth)?,
                ];
                Some(self.bank.app("if", args, None))
            }
            Expr::Call(f, args) => {
                let g = self.model.declared_function(f)?;
                let args: Option<Vec<Id>> = args.iter().map(|a| self.term(a, env, depth)).collect();
                let args = args?;
                // On values, a function Mitra computes is computed.
                let values: Option<Vec<Expr>> = args
                    .iter()
                    .map(|a| match &self.bank.nodes[*a] {
                        Node::Val(v) => Some(Expr::Lit(v.clone())),
                        Node::App(..) => None,
                    })
                    .collect();
                if let Some(values) = values.filter(|_| g.is_spec()) {
                    let call = Expr::Call(f.clone(), values);
                    if let Some(v) = crate::eval::computed(self.model, &call, &[]) {
                        return Some(self.bank.val(v));
                    }
                }
                match &g.body {
                    // Else its body says what it is.
                    Some(body) if depth < 6 => {
                        let names = g.params.iter().map(|p| p.0.clone());
                        let env: Vec<(String, Id)> = names.zip(args).collect();
                        self.term(body, &env, depth + 1)
                    }
                    Some(_) => None,
                    None => Some(self.bank.app(f, args, Some(g.range.clone()))),
                }
            }
            // Anything else is a value when every name is one.
            _ => {
                let values: Option<Vec<(String, Value)>> = env
                    .iter()
                    .map(|(x, t)| match &self.bank.nodes[*t] {
                        Node::Val(v) => Some((x.clone(), v.clone())),
                        Node::App(..) => None,
                    })
                    .collect();
                let v = crate::eval::computed(self.model, e, &values?)?;
                Some(self.bank.val(v))
            }
        }
    }
}

/// The inputs of a claim with their sets, and what it says as one
/// expression.
fn statement(model: &Model, c: &Claim) -> Option<(Vec<(String, Set)>, Expr)> {
    match &c.target {
        Target::Property(e) => {
            let hidden = model.properties.iter().find(|f| f.name == c.name)?;
            let inputs = hidden.params.clone();
            Some((inputs, e.clone()))
        }
        Target::Matches(e) => {
            let f = model.declared_function(&c.func)?;
            let (mut inputs, mut args) = (Vec::new(), Vec::new());
            for (pos, p) in c.pattern.iter().zip(&f.params) {
                match (&pos.var, &pos.set) {
                    (Some(x), set) => {
                        inputs.push((x.clone(), set.clone().unwrap_or(p.1.clone())));
                        args.push(Expr::Var(x.clone()));
                    }
                    (None, Some(set)) => args.push(Expr::Lit(set.single()?)),
                    (None, None) => return None,
                }
            }
            let call = Expr::Call(c.func.clone(), args);
            Some((
                inputs,
                Expr::Binary("==", Box::new(call), Box::new(e.clone())),
            ))
        }
        _ => None,
    }
}

/// Whether `claim` follows from `premises`, on every input.
fn follows(model: &Model, claim: &Claim, premises: &[&Claim]) -> bool {
    let Some((inputs, said)) = statement(model, claim) else {
        return false;
    };
    let mut p = Prover {
        model,
        bank: Bank::default(),
        fuel: 20_000,
    };
    // Inputs nobody chose: one term each.
    let env: Vec<(String, Id)> = inputs
        .iter()
        .map(|(x, set)| {
            let t = p.bank.app(&format!("${x}"), Vec::new(), Some(set.clone()));
            (x.clone(), t)
        })
        .collect();
    let Some(goal) = p.term(&said, &env, 0) else {
        return false;
    };
    // Each premise, on every term that is in the sets of its inputs.
    const MOST: usize = 512;
    let mut facts = Vec::new();
    for _round in 0..2 {
        let terms = p.bank.nodes.len();
        for premise in premises {
            let Some((inputs, said)) = statement(model, premise) else {
                continue;
            };
            let fits = |set: &Set, t: Id, bank: &Bank| match (&bank.nodes[t], &bank.range[t]) {
                (Node::Val(v), _) => set.contains(v),
                (_, Some(r)) => r.is_subset(set),
                _ => false,
            };
            let each: Vec<Vec<Id>> = inputs
                .iter()
                .map(|(_, set)| (0..terms).filter(|t| fits(set, *t, &p.bank)).collect())
                .collect();
            let count = each.iter().map(Vec::len).product::<usize>();
            if count == 0 || count > MOST || facts.len() > 4 * MOST {
                continue;
            }
            let mut picks = vec![0; each.len()];
            loop {
                let names = inputs.iter().map(|i| i.0.clone());
                let env: Vec<(String, Id)> = names
                    .zip(picks.iter().zip(&each).map(|(i, e)| e[*i]))
                    .collect();
                if let Some(fact) = p.term(&said, &env, 0) {
                    facts.push(fact);
                }
                let Some(at) = (0..picks.len()).find(|i| picks[*i] + 1 < each[*i].len()) else {
                    break;
                };
                picks[at] += 1;
                picks[..at].iter_mut().for_each(|i| *i = 0);
            }
        }
    }
    let mut state = State::default();
    state.grow(&p.bank);
    for fact in facts {
        state.suppose(&mut p.bank, fact, true);
    }
    state.suppose(&mut p.bank, goal, false);
    state.impossible(&mut p.bank, &mut 64)
}

/// The functions a claim reaches: those it calls, those their bodies
/// call, and those they are declared to use.
fn reach(model: &Model, c: &Claim) -> Vec<String> {
    let said = |c: &Claim| match &c.target {
        Target::Matches(e) | Target::Property(e) => calls(e),
        _ => Vec::new(),
    };
    let mut all = said(c);
    all.push(c.func.clone());
    let mut i = 0;
    while i < all.len() {
        let f = all[i].clone();
        i += 1;
        let mut more = Vec::new();
        for g in [f.clone(), format!("{f}::body")] {
            if let Some(body) = model.declared_function(&g).and_then(|g| g.body.as_ref()) {
                more.push(g);
                more.extend(calls(body));
            }
        }
        for d in model.claims.iter().filter(|d| d.func == f) {
            if let Target::Uses { used, .. } = &d.target {
                more.extend(used.iter().cloned());
            }
        }
        for g in more {
            if !all.contains(&g) {
                all.push(g);
            }
        }
    }
    all
}

/// For each claim that follows from claims above it on the functions it
/// reaches: the fewest of them that are enough.
// mitra: implements verdicts::derive
pub fn derive(model: &mut Model) {
    for i in 0..model.claims.len() {
        let c = &model.claims[i];
        if !matches!(c.target, Target::Matches(_) | Target::Property(_)) {
            continue;
        }
        // A spec is what a body says: it is evidence that settles it.
        if c.name.ends_with("::spec") {
            continue;
        }
        // A claim with no input is a point: evidence or the specs settle
        // it.
        if c.pattern.iter().all(|p| p.var.is_none()) {
            continue;
        }
        let reached = reach(model, c);
        let about = |d: &&Claim| {
            let mut named = match &d.target {
                Target::Matches(e) => calls(e),
                Target::Property(e) => calls(e),
                _ => return false,
            };
            if matches!(d.target, Target::Matches(_)) {
                named.push(d.func.clone());
            }
            !named.is_empty() && named.iter().all(|f| reached.contains(f))
        };
        let mut premises: Vec<&Claim> = model.claims[..i].iter().filter(about).collect();
        if premises.is_empty() || !follows(model, c, &premises) {
            continue;
        }
        let mut at = 0;
        while at < premises.len() {
            let mut fewer = premises.clone();
            fewer.remove(at);
            match follows(model, c, &fewer) {
                true => premises = fewer,
                false => at += 1,
            }
        }
        if premises.is_empty() {
            continue;
        }
        let from = premises.iter().map(|d| d.name.clone()).collect();
        let c = &mut model.claims[i];
        (c.from, c.auto, c.proved) = (from, true, true);
    }
}
