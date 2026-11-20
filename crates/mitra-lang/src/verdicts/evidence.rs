//! Step 1: what the evidence says about each claim, before derivations
//! and classes, and the evidence that contradicts itself.

use super::*;
use crate::eval::Subjects;
use std::collections::BTreeMap;

/// One judged input: the input, whether it passed, the evidence line.
pub(crate) type Point = (Vec<Value>, bool, usize);

/// What the evidence says about a claim, before derivations and classes.
pub(crate) struct Direct {
    pub(crate) domain: Tuples,
    pub(crate) right: Tuples,
    pub(crate) wrong: Tuples,
    /// Single inputs judged for this claim, contradictions left out.
    pub(crate) points: Vec<Point>,
    /// A failure over a set too large to read input by input: its classes
    /// are unknown.
    pub(crate) blind: bool,
    /// Inputs with both a pass and a failure: their classes are not relied
    /// on.
    pub(crate) tainted: Vec<Vec<Value>>,
    /// Each pass or failure: its inputs, its side and its evidence line.
    pub(crate) sources: Vec<(Tuples, bool, usize)>,
    /// Each way the evidence says the claim passes or fails on some
    /// inputs: the evidence lines that say it together.
    pub(crate) arguments: Vec<(Tuples, bool, Vec<usize>)>,
    /// For a property: each failing input with the lines of every output
    /// it was computed from. They are wrong together or not at all.
    pub(crate) joint: Vec<(Vec<Value>, Vec<usize>)>,
    /// For a claim on a measure that has no value yet: what was counted.
    pub(crate) counted: Option<String>,
    /// The assumptions whose word gave an output the claim read.
    pub(crate) leans: Vec<String>,
}

/// The inputs of `set` one by one, when there are few enough.
pub(crate) fn listed(set: &Tuples) -> Vec<Vec<Value>> {
    match set.size_bound() {
        Some(n) if n <= POINTS => set.values().unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The evidence on this line as it was written.
pub(super) fn shown(model: &Model, line: usize) -> String {
    let written = model.written.as_ref();
    if let Some(text) = written.and_then(|w| crate::parse::written(&w.1, line)) {
        return text;
    }
    let e = model.evidence.iter().find(|e| e.line == line);
    e.map(|e| {
        crate::report::print::print_evidence(std::slice::from_ref(e), Some(model))
            .trim_end()
            .to_string()
    })
    .unwrap_or_default()
}

/// A pass and a failure that can't both be true, once.
pub(crate) fn clash(
    model: &Model,
    all: &mut Vec<Contradiction>,
    subject: &Function,
    both: &Tuples,
    a: usize,
    b: usize,
) {
    let k = Contradiction {
        subject: subject.name.clone(),
        inputs: both.show(&subject.domain()),
        pass: (a, shown(model, a)),
        fail: (b, shown(model, b)),
    };
    // The same two statements on other lines add nothing.
    let known = |o: &Contradiction| {
        (o.pass.0, o.fail.0) == (a, b)
            || (&o.subject, &o.inputs, &o.pass.1, &o.fail.1)
                == (&k.subject, &k.inputs, &k.pass.1, &k.fail.1)
    };
    if !all.iter().any(known) {
        all.push(k);
    }
}

impl<'a> Judge<'a> {
    /// The outputs observed, and the evidence that applies to nothing.
    pub(super) fn new(model: &'a Model) -> Judge<'a> {
        let mut observed: BTreeMap<(String, Vec<Value>), Vec<Value>> = BTreeMap::new();
        let mut flags = Vec::new();
        for e in model.evidence.iter() {
            let Some(f) = model.function(&e.about) else {
                continue;
            };
            if f.is_spec() {
                flags.push(Flag {
                    line: e.line,
                    message: format!(
                        "`{}` is a spec, true by definition: evidence does not apply to it",
                        f.name
                    ),
                });
            } else if let Judgment::Output(out) = &e.judgment {
                let input = single(&e.pattern).expect("checked");
                observed
                    .entry((f.name.clone(), input))
                    .or_default()
                    .push(out.clone());
            }
        }
        let mut ev = Eval::bare(model);
        ev.observed = observed;
        ev.rules = crate::eval::rules(model);
        ev.relations = crate::eval::relations(model);
        Judge {
            model,
            ev,
            flags,
            contradictions: Vec::new(),
            split: Vec::new(),
            direct: Vec::new(),
            sorting: Vec::new(),
            conflicts: Vec::new(),
            stands: Vec::new(),
            logic_of: HashMap::new(),
            assuming_of: HashMap::new(),
            verdicts: Vec::new(),
            through: Vec::new(),
            confidence_of: HashMap::new(),
        }
    }

    /// Whatever the claims: an input judged correct and not correct, or
    /// seen with two outputs.
    pub(super) fn split(&mut self) {
        let model = self.model;
        for f in model.functions.iter().filter(|f| !f.is_spec()) {
            let about = |e: &&Evidence| e.about == f.name;
            let of = |want: bool| {
                let want = Judgment::Correct(want);
                let on = model.evidence.iter().filter(about);
                on.filter(move |e| e.judgment == want)
            };
            for a in of(true) {
                for b in of(false) {
                    let both = model
                        .covers(&f.name, &a.pattern)
                        .intersect(&model.covers(&f.name, &b.pattern));
                    if !both.is_empty() {
                        clash(model, &mut self.contradictions, f, &both, a.line, b.line);
                    }
                }
            }
            let outputs: Vec<(Vec<Value>, &Value, usize)> = model
                .evidence
                .iter()
                .filter(about)
                .filter_map(|e| match &e.judgment {
                    Judgment::Output(v) => Some((single(&e.pattern)?, v, e.line)),
                    _ => None,
                })
                .collect();
            for (n, (x, v, a)) in outputs.iter().enumerate() {
                // A boundary may answer differently each time, and a function
                // the model says is not deterministic.
                let free = |d: &(String, String, bool)| d.1 == f.name && !d.2;
                if outputs[..n].iter().any(|o| &o.0 == x)
                    || model.boundaries.contains(&f.name)
                    || model.determinism.iter().any(free)
                {
                    continue;
                }
                // Two outputs on one input: neither is relied on, and the
                // claim that the function is deterministic is refuted.
                let _ = a;
                for _ in outputs.iter().filter(|o| &o.0 == x && o.1 != *v) {
                    self.split.push((f.name.clone(), x.clone()));
                }
            }
        }
    }

    /// For each claim: where the evidence says it is right, where wrong.
    pub(super) fn direct(&mut self) -> Result<(), Fail> {
        let (model, ev) = (self.model, &self.ev);
        for c in &model.claims {
            ev.claim.replace(c.name.clone());
            ev.leaned.borrow_mut().clear();
            let func = model.function(&c.func).expect("checked");
            let mut domain = model.covers(&c.func, &c.pattern);
            // `when` on the inputs alone: the inputs it
            // refuses are not in the claim.
            let when = format!("{}::when", c.name);
            if let (Some(all), true) = (domain.values(), model.function(&when).is_some()) {
                for x in all {
                    let kept = ev.call(&when, &x, c.line, Subjects::Observed);
                    if kept.map_err(model_err)? != [Value::Bool(true)] {
                        domain = domain.minus(&Tuples::point(&x));
                    }
                }
            }
            let arity = domain.arity();
            // `right` and `wrong` over all inputs of the function, then cut to
            // the claim's domain; `points` keeps the inputs outside it too.
            let (mut right, mut wrong) = (Tuples::empty(arity), Tuples::empty(arity));
            let mut points: Vec<Point> = Vec::new();
            let mut blind = false;
            let mut tainted = Vec::new();
            // For a property: each failing input with every output it read.
            let mut joint: Vec<(Vec<Value>, Vec<usize>)> = Vec::new();
            let mut counted: Option<String> = None;
            // Each pass or failure with its inputs and evidence line, to find
            // those that can't both be true.
            let mut sources: Vec<(Tuples, bool, usize)> = Vec::new();
            let mut arguments: Vec<(Tuples, bool, Vec<usize>)> = Vec::new();
            for e in model.evidence.iter() {
                let ok = match &e.judgment {
                    Judgment::Holds(ok) if e.about == c.name => *ok,
                    Judgment::Correct(ok)
                        if e.about == c.func && c.target == Target::Correct && !func.is_spec() =>
                    {
                        *ok
                    }
                    _ => continue,
                };
                let mut set = model.covers(&c.func, &e.pattern);
                if matches!(e.judgment, Judgment::Holds(_)) {
                    // Evidence on a claim says nothing outside the claim.
                    set = set.intersect(&domain);
                }
                let inputs = listed(&set);
                blind |= !ok && inputs.is_empty() && !set.is_empty();
                points.extend(inputs.into_iter().map(|x| (x, ok, e.line)));
                sources.push((set.clone(), ok, e.line));
                arguments.push((set.clone(), ok, vec![e.line]));
                let side = if ok { &mut right } else { &mut wrong };
                *side = side.union(&set);
            }
            if let Target::Matches(target) | Target::Property(target) = &c.target {
                let property = matches!(c.target, Target::Property(_));
                // Every input of a finite domain; otherwise every observed input.
                // A claim on a record is judged on the values observed only.
                let seen = model.invariant(c).then(Vec::new);
                let inputs = seen
                    .or_else(|| domain.values())
                    .unwrap_or_else(|| match property {
                        true => ev.candidates(c, target, &domain),
                        false => ev
                            .observed
                            .keys()
                            .map(|(_, x)| x.clone())
                            .filter(|x| x.len() == arity && domain.contains(x))
                            .collect(),
                    });
                // For classes, also the observed inputs outside the domain that
                // the same statement reaches: those that fit the claim's values.
                let reach: Pattern = c
                    .pattern
                    .iter()
                    .map(|p| Position {
                        var: p.var.clone(),
                        set: p.set.clone().filter(|_| p.var.is_none()),
                    })
                    .collect();
                let reach = model.covers(&c.func, &reach);
                let outside: Vec<Vec<Value>> = ev
                    .observed
                    .keys()
                    .filter(|(f, x)| f == &c.func && reach.contains(x) && !domain.contains(x))
                    .map(|(_, x)| x.clone())
                    .collect();
                for x in inputs.into_iter().chain(outside) {
                    if self.split.contains(&(c.func.clone(), x.clone())) {
                        // Two outputs for one input: neither is relied on.
                        tainted.push(x);
                        continue;
                    }
                    let inside = domain.contains(&x);
                    ev.read.borrow_mut().clear();
                    ev.measured.borrow_mut().clear();
                    ev.notes.borrow_mut().clear();
                    let compared = match ev.compare(c, target, &x, Subjects::Observed) {
                        // Outside the domain the statement need not be defined.
                        Err(_) if !inside => continue,
                        r => r.map_err(model_err)?,
                    };
                    // A measure without a value: what was counted so far.
                    if let (None, Some(note)) = (compared, ev.notes.take().into_iter().next()) {
                        counted.get_or_insert(note);
                    }
                    // The outputs a property was computed from.
                    let mut read = ev.read.take();
                    read.sort();
                    read.dedup();
                    if property && read.iter().any(|r| self.split.contains(r)) {
                        tainted.push(x);
                        continue;
                    }
                    if let Some((ok, ko)) = compared {
                        let own = [(c.func.clone(), x.clone())];
                        // A matched output that a vouched claim gave: what it
                        // was worked out from, as for a property.
                        let property = property || !ev.measured.borrow().is_empty();
                        let lines: Vec<usize> = model
                            .evidence
                            .iter()
                            .filter(|e| {
                                let from = if property { &read[..] } else { &own[..] };
                                matches!(e.judgment, Judgment::Output(_))
                                    && from.iter().any(|(g, x)| {
                                        &e.about == g && single(&e.pattern).as_ref() == Some(x)
                                    })
                            })
                            .map(|e| e.line)
                            .collect();
                        let mut lines = lines;
                        if property {
                            lines.extend(ev.measured.take());
                            lines.sort();
                            lines.dedup();
                        }
                        if property && ko && !ok {
                            joint.push((x.clone(), lines.clone()));
                        }
                        let line = lines.first().copied().unwrap_or(0);
                        let point = Tuples::point(&x);
                        // True of one answer of a boundary and not of another:
                        // the claim is about every answer.
                        let answers = read.iter().any(|(g, _)| model.boundaries.contains(g));
                        let ok = ok && !(ko && answers);
                        if ok && ko {
                            // The other side has two outputs, reported there.
                            tainted.push(x);
                            continue;
                        }
                        // Every line that records the output is a source.
                        sources.push((point.clone(), ok, line));
                        arguments.push((point.clone(), ok, lines.clone()));
                        for l in lines.iter().skip(1) {
                            sources.push((point.clone(), ok, *l));
                        }
                        if ok {
                            if inside {
                                right = right.union(&point);
                            }
                            points.push((x.clone(), true, line));
                        }
                        if ko {
                            if inside {
                                wrong = wrong.union(&point);
                            }
                            points.push((x, false, line));
                        }
                    }
                }
            }
            points.retain(|(x, ok, _)| !(if *ok { &wrong } else { &right }).contains(x));
            // A pass and a failure on the same input can't both be true.
            for (pass, _, a) in sources.iter().filter(|s| s.1) {
                for (fail, _, b) in sources.iter().filter(|s| !s.1) {
                    let both = pass.intersect(fail);
                    if both.is_empty() {
                        continue;
                    }
                    let inputs = listed(&both);
                    blind |= inputs.is_empty();
                    tainted.extend(inputs);
                    clash(model, &mut self.contradictions, func, &both, *a, *b);
                }
            }
            points.retain(|(x, _, _)| !tainted.contains(x));
            let mut leans = ev.leaned.take();
            leans.sort();
            leans.dedup();
            self.direct.push(Direct {
                leans,
                counted,
                joint,
                tainted,
                right: right.intersect(&domain),
                wrong: wrong.intersect(&domain),
                domain,
                points,
                blind,
                sources,
                arguments,
            });
        }
        Ok(())
    }

    /// Inputs judged `correct` or not, per subject, whatever the claims.
    pub(super) fn judged(&self, subject: &str) -> Vec<Point> {
        let model = self.model;
        let about =
            |e: &&Evidence| e.about == subject && matches!(e.judgment, Judgment::Correct(_));
        let side = |want: bool| {
            let arity = model.function(subject).expect("checked").params.len();
            model
                .evidence
                .iter()
                .filter(about)
                .filter(|e| e.judgment == Judgment::Correct(want))
                .fold(Tuples::empty(arity), |s, e| {
                    s.union(&model.covers(subject, &e.pattern))
                })
        };
        let (right, wrong) = (side(true), side(false));
        let mut points = Vec::new();
        for e in model.evidence.iter().filter(about) {
            let ok = e.judgment == Judgment::Correct(true);
            for x in listed(&model.covers(subject, &e.pattern)) {
                if !(if ok { &wrong } else { &right }).contains(&x) {
                    points.push((x, ok, e.line));
                }
            }
        }
        points
    }
}
