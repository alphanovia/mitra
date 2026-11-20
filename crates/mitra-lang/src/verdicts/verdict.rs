//! Step 3: the verdict of each claim, from what the evidence says of it,
//! the claims it follows from, the classes that stand for its inputs and
//! the sets that are represented by one input.

use super::classes::Sorted;
use super::evidence::Direct;
use super::*;

/// A `represented` claim: its set and, when one of its inputs passes a
/// claim and another fails it, the two evidence lines.
pub(super) type Stand = (usize, Tuples, Option<(usize, usize)>);

/// The set of a `represented` claim: the inputs where its spec is true.
/// `None` when Mitra can't list them.
pub(crate) fn represented(model: &Model, c: &Claim) -> Option<Tuples> {
    let Target::Represented(spec) = &c.target else {
        return None;
    };
    let g = model.function(spec)?;
    let domain = g.domain();
    if g.body == Some(Expr::Lit(Value::Bool(true))) {
        return Some(domain);
    }
    let mut set = Tuples::empty(domain.arity());
    for x in domain.values()? {
        let names: Vec<(String, Value)> = g
            .params
            .iter()
            .map(|p| p.0.clone())
            .zip(x.iter().cloned())
            .collect();
        if crate::eval::computed(model, g.body.as_ref()?, &names)? == Value::Bool(true) {
            set = set.union(&Tuples::point(&x));
        }
    }
    Some(set)
}

impl<'a> Judge<'a> {
    pub(super) fn stands(&self) -> Vec<Stand> {
        let model = self.model;
        let mut stands: Vec<Stand> = Vec::new();
        for (r, c) in model.claims.iter().enumerate() {
            let Some(set) = represented(model, c) else {
                continue;
            };
            let mut split = None;
            for (j, d) in model.claims.iter().enumerate() {
                if d.func != c.func || d.target.is_structural() {
                    continue;
                }
                let inside = |ok: bool| {
                    let on = self.direct[j].points.iter();
                    on.filter(move |p| p.1 == ok).find(|p| set.contains(&p.0))
                };
                if let (Some(pass), Some(fail)) = (inside(true), inside(false)) {
                    split.get_or_insert((pass.2, fail.2));
                }
            }
            stands.push((r, set, split));
        }
        stands
    }

    /// The samples that failed on a claim, or on the function a claim of
    /// correctness is about: the lines of their evidence.
    pub(super) fn drawn(&self, c: &Claim) -> Vec<usize> {
        let model = self.model;
        let whole = c.target == Target::Correct;
        let on = |e: &&Evidence| {
            matches!(e.judgment, Judgment::Sampled { ok: false, .. })
                && (e.about == c.name || (whole && e.about == c.func))
        };
        model.evidence.iter().filter(on).map(|e| e.line).collect()
    }

    /// Every claim in order: a claim is judged after those it follows from.
    pub(super) fn verdicts(&mut self) -> Result<(), Fail> {
        let model = self.model;
        for (i, c) in model.claims.iter().enumerate() {
            let (v, through) = self.verdict(i, c)?;
            self.logic_of.insert(&c.name, v.logic);
            self.assuming_of.insert(&c.name, v.assuming.clone());
            self.verdicts.push(v);
            self.through.push(through);
        }
        Ok(())
    }

    fn verdict(&self, i: usize, c: &Claim) -> Result<(Verdict, (Vec<usize>, Vec<usize>)), Fail> {
        let (model, ev) = (self.model, &self.ev);
        let Direct {
            domain,
            right,
            wrong,
            ..
        } = &self.direct[i];
        let mut assuming: Vec<String> = Vec::new();
        let mut assume = |name: &String| {
            if !assuming.contains(name) {
                assuming.push(name.clone());
            }
        };
        // A derivation: when the implication is not refuted and no premise
        // fails, the claim follows on its whole domain, assuming the
        // premises that are not verified. A direct counterexample still
        // beats a derivation.
        let implication = if c.from.is_empty() {
            None
        } else if c.proved {
            Some(Implication::Checked)
        } else {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            Some(ev.implication(c, domain, &premises).map_err(model_err)?)
        };
        // Premises nobody wrote count only where every input was checked.
        let enough = |i: Implication| match c.auto {
            true => i == Implication::Checked,
            false => i != Implication::Refuted,
        };
        // A requirement in prose with no claim behind it rests on nothing.
        let orphan = model
            .ensure(&c.name)
            .is_some_and(|e| !e.boundary && e.claims.is_empty());
        let follows = implication.is_some_and(enough)
            && !orphan
            && c.from
                .iter()
                .all(|p| self.logic_of[p.as_str()] != Logic::Fails);
        // An input both right and wrong is a contradiction: it counts on
        // neither side.
        let mut refuted = wrong.minus(right);
        // The specs of what it calls say it is false on an input: the
        // model contradicts itself, whatever the evidence.
        let mut against = None;
        if c.auto && implication == Some(Implication::Refuted) {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            against = ev.counter(c, domain, &premises);
            if let Some(x) = &against {
                refuted = refuted.union(&Tuples::point(x));
            }
        }
        // A word for a requirement of a subject, or for a named need, does
        // not stand in for what it rests on: its claims, its capture, the
        // promise it names.
        let rests = model
            .ensure(&c.name)
            .is_some_and(|e| !e.boundary || e.needs.is_some());
        let tested = match rests {
            true => Tuples::empty(domain.arity()),
            false => right.minus(wrong).minus(&refuted),
        };
        let derived = if follows {
            domain.minus(wrong).minus(&tested)
        } else {
            Tuples::empty(domain.arity())
        };
        if !derived.is_empty() {
            for p in &c.from {
                match self.logic_of[p.as_str()] {
                    Logic::Assuming => self.assuming_of[p.as_str()].iter().for_each(&mut assume),
                    Logic::Unverified => assume(p),
                    _ => {}
                }
            }
        }
        // An assumption on a boundary gave an output: the claim holds as
        // far as the assumption does.
        for a in &self.direct[i].leans {
            if self.logic_of.get(a.as_str()) != Some(&Logic::Holds) {
                assume(a);
            }
        }
        // An assumption itself: nobody checked it where no evidence other
        // than its own word covers it.
        let word = |l: usize| {
            let e = model.evidence.iter().find(|e| e.line == l);
            e.is_some_and(|e| e.source.as_deref() == Some("assumed"))
        };
        let about = |e: &&Evidence| e.about == c.name && word(e.line);
        if model.evidence.iter().any(|e| about(&e)) {
            let mut backed = Tuples::empty(domain.arity());
            for s in self.direct[i].sources.iter().filter(|s| s.1 && !word(s.2)) {
                backed = backed.union(&s.0);
            }
            if !backed.intersect(domain).same(domain) {
                assume(&c.name);
            }
        }
        let covered = tested.union(&derived);
        // `uniform` claims on the subjects it calls: a passing input stands
        // for its class.
        let mut assumed = None;
        let mut via = Vec::new();
        let mut unsorted = None;
        let mut relied = Vec::new();
        if let Sorted::Unsorted(why) = &self.sorting[i] {
            unsorted = Some(why.clone());
        }
        if unsorted.is_none() {
            unsorted = self.direct[i].counted.clone();
        }
        if let Sorted::By(s) = &self.sorting[i] {
            relied = s.claims.clone();
            let u = s.claims[0];
            let class_of = |_: &(), x: &[Value], _: usize| self.sig(i, s, x);
            let spec = &();
            let line = model.claims[u].line;
            let (mut classes, mut failing) = (Vec::new(), Vec::new());
            let points = self.points_of(i);
            for p in &points {
                let class = class_of(spec, &p.0, line)?;
                let side = if p.1 { &mut classes } else { &mut failing };
                if !side.contains(&class) {
                    side.push(class);
                }
            }
            // A class with a contradicted input is not relied on.
            let mut tainted = Vec::new();
            for j in (0..self.direct.len()).filter(|j| self.same(i, *j)) {
                for x in &self.direct[j].tainted {
                    let class = class_of(spec, x, line)?;
                    classes.retain(|k| k != &class);
                    if !tainted.contains(&class) {
                        tainted.push(class);
                    }
                }
            }
            classes.sort();
            let blind = self.blind(i);
            if blind {
                // A failure somewhere unknown: no class can be relied on.
                classes.clear();
            }
            let usable = s.claims.iter().all(|u| self.usable(*u));
            let vouched = s.claims.iter().all(|u| self.vouched(*u));
            let open = domain.minus(&covered).minus(wrong);
            let (mut set, mut untested) = (Tuples::empty(domain.arity()), Vec::new());
            if open.size_bound().is_some_and(|n| n <= POINTS) {
                // Only the classes this domain meets count.
                let passing = std::mem::take(&mut classes);
                let mut hit = Vec::new();
                for x in open.values().unwrap_or_default() {
                    let class = class_of(spec, &x, line)?;
                    if vouched && failing.contains(&class) {
                        // A failure stands for its class too.
                        refuted = refuted.union(&Tuples::point(&x));
                        if !hit.contains(&class) {
                            hit.push(class);
                        }
                    } else if passing.contains(&class) {
                        set = set.union(&Tuples::point(&x));
                        if !classes.contains(&class) {
                            classes.push(class);
                        }
                    } else if !untested.contains(&class) {
                        untested.push(class);
                    }
                }
                for p in points.iter().filter(|p| !p.1) {
                    if hit.contains(&class_of(spec, &p.0, line)?) && !via.contains(&p.2) {
                        via.push(p.2);
                    }
                }
            } else {
                untested = self.every(s);
                untested.retain(|v| !classes.contains(v));
                if untested.is_empty() {
                    set = open;
                }
            }
            untested.sort();
            classes.sort();
            if usable && !set.is_empty() {
                for w in s.claims.iter().filter(|w| !self.vouched(**w)) {
                    assume(&model.claims[*w].name);
                }
            }
            let wide = s.parts.len() > 1;
            let Target::Uniform(spec) = &model.claims[u].target else {
                unreachable!()
            };
            assumed = Some(Assumed {
                blind,
                tainted,
                by: model.claims[u].name.clone(),
                spec: spec.clone().filter(|_| !wide),
                calls: s.calls.clone(),
                unvouched: s
                    .claims
                    .iter()
                    .filter(|w| !self.vouched(**w))
                    .map(|w| model.claims[*w].name.clone())
                    .collect(),
                with: s.claims[1..]
                    .iter()
                    .map(|w| model.claims[*w].name.clone())
                    .collect(),
                vouched,
                usable,
                classes,
                untested,
                failing,
                set,
            });
        }
        let through = (via, relied);
        let with_assumed = match &assumed {
            Some(a) if a.usable => covered.union(&a.set),
            _ => covered.clone(),
        };
        // One tested input of a `represented` set stands for the set.
        let mut stood = Vec::new();
        let mut with_assumed = with_assumed;
        let own = !c.target.is_structural() && c.func != c.name;
        let subject = matches!(c.target, Target::Correct | Target::Matches(_));
        for (r, set, split) in self.stands.iter().filter(|_| own && subject) {
            let by = &model.claims[*r];
            let about = |e: &&Evidence| e.about == by.name;
            let says = |ok: bool| {
                let mut on = model.evidence.iter().filter(about);
                on.any(|e| e.judgment == Judgment::Holds(ok))
            };
            let inside = set.intersect(domain);
            let tested_there = !inside.intersect(&tested).is_empty();
            let fails_there = !inside.intersect(&refuted).is_empty();
            if by.func != c.func || split.is_some() || says(false) || !tested_there || fails_there {
                continue;
            }
            let more = inside.minus(&with_assumed);
            if more.is_empty() {
                continue;
            }
            with_assumed = with_assumed.union(&more);
            stood.push((by.name.clone(), more));
            if !says(true) && !assuming.contains(&by.name) {
                assuming.push(by.name.clone());
            }
        }
        // A vouched claim that reads `correct(…)` forces the judgement of
        // the inputs nothing judges.
        if own && subject && !ev.relations.is_empty() {
            let left = domain.minus(&with_assumed).minus(&refuted);
            let few = left.size_bound().is_some_and(|n| n <= 4096.0);
            ev.claim.replace(c.name.clone());
            for x in left.values().filter(|_| few).unwrap_or_default() {
                let Some((true, line)) = ev.forced(&c.func, &x, c.line).map_err(model_err)? else {
                    continue;
                };
                let by = ev
                    .relations
                    .iter()
                    .find(|r| r.line == line)
                    .expect("its own");
                let point = Tuples::point(&x);
                with_assumed = with_assumed.union(&point);
                match stood
                    .iter_mut()
                    .find(|s: &&mut (String, Tuples)| s.0 == by.claim)
                {
                    Some(s) => s.1 = s.1.union(&point),
                    None => stood.push((by.claim.clone(), point)),
                }
            }
        }
        // A sample that passed, drawn from every input nothing judges:
        // the claim holds as far as the sample is trusted.
        let left = domain.minus(&with_assumed).minus(&refuted);
        if model.settings.counts && refuted.is_empty() && !left.is_empty() {
            let whole = c.target == Target::Correct;
            let mut rest = left;
            for e in &model.evidence {
                let Judgment::Sampled {
                    ok: true,
                    over: Some(over),
                    ..
                } = &e.judgment
                else {
                    continue;
                };
                if e.about == c.name || (whole && e.about == c.func) {
                    rest = match over.is_empty() {
                        true => rest.minus(domain),
                        false => rest.minus(&model.covers(&c.func, over)),
                    };
                }
            }
            if rest.is_empty() {
                with_assumed = domain.clone();
                assuming.push(format!("{}::sample", c.name));
            }
        }
        let split = self.stands.iter().find(|s| s.0 == i).and_then(|s| s.2);
        let mut logic = if !refuted.is_empty() || split.is_some() || !self.drawn(c).is_empty() {
            Logic::Fails
        } else if !with_assumed.same(domain) {
            Logic::Unverified
        } else if assuming.is_empty() {
            Logic::Holds
        } else {
            Logic::Assuming
        };
        if self.broken(&c.name) {
            logic = Logic::Fails;
            for k in self.conflicts.iter().filter(|k| k.claim == c.name) {
                for x in [&k.pass.0, &k.fail.0] {
                    refuted = refuted.union(&Tuples::point(x));
                }
            }
        }
        // A requirement in prose fails when a claim it rests on fails, or a
        // requirement it needs.
        if let Some(e) = model.ensure(&c.name) {
            let mut rests = e.claims.iter().chain(&e.requires);
            if rests.any(|p| self.logic_of[p.as_str()] == Logic::Fails) {
                logic = Logic::Fails;
            }
        }
        if logic != Logic::Assuming {
            assuming.clear();
        }
        let verdict = Verdict {
            stood,
            against,
            claim: c.name.clone(),
            logic,
            assuming,
            missed: domain.minus(&with_assumed).minus(&refuted),
            tested,
            derived,
            follows,
            refuted,
            assumed,
            unsorted,
            domain: domain.clone(),
            implication,
        };
        Ok((verdict, through))
    }
}
