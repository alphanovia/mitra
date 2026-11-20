//! Step 4: errors. Each failing claim, and each set of pieces that can't
//! all be true, with the pieces that may be the wrong one.

use super::*;

impl Judge<'_> {
    /// What a subject uses, directly or not, by the `uses` claims that do
    /// not fail.
    pub(super) fn uses_of<'b>(&'b self, f: &'b String) -> Vec<&'b String> {
        let model = self.model;
        let mut used: Vec<&String> = Vec::new();
        let mut todo = vec![f];
        while let Some(g) = todo.pop() {
            for c in model.claims.iter().filter(|c| &c.func == g) {
                let Target::Uses { used: list, .. } = &c.target else {
                    continue;
                };
                if self.logic_of[c.name.as_str()] == Logic::Fails {
                    continue;
                }
                for h in list {
                    if h != f && !used.contains(&h) {
                        used.push(h);
                        todo.push(h);
                    }
                }
            }
        }
        used.sort();
        used
    }

    fn is_subject(&self, f: &str) -> bool {
        self.model.function(f).is_some_and(|f| !f.is_spec())
    }

    fn quoted(&self, line: usize) -> (usize, String) {
        (line, self.shown(line))
    }

    pub(super) fn shown(&self, line: usize) -> String {
        evidence::shown(self.model, line)
    }

    pub(super) fn problems(&self) -> Result<Vec<Problem>, Fail> {
        let mut problems: Vec<Problem> = Vec::new();
        self.failures(&mut problems);
        self.misfits(&mut problems);
        self.measures(&mut problems)?;
        self.conflicting(&mut problems);
        rank::rank(self.model, &mut problems);
        // The same observation recorded twice gives the same error twice.
        let mut once: Vec<Problem> = Vec::new();
        for p in problems {
            if !once
                .iter()
                .any(|o| (&o.kind, &o.clauses) == (&p.kind, &p.clauses))
            {
                once.push(p);
            }
        }
        Ok(once)
    }

    /// Each claim that fails, with what may be wrong: the code, a helper,
    /// a tie, the claim itself, or the evidence against it.
    fn failures(&self, problems: &mut Vec<Problem>) {
        let model = self.model;
        for (i, c) in model.claims.iter().enumerate() {
            let v = &self.verdicts[i];
            if v.logic != Logic::Fails || self.broken(&c.name) {
                continue;
            }
            // A requirement that fails with its claims: their errors say why.
            if model.ensure(&c.name).is_some() && v.refuted.is_empty() {
                continue;
            }
            if let Some((_, _, Some((pass, fail)))) = self.stands.iter().find(|s| s.0 == i) {
                // The two inputs did not behave the same: the claim is denied.
                let mut pieces = vec![Piece::Evidence(*pass), Piece::Evidence(*fail)];
                let vouches = model.evidence.iter().filter(|e| e.about == c.name);
                pieces.extend(vouches.map(|e| Piece::Evidence(e.line)));
                pieces.push(Piece::Claim(c.name.clone()));
                problems.push(Problem {
                    kind: Kind::Denied {
                        claim: c.name.clone(),
                    },
                    evidence: vec![self.quoted(*pass), self.quoted(*fail)],
                    clauses: vec![pieces],
                    candidates: Ok(Vec::new()),
                    conflict: 0.0,
                });
                continue;
            }
            if let Some(x) = &v.against {
                let mut pieces = vec![Piece::Claim(c.name.clone())];
                pieces.extend(c.from.iter().map(|p| Piece::Claim(p.clone())));
                problems.push(Problem {
                    kind: Kind::Against {
                        claim: c.name.clone(),
                        specs: c.from.clone(),
                        input: brief(&Tuples::point(x), &v.domain),
                    },
                    evidence: Vec::new(),
                    clauses: vec![pieces],
                    candidates: Ok(Vec::new()),
                    conflict: 0.0,
                });
                if self.direct[i].wrong.is_empty() {
                    continue;
                }
            }
            let mut lines: Vec<usize> = Vec::new();
            for j in (0..self.direct.len()).filter(|j| self.same(i, *j)) {
                for (set, ok, line) in &self.direct[j].sources {
                    if !ok && !set.intersect(&v.refuted).is_empty() {
                        lines.push(*line);
                    }
                }
            }
            // A sample that failed, on an input it does not state.
            lines.extend(self.drawn(c));
            lines.sort();
            lines.dedup();
            lines.retain(|l| *l > 0);
            // A failure that refutes the claim through its class does so with
            // the evidence vouching for the uniform claim.
            let (via, by) = &self.through[i];
            let via: Vec<usize> = via.iter().filter(|l| !lines.contains(l)).copied().collect();
            let mut vouches: Vec<usize> = Vec::new();
            for u in by.iter().filter(|_| !via.is_empty()) {
                for (_, ok, line) in &self.direct[*u].sources {
                    if *ok && !vouches.contains(line) {
                        vouches.push(*line);
                    }
                }
            }
            // What may be wrong on the model's side: the code of the subject,
            // of what it uses and of the subjects the claim calls, or the
            // claim itself.
            let mut side: Vec<Piece> = Vec::new();
            if let Target::Property(_) = &c.target {
                for g in model.subjects_of(c) {
                    for h in std::iter::once(&g).chain(self.uses_of(&g)) {
                        if !side.contains(&Piece::Code(h.clone())) {
                            side.push(Piece::Code(h.clone()));
                        }
                    }
                }
            } else if let Some((_, f, _)) = model.determinism.iter().find(|d| d.0 == c.name) {
                // The function itself, and the other output seen on the input.
                side.push(Piece::Code(f.clone()));
                let out =
                    |e: &&Evidence| &e.about == f && matches!(e.judgment, Judgment::Output(_));
                let outputs: Vec<&Evidence> = model.evidence.iter().filter(out).collect();
                for (n, e) in outputs.iter().enumerate() {
                    let differs =
                        |o: &&&Evidence| o.pattern == e.pattern && o.judgment != e.judgment;
                    if outputs[n + 1..].iter().any(|o| differs(&o)) {
                        side.push(Piece::Evidence(e.line));
                    }
                }
            } else if c.target.is_structural() || !self.is_subject(&c.func) {
            } else {
                side.push(Piece::Code(c.func.clone()));
                for g in self.uses_of(&c.func) {
                    side.push(Piece::Code(g.clone()));
                }
            }
            if let Target::Matches(e) = &c.target {
                for g in calls(e).into_iter().filter(|g| self.is_subject(g)) {
                    if !side.contains(&Piece::Code(g.clone())) {
                        side.push(Piece::Code(g));
                    }
                }
            }
            // A claim on a record: the code that built a value against it.
            if let (Target::Property(_), Some([(_, of)])) = (
                &c.target,
                model.function(&c.func).map(|f| f.params.as_slice()),
            ) {
                let denied = |l: usize| {
                    let no =
                        |e: &Evidence| e.about == c.name && e.judgment == Judgment::Holds(false);
                    model.evidence.iter().any(|e| e.line == l && no(e))
                };
                for e in &model.evidence {
                    let built = matches!(e.judgment, Judgment::Output(_))
                        && model
                            .declared_function(&e.about)
                            .is_some_and(|g| &g.range == of);
                    if built && denied(e.line) && !side.contains(&Piece::Code(e.about.clone())) {
                        side.push(Piece::Code(e.about.clone()));
                    }
                }
            }
            // The helpers the statement reads, and those they read: each is a
            // definition of the model that may be the wrong one.
            if let Target::Property(e) | Target::Matches(e) = &c.target {
                let helper = |g: &str| {
                    let f = model.declared_function(g)?;
                    (f.body.is_some() && !g.ends_with("::body")).then_some(f)
                };
                let mut todo = calls(e);
                let mut seen: Vec<String> = Vec::new();
                while let Some(g) = todo.pop() {
                    // The spec of a function of the code: read through.
                    let body = format!("{g}::body");
                    let read = helper(&g).or(model.declared_function(&body));
                    let Some(f) = read.filter(|_| !seen.contains(&g)) else {
                        continue;
                    };
                    seen.push(g.clone());
                    if f.name == g && !side.contains(&Piece::Helper(g.clone())) {
                        side.push(Piece::Helper(g));
                    }
                    todo.extend(f.body.as_ref().map(calls).unwrap_or_default());
                }
            }
            // A boundary has no code of ours to be wrong: it answers as it
            // does.
            side.retain(|p| !matches!(p, Piece::Code(g) if model.boundaries.contains(g)));
            // Where a function is tied to its code, the tie is a piece too:
            // model and code may have drifted apart behind it.
            if c.target != Target::Tied {
                let tied = |g: &String| {
                    let about = format!("{g}::tied");
                    let holds =
                        |e: &Evidence| e.about == about && e.judgment == Judgment::Holds(true);
                    model.evidence.iter().any(holds)
                };
                let code: Vec<String> = side
                    .iter()
                    .filter_map(|p| match p {
                        Piece::Code(g) if tied(g) => Some(g.clone()),
                        _ => None,
                    })
                    .collect();
                side.extend(code.into_iter().map(Piece::Tie));
            }
            // The statement of the claim is a piece like the others. For a
            // tie: what the model says of the function, or the code.
            if c.target == Target::Tied {
                // For a record: what the model says of it, or the struct.
                let of = c.func.strip_suffix("::tied::prose").unwrap_or(&c.func);
                side.push(Piece::Signature(of.to_string()));
                side.push(Piece::Code(of.to_string()));
            } else if let Some(e) = model.ensure(&c.name) {
                // Evidence against a requirement in prose: the prose, the word
                // that its claims say it, or a requirement it needs. Not code.
                side.clear();
                side.push(Piece::Requirement(e.name.clone()));
                if !e.boundary {
                    side.push(Piece::Capture(e.name.clone()));
                }
                // A promise of the world: each assumption that reads it.
                if e.world {
                    let word = |n: &String| {
                        let said = |x: &Evidence| x.source.as_deref() == Some("assumed");
                        model.evidence.iter().any(|x| &x.about == n && said(x))
                    };
                    let reads = e.claims.iter().filter(|n| word(n));
                    side.extend(reads.cloned().map(Piece::Assumption));
                }
                for r in &e.requires {
                    side.push(Piece::Requirement(r.clone()));
                    // Through a named need, the promise it names.
                    let named = model.ensure(r).and_then(|n| n.needs.clone());
                    side.extend(named.map(Piece::Requirement));
                }
            } else if let Some(e) = model.captured(&c.name) {
                side.clear();
                side.push(Piece::Capture(e.name.clone()));
            } else {
                side.push(Piece::Claim(c.name.clone()));
            }
            let with = |l: &usize| [side.clone(), vec![Piece::Evidence(*l)]].concat();
            let mut clauses: Vec<Vec<Piece>> = lines.iter().map(with).collect();
            if let Target::Property(_) = &c.target {
                // The outputs a failing input read are wrong together.
                clauses.clear();
                for (x, read) in &self.direct[i].joint {
                    if v.refuted.contains(x) {
                        let read = read.iter().map(|l| Piece::Evidence(*l));
                        let clause: Vec<Piece> = side.iter().cloned().chain(read).collect();
                        if !clauses.contains(&clause) {
                            clauses.push(clause);
                        }
                    }
                }
                // A claim refuted by evidence on the claim itself.
                let stated = |l: &&usize| {
                    let first = model.evidence.iter().find(|e| e.line == **l);
                    let said = |e: &Evidence| {
                        matches!(e.judgment, Judgment::Holds(_) | Judgment::Sampled { .. })
                    };
                    // Also what an observed output says of a claim on a record.
                    let on = |e: &&Evidence| e.line == **l && e.about == c.name && said(e);
                    first.is_some_and(said) || model.evidence.iter().any(|e| on(&e))
                };
                clauses.extend(lines.iter().filter(stated).map(with));
            }
            for l in &via {
                for w in &vouches {
                    clauses.push([with(l), vec![Piece::Evidence(*w)]].concat());
                }
            }
            if clauses.is_empty() {
                clauses.push(side.clone());
            }
            lines.extend(via);
            lines.extend(vouches);
            lines.sort();
            lines.dedup();
            let denied = c.target.is_structural();
            // Claims refuted by the same pieces share one error: the same
            // clauses, the statements of the claims apart.
            let apart = |clauses: &[Vec<Piece>]| -> Vec<Vec<Piece>> {
                let stated = |p: &&Piece| !matches!(p, Piece::Claim(_));
                let each = clauses.iter();
                each.map(|c| c.iter().filter(stated).cloned().collect())
                    .collect()
            };
            let merged = problems.iter_mut().find(|p| {
                matches!(p.kind, Kind::Fails { .. })
                    && !denied
                    && apart(&p.clauses) == apart(&clauses)
            });
            if let Some(p) = merged {
                if let Kind::Fails { claims } = &mut p.kind {
                    claims.push(c.name.clone());
                }
                p.clauses.extend(clauses);
                continue;
            }
            let name = c.name.clone();
            problems.push(Problem {
                kind: if denied {
                    Kind::Denied { claim: name }
                } else {
                    Kind::Fails { claims: vec![name] }
                },
                evidence: lines.into_iter().map(|l| self.quoted(l)).collect(),
                clauses,
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
        }
    }

    /// Evidence that fits no signature, and evidence that contradicts
    /// itself.
    fn misfits(&self, problems: &mut Vec<Problem>) {
        let model = self.model;
        for (e, why) in &model.misfits {
            let written = model.written.as_ref();
            let text = written.and_then(|w| crate::parse::written(&w.1, e.line));
            let text = text.unwrap_or_else(|| {
                let old = crate::print::print_evidence(std::slice::from_ref(e), Some(model));
                old.trim_end().to_string()
            });
            problems.push(Problem {
                kind: Kind::Misfit {
                    function: e.about.clone(),
                    why: why.clone(),
                },
                evidence: vec![(e.line, text)],
                clauses: vec![vec![
                    Piece::Evidence(e.line),
                    Piece::Signature(e.about.clone()),
                ]],
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
        }
        for k in &self.contradictions {
            problems.push(Problem {
                kind: Kind::Contradiction {
                    subject: k.subject.clone(),
                    inputs: k.inputs.clone(),
                },
                evidence: vec![k.pass.clone(), k.fail.clone()],
                clauses: vec![vec![Piece::Evidence(k.pass.0), Piece::Evidence(k.fail.0)]],
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
        }
    }

    /// A measurement against another one of the same measure, and against
    /// what the observations count.
    fn measures(&self, problems: &mut Vec<Problem>) -> Result<(), Fail> {
        let (model, ev) = (self.model, &self.ev);
        let measurements: Vec<(&Evidence, &Expr, f64)> = model
            .evidence
            .iter()
            .filter_map(|e| match &e.judgment {
                Judgment::Measured(m, v) => Some((e, m, *v)),
                _ => None,
            })
            .collect();
        for (n, (e, measure, v)) in measurements.iter().enumerate() {
            for (o, _, w) in measurements[..n].iter().filter(|o| o.0.about == e.about) {
                if w != v {
                    problems.push(Problem {
                        kind: Kind::Measure {
                            measure: e.about.clone(),
                            counted: String::new(),
                        },
                        evidence: vec![self.quoted(o.line), self.quoted(e.line)],
                        clauses: vec![vec![Piece::Evidence(o.line), Piece::Evidence(e.line)]],
                        candidates: Ok(Vec::new()),
                        conflict: 0.0,
                    });
                }
            }
            ev.read.borrow_mut().clear();
            ev.measured.borrow_mut().clear();
            let count = ev
                .count(measure, e.line)
                .map_err(|x| (Input::Evidence, x))?;
            let (lo, hi) = count.bounds;
            if *v < lo - 1e-9 || *v > hi + 1e-9 {
                // The outputs that were counted, with the measurement.
                let read = ev.read.take();
                let mut lines: Vec<usize> = model
                    .evidence
                    .iter()
                    .filter(|o| {
                        matches!(o.judgment, Judgment::Output(_))
                            && read.iter().any(|(g, x)| {
                                &o.about == g && single(&o.pattern).as_ref() == Some(x)
                            })
                    })
                    .map(|o| o.line)
                    .collect();
                lines.push(e.line);
                lines.extend(ev.measured.take());
                lines.sort();
                lines.dedup();
                problems.push(Problem {
                    kind: Kind::Measure {
                        measure: e.about.clone(),
                        counted: count.text.clone(),
                    },
                    clauses: vec![lines.iter().map(|l| Piece::Evidence(*l)).collect()],
                    evidence: lines.into_iter().map(|l| self.quoted(l)).collect(),
                    candidates: Ok(Vec::new()),
                    conflict: 0.0,
                });
            }
        }
        Ok(())
    }

    /// Each conflict in a class, with the evidence vouching for the
    /// `uniform` claims it rests on.
    fn conflicting(&self, problems: &mut Vec<Problem>) {
        let model = self.model;
        for k in &self.conflicts {
            let u = model.claims.iter().position(|c| c.name == k.claim);
            let u = u.expect("self.conflicts name claims");
            // The claim stands through the evidence vouching for it: a vouch
            // is a piece where it covers both inputs.
            let mut clauses: Vec<Vec<Piece>> = Vec::new();
            let mut lines: Vec<usize> = Vec::new();
            for (x, pass) in &k.passes {
                for (y, fail) in &k.fails {
                    let rest = [Piece::Evidence(*pass), Piece::Evidence(*fail)];
                    let mut vouched = false;
                    for (set, ok, line) in &self.direct[u].sources {
                        if *ok && set.contains(x) && set.contains(y) {
                            vouched = true;
                            clauses.push([vec![Piece::Evidence(*line)], rest.to_vec()].concat());
                            lines.push(*line);
                        }
                    }
                    if !vouched {
                        clauses.push([vec![Piece::Claim(k.claim.clone())], rest.to_vec()].concat());
                    }
                }
            }
            // The other `uniform` claims it rests on: each by what vouches for
            // it, or the claim itself.
            for w in &k.with {
                let w = model.claims.iter().position(|c| &c.name == w);
                let w = w.expect("self.conflicts name claims");
                let mut pieces: Vec<Piece> = Vec::new();
                for (_, ok, line) in &self.direct[w].sources {
                    if *ok && !pieces.contains(&Piece::Evidence(*line)) {
                        pieces.push(Piece::Evidence(*line));
                        lines.push(*line);
                    }
                }
                if pieces.is_empty() {
                    pieces.push(Piece::Claim(model.claims[w].name.clone()));
                }
                clauses = clauses
                    .iter()
                    .flat_map(|c| pieces.iter().map(|p| [c.clone(), vec![p.clone()]].concat()))
                    .collect();
            }
            clauses.sort_by_key(|c| format!("{c:?}"));
            clauses.dedup();
            lines.sort();
            lines.dedup();
            let mut evidence: Vec<(usize, String)> =
                lines.into_iter().map(|l| self.quoted(l)).collect();
            for (_, l) in k.passes.iter().chain(&k.fails) {
                if !evidence.iter().any(|e| e.0 == *l) {
                    evidence.push(self.quoted(*l));
                }
            }
            problems.push(Problem {
                kind: Kind::Conflict {
                    claim: k.claim.clone(),
                    spec: k.spec.clone(),
                    class: k.class.clone(),
                    calls: k.calls.clone(),
                },
                evidence,
                clauses,
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
        }
    }
}
