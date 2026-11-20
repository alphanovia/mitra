//! An error as text: the pieces that can't all be right, ranked.

use super::*;

impl Writer<'_> {
    /// The evidence of an error, one per line, with what each says.
    pub(super) fn quoted(&self, p: &Problem, out: &mut String) {
        for (line, _) in p.evidence.iter().take(QUOTED) {
            *out += &format!(
                "    {:<9} {}  {}\n",
                self.role(*line),
                self.evidence_at(*line),
                self.evidence_text(*line)
            );
        }
        if p.evidence.len() > QUOTED {
            *out += &format!("    and {} more\n", p.evidence.len() - QUOTED);
        }
    }

    /// A piece, and the trust given to it.
    pub(super) fn piece(&self, piece: &Piece) -> (String, String) {
        match piece {
            Piece::Evidence(line) => {
                let e = self.entry(*line);
                let source = e.and_then(|e| e.source.as_ref());
                let source = source.map_or(String::new(), |s| format!(" ({s})"));
                let trust = e.map_or(0.0, |e| e.confidence);
                let text = match self.declared(*line) {
                    Some(c) => format!("`{}` as{source}, {}", c.name, self.evidence_at(*line))
                        .replace("as (", "as ")
                        .replace("),", ","),
                    None => format!("{}{source}", self.evidence_at(*line)),
                };
                (text, format!("{trust}"))
            }
            Piece::Code(f) => {
                if let Some(r) = self
                    .model
                    .record(f)
                    .filter(|_| self.model.function(f).is_none())
                {
                    let place = self.places.model(r.line);
                    return (format!("the struct of `{f}`, {place}"), "none".to_string());
                }
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                (format!("the code of `{f}`, {place}"), "none".to_string())
            }
            Piece::Claim(c) => {
                let line = self.claim(c).line;
                let place = self.places.model(line);
                let trust = self.claim(c).trust;
                (
                    format!("the claim `{c}` itself, {place}"),
                    format!("{}", trust.unwrap_or(1.0)),
                )
            }
            Piece::Signature(f) => {
                if let Some(r) = self
                    .model
                    .record(f)
                    .filter(|_| self.model.function(f).is_none())
                {
                    let place = self.places.model(r.line);
                    return (
                        format!("the record `{f}` as written, {place}"),
                        "1".to_string(),
                    );
                }
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                (format!("the signature of `{f}`, {place}"), "1".to_string())
            }
            Piece::Helper(f) => {
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                let said = self.model.helper_trust.iter().find(|h| &h.0 == f);
                let trust = said.map_or(1.0, |h| h.1);
                (
                    format!("the helper `{f}` as written, {place}"),
                    format!("{trust}"),
                )
            }
            Piece::Tie(f) => {
                let line = self.model.function(f).map_or(0, |f| f.line);
                let place = self.places.model(line);
                let about = format!("{f}::tied");
                let said = self.model.evidence.iter().filter(|e| e.about == about);
                let trust = said.clone().map(|e| e.confidence).reduce(f64::max);
                // What the reader of the code did not compare.
                let open = said
                    .filter_map(|e| e.source.as_deref()?.split_once("; not compared: "))
                    .map(|s| format!(" (not compared: {})", s.1))
                    .next()
                    .unwrap_or_default();
                (
                    format!("that `{f}` and its code agree{open}, {place}"),
                    trust.map_or("none".to_string(), |t| format!("{t}")),
                )
            }
            Piece::Requirement(c) => {
                let place = self.places.model(self.claim(c).line);
                let needs = self.model.ensure(c).and_then(|e| e.needs.clone());
                let text = match needs {
                    Some(n) => format!("that `{n}` is what `{c}` needs, {place}"),
                    None => format!("the text of `{c}`, {place}"),
                };
                let trust = self.claim(c).trust;
                (text, trust.map_or("none".to_string(), |t| format!("{t}")))
            }
            Piece::Capture(c) => {
                let place = self.places.model(self.claim(c).line);
                let text = format!("that the claims say what `{c}` says, {place}");
                let about = format!("{c}::captured");
                let vouches = self.model.evidence.iter().filter(|e| e.about == about);
                let trust = vouches.map(|e| e.confidence).reduce(f64::max);
                (text, trust.map_or("none".to_string(), |t| format!("{t}")))
            }
            Piece::Assumption(_) | Piece::Uses(_) => {
                let name = piece.declared_claim().expect("declared");
                let how = match piece {
                    Piece::Uses(_) => "declared",
                    _ => "assumed",
                };
                let place = self.places.model(self.claim(&name).line);
                let trust = self.stated(&name).map_or(1.0, |e| e.confidence);
                (format!("`{name}` as {how}, {place}"), format!("{trust}"))
            }
        }
    }

    /// Which piece is most plausibly wrong: the trust given to each, and
    /// the trust left once they are weighed against each other.
    pub(super) fn ranking(&self, p: &Problem, out: &mut String) {
        let candidates = match &p.candidates {
            Ok(c) => c,
            Err(Unranked::TooMany) => {
                *out += "\n  These can't all be right. More than 20 of them would have to be\n  \
                         weighed against each other: too many to rank.\n";
                return;
            }
        };
        let prose = |c: &String| self.model.ensure(c).is_some();
        *out += match &p.kind {
            Kind::Fails { claims } if claims.iter().any(prose) => {
                "\n  The text, the word that the claims say it, what it rests on or requires,\n  or the evidence is wrong."
            }
            Kind::Fails { .. } => "\n  The code, the claim or the evidence is wrong.",
            Kind::Misfit { .. } => "\n  Either the model is wrong or the evidence is.",
            Kind::Against { .. } => "\n  The claim or a spec is wrong.",
            _ => "\n  These can't all be right.",
        };
        *out += " Below, the trust given to each\n  piece, and what is left of it once they are weighed against each other:\n\n";
        let shown = |c: &&Candidate| c.plausibility >= self.display.threshold;
        let listed: Vec<&Candidate> = candidates
            .iter()
            .filter(shown)
            .take(self.display.max)
            .collect();
        let rows: Vec<(String, String, String)> = listed
            .iter()
            .map(|c| {
                let (piece, trust) = self.piece(&c.piece);
                // What is left: from "it can be wrong this much" to "it
                // must be wrong this much".
                let (low, high) = (1.0 - c.plausibility, 1.0 - c.belief);
                let left = if (high - low).abs() < 0.005 {
                    format!("{low:.2}")
                } else {
                    format!("{low:.2} to {high:.2}")
                };
                (piece, trust, left)
            })
            .collect();
        let wide = |f: fn(&(String, String, String)) -> &String, head: &str| {
            let cells = rows.iter().map(|r| f(r).chars().count());
            cells.chain([head.len()]).max().unwrap_or(0)
        };
        let (w0, w1) = (wide(|r| &r.0, ""), wide(|r| &r.1, "trust"));
        *out += &format!("    {:<9} {:<w0$}  {:<w1$}  left\n", "", "", "trust");
        for (c, (piece, trust, left)) in listed.iter().zip(&rows) {
            *out += &format!("    {:<9} {piece:<w0$}  {trust:<w1$}  {left}\n", word(c));
        }
        let possible = candidates.iter().filter(|c| c.plausibility > 0.0).count();
        if possible > listed.len() {
            *out += &format!(
                "    {} less plausible not shown (`--all` shows them)\n",
                possible - listed.len()
            );
        }
        // The model's own definitions and its ties to the code, where
        // nothing doubts them: named all the same.
        let quiet = |c: &&Candidate| {
            let own = matches!(c.piece, Piece::Helper(_) | Piece::Tie(_));
            own && !listed.iter().any(|l| l.piece == c.piece)
        };
        for c in candidates.iter().filter(quiet) {
            let (piece, _) = self.piece(&c.piece);
            *out += &format!("    {:<9} {piece}\n", "trusted");
        }
        *out += "\n  The lower what is left, the more plausibly the piece is the wrong one.\n";
        if listed.iter().any(|c| c.belief + 0.005 < c.plausibility) {
            *out += "  A range means the evidence does not settle it.\n";
        }
        // Pieces trusted at 1 that are doubted all the same.
        let sure = |c: &&&Candidate| match &c.piece {
            Piece::Evidence(line) => {
                let e = self.model.evidence.iter().find(|e| e.line == *line);
                e.is_some_and(|e| e.confidence >= 1.0) && c.plausibility > 0.0
            }
            Piece::Assumption(_) | Piece::Uses(_) => {
                let name = c.piece.declared_claim().expect("declared");
                self.stated(&name).is_some_and(|e| e.confidence >= 1.0) && c.plausibility > 0.0
            }
            _ => false,
        };
        if listed.iter().filter(sure).count() > 0 {
            *out += "\n  note: the pieces trusted at 1 can't all be right, so their trust says\n  \
                     nothing here: Mitra shares the doubt equally between them. Lower the\n  \
                     trust of the one you doubt most to set them apart.\n";
        } else if p.conflict >= 0.5 && !matches!(p.kind, Kind::Fails { .. }) {
            *out += "\n  note: the trust given says these are probably all right, which can't\n  \
                     be: at least one is trusted more than it deserves\n";
        }
    }

    pub(super) fn error(&self, p: &Problem, attached: &mut Vec<String>, out: &mut String) {
        let code = p.kind.code();
        *out += &format!("\nerror[{code}]: {}\n", headline(code));
        match &p.kind {
            Kind::Fails { claims } => {
                for name in claims {
                    *out += &self.at_claim(name);
                }
                *out += "\n";
                for name in claims {
                    let (c, v) = (self.claim(name), self.verdict(name));
                    if self.model.ensure(name).is_some() {
                        *out += &format!("  `{name}` is not met.\n");
                        continue;
                    }
                    if v.refuted.is_empty() {
                        *out += &format!(
                            "  `{name}` fails on an input a sample drew and does not state.\n"
                        );
                        continue;
                    }
                    *out += &format!(
                        "  `{name}` fails on {}.\n",
                        self.inputs(&c.func, &v.refuted, &v.domain)
                    );
                }
                if p.evidence.is_empty() {
                    *out += "  Its two sides are definitions, and they differ there.\n";
                } else {
                    *out += "  The evidence that says so:\n\n";
                    self.quoted(p, out);
                }
                self.ranking(p, out);
                // The subjects the claim is about: one, or for a property
                // those it calls.
                let mut subjects = self.model.subjects_of(self.claim(&claims[0]));
                subjects.retain(|f| !self.model.boundaries.contains(f));
                subjects.retain(|f| !f.ends_with("::prose"));
                subjects.retain(|f| {
                    self.model
                        .declared_function(f)
                        .is_some_and(|f| !f.is_spec())
                });
                *out += "\n";
                for f in &subjects {
                    let suspects = self.report.suspects.iter().filter(|s| &s.subject == f);
                    for s in suspects {
                        *out += &format!(
                            "  help: look first at `{}`, which `{f}` uses: {} fail{} there\n",
                            s.uses,
                            s.failing.join(", "),
                            if s.failing.len() == 1 { "s" } else { "" }
                        );
                        attached.push(format!("suspect {f} {}", s.uses));
                    }
                    for r in self.report.roots.iter().filter(|r| &r.subject == f) {
                        *out += &format!("  note: {}\n", r.text());
                        attached.push(format!("root {f}"));
                    }
                }
                *out += &if self.model.ensure(&claims[0]).is_some_and(|e| e.world) {
                    "  help: if the evidence is right, the world did not keep its promise, or the
  \
                     assumptions read more into it than it says; if not, correct the evidence\n"
                        .to_string()
                } else if self.model.ensure(&claims[0]).is_some() {
                    "  help: if the evidence is right, the claims miss what the text says: write the\n  \
                     claim that would have caught it; if not, correct the evidence\n"
                        .to_string()
                } else if !subjects.is_empty() {
                    let f = subjects.join("` or `");
                    format!(
                        "  help: if the evidence is right, fix `{f}`; if not, correct the evidence\n"
                    )
                } else {
                    "  help: correct the claim or the definitions it compares\n".to_string()
                };
            }
            Kind::Denied { claim } if self.claim(claim).target == crate::Target::Tied => {
                let f = &self.claim(claim).func;
                *out += &self.at_claim(claim);
                *out += &match f.strip_suffix("::tied::prose") {
                    Some(r) if self.model.enums.iter().any(|e| e.name == r) => format!(
                        "\n  The enum `{r}` and the enum of the code do not agree, says this evidence:\n\n"
                    ),
                    Some(r) => format!(
                        "\n  The record `{r}` and its struct do not agree, says this evidence:\n\n"
                    ),
                    None => format!(
                        "\n  The model and the code of `{f}` do not agree, says this evidence:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: change the model or the code so that they say the same\n";
            }
            Kind::Denied { claim } => {
                *out += &self.at_claim(claim);
                let split = matches!(self.claim(claim).target, crate::Target::Represented(_));
                *out += &match split {
                    true => format!(
                        "\n  `{claim}` says one input stands for the others, and two of them\n  \
                         do not behave the same:\n\n"
                    ),
                    false => format!(
                        "\n  The model states `{claim}`, and evidence says it does not hold:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: correct or remove the claim, or correct the evidence\n";
            }
            Kind::Contradiction { subject, inputs } => {
                if let Some((line, _)) = p.evidence.first() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &match subject.strip_suffix("::prose") {
                    Some(name) => format!("\n  On `{name}`, these two say opposite things:\n\n"),
                    None => format!(
                        "\n  On the inputs {inputs} of `{subject}`, these two say opposite things:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  note: until this is settled, neither counts for these inputs\n  \
                         help: find out which one is right, and correct the other\n";
            }
            Kind::Against { claim, specs, .. } => {
                *out += &self.at_claim(claim);
                let (c, v) = (self.claim(claim), self.verdict(claim));
                let at = v.against.as_ref().map(|x| Tuples::point(x));
                let input = at.map_or(String::new(), |x| self.inputs(&c.func, &x, &v.domain));
                let specs: Vec<String> = specs.iter().map(|s| format!("`{s}`")).collect();
                *out += &format!(
                    "\n  On {input}, the specs of the functions `{claim}` calls say it is false\n  \
                     ({}). No evidence is needed to see it.\n",
                    specs.join(", ")
                );
                self.ranking(p, out);
                *out += "\n  help: correct the claim, or the body of the function it contradicts\n";
            }
            Kind::Misfit { function, why } => {
                if let Some((line, _)) = p.evidence.first() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &format!(
                    "\n  This evidence does not fit the signature of `{function}`: {why}.\n\n"
                );
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  note: until this is settled, the evidence judges nothing\n";
                *out += &format!(
                    "  help: if the observation is real, change the signature of \
                     `{function}`; if not, correct the evidence\n"
                );
            }
            Kind::Measure { measure, counted } => {
                if let Some((line, _)) = p.evidence.last() {
                    *out += &format!("  --> {}\n", self.evidence_at(*line));
                }
                *out += &match counted.is_empty() {
                    true => format!("\n  `{measure}` was measured with two values:\n\n"),
                    false => format!(
                        "\n  `{measure}` as measured does not fit what was observed \
                         ({counted}):\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                *out += "\n  help: find out which one is right, and correct the other\n";
            }
            Kind::Conflict {
                claim,
                spec,
                class,
                calls,
            } => {
                let found = self.report.conflicts.iter();
                let on = found
                    .filter(|k| &k.claim == claim && &k.class == class && &k.calls == calls)
                    .find_map(|k| k.on.clone());
                let func = &self.claim(claim).func;
                *out += &self.at_claim(claim);
                *out += &match (spec, &on) {
                    (_, Some(on)) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input{}.\n  \
                         So `{on}`, which calls `{func}`, should pass or fail the same way{}.\n  \
                         But some inputs pass and others fail:\n\n",
                        if calls.is_empty() { "" } else { " of a class" },
                        match calls.is_empty() {
                            true => String::new(),
                            false => format!(
                                " for\n  {}",
                                crate::verdicts::class_text(std::slice::from_ref(class), spec, calls)
                            ),
                        }
                    ),
                    (Some(spec), None) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input of class {class}\n  \
                         of `{spec}`. But in that class some inputs pass and others fail:\n\n"
                    ),
                    (None, None) => format!(
                        "\n  `{claim}` says `{func}` behaves the same for every input. But some inputs\n  \
                         pass and others fail:\n\n"
                    ),
                };
                self.quoted(p, out);
                self.ranking(p, out);
                let found = self.report.conflicts.iter();
                let mut found = found.filter(|k| &k.claim == claim && &k.class == class);
                let fails = found
                    .next()
                    .map(|k| call(on.as_deref().unwrap_or(func), &k.fail.0));
                let fails = fails.unwrap_or_else(|| "the failing input".to_string());
                *out += &match spec {
                    _ if !calls.is_empty() => format!(
                        "\n  help: check that `{fails}` really fails; if it does, a class\n  \
                         is too wide: split it\n"
                    ),
                    Some(spec) => format!(
                        "\n  help: check that `{fails}` really fails; if it does, class {class} is\n  \
                         too wide: split it in `{spec}`\n"
                    ),
                    None => format!(
                        "\n  help: check that `{fails}` really fails; if it does, sort the\n  \
                         inputs of `{func}` into classes: claim {claim} = uniform({func}, <classes>);\n"
                    ),
                };
            }
        }
    }
}
