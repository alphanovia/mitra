//! Step 5: suspects and root causes, from what each subject uses.

use super::*;

impl Judge<'_> {
    /// The claims on a subject that fail.
    fn failing(&self, subject: &str) -> Vec<String> {
        let model = self.model;
        let fails = |c: &&Claim| {
            model.subjects_of(c).iter().any(|g| g == subject)
                && !matches!(c.target, Target::Uses { .. })
                && self.logic_of[c.name.as_str()] == Logic::Fails
        };
        model
            .claims
            .iter()
            .filter(fails)
            .map(|c| c.name.clone())
            .collect()
    }

    /// Suspects: what a subject uses, directly or not, that fails.
    pub(super) fn suspects(&self) -> Vec<Suspect> {
        let model = self.model;
        let mut suspects = Vec::new();
        for f in model.functions.iter().filter(|f| !f.is_spec()) {
            for g in self.uses_of(&f.name) {
                let failing = self.failing(g);
                if !failing.is_empty() {
                    suspects.push(Suspect {
                        subject: f.name.clone(),
                        uses: g.clone(),
                        failing,
                    });
                }
            }
        }
        suspects
    }

    /// Root causes: a failing subject that only uses subjects where
    /// nothing fails.
    pub(super) fn roots(&self, suspects: &[Suspect]) -> Vec<Root> {
        let model = self.model;
        let mut roots: Vec<Root> = Vec::new();
        for (i, c) in model.claims.iter().enumerate() {
            let Target::Uses { used, only: true } = &c.target else {
                continue;
            };
            let suspect = suspects.iter().any(|s: &Suspect| s.subject == c.func);
            // The subject itself must fail, not only a claim on its structure.
            let fails = model.claims.iter().any(|d| {
                model.subjects_of(d).contains(&c.func)
                    && !d.target.is_structural()
                    && self.logic_of[d.name.as_str()] == Logic::Fails
            });
            let named = roots.iter().any(|r| r.subject == c.func);
            if !fails || suspect || named || !self.vouched(i) {
                continue;
            }
            // Every other `uses` claim on the subject that is not denied counts
            // too, vouched for or not.
            let mut all: Vec<&String> = Vec::new();
            for d in model.claims.iter().filter(|d| d.func == c.func) {
                if let Target::Uses { used, .. } = &d.target {
                    if self.logic_of[d.name.as_str()] != Logic::Fails {
                        all.extend(used);
                    }
                }
            }
            // A failure in what it uses that no claim carries: not a root cause.
            if all.iter().any(|g| self.judged(g).iter().any(|p| !p.1)) {
                continue;
            }
            // Every claim on what it uses holds: the fault can only be here.
            let covered = all.iter().all(|g| {
                let mut on = model.claims.iter().filter(|d| &&d.func == g);
                let mut on = on.by_ref().filter(|d| !d.target.is_structural());
                let mut any = false;
                on.all(|d| {
                    any = true;
                    self.logic_of[d.name.as_str()] == Logic::Holds
                }) && any
            });
            roots.push(Root {
                subject: c.func.clone(),
                by: c.name.clone(),
                used: used.clone(),
                covered,
            });
        }
        roots
    }
}
