//! Dempster–Shafer on the binary frame {correct, wrong}.
//!
//! A [`Mass`] splits belief between *correct* (`b`), *wrong* (`d`) and
//! *unknown* (`u`), with `b + d + u = 1`. A piece of evidence at
//! confidence c gives `c` to its side and `1 − c` to unknown. Two masses
//! combine by Dempster's rule (conflict renormalised away, undefined at
//! total conflict) or by Yager's rule (conflict moved to unknown, and
//! reported).
//!
//! The crate is dogfooded: its model is aside, in `.mitra/model.mitra`
//! and `.mitra/points.mitra`. A comment `// mitra: implements mass::judged`
//! above a function ties it to the model, `mitra tie` checks that the two
//! agree, and `.mitra/evidence.mitra` holds the results of its own test
//! runs.

pub mod culprits;

/// Belief split on {correct, wrong}: `b + d + u = 1`, each in [0, 1].
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mass {
    /// Belief the subject is correct.
    pub b: f64,
    /// Belief it is wrong.
    pub d: f64,
    /// Belief left unassigned.
    pub u: f64,
}

/// Rounding slack when checking that a mass sums to 1.
pub const EPSILON: f64 = 1e-9;

impl Mass {
    /// Nothing known.
    pub const VACUOUS: Mass = Mass {
        b: 0.0,
        d: 0.0,
        u: 1.0,
    };

    /// The mass of one piece of evidence: `correct` or wrong, at confidence
    /// `c` in [0, 1]. `None` if `c` is outside [0, 1] or not a number.
    // mitra: implements mass::judged
    pub fn judged(ok: bool, c: f64) -> Option<Mass> {
        if !(0.0..=1.0).contains(&c) {
            return None;
        }
        let (b, d) = if ok { (c, 0.0) } else { (0.0, c) };
        Some(Mass { b, d, u: 1.0 - c })
    }

    /// Each part in [0, 1] and the parts sum to 1, up to [`EPSILON`].
    // mitra: implements mass::is_valid
    pub fn is_valid(&self) -> bool {
        let parts = [self.b, self.d, self.u];
        parts.iter().all(|x| (-EPSILON..=1.0 + EPSILON).contains(x))
            && (parts.iter().sum::<f64>() - 1.0).abs() <= EPSILON
    }

    /// The mass two sources give to opposite sides: `b₁d₂ + d₁b₂`.
    // mitra: implements mass::conflict
    pub fn conflict(&self, o: &Mass) -> f64 {
        self.b * o.d + self.d * o.b
    }

    /// The parts both sources agree on, before any normalisation.
    ///
    fn agreement(&self, o: &Mass) -> Mass {
        Mass {
            b: self.b * o.b + self.b * o.u + self.u * o.b,
            d: self.d * o.d + self.d * o.u + self.u * o.d,
            u: self.u * o.u,
        }
    }

    /// Dempster's rule: the conflict is renormalised away. `None` at total
    /// conflict (two fully believed, opposite judgments), taken as a
    /// conflict within [`EPSILON`] of 1 (renormalising by less is
    /// numerically meaningless), and when either mass is not valid.
    // mitra: implements mass::dempster
    pub fn dempster(&self, o: &Mass) -> Option<Mass> {
        if !self.is_valid() || !o.is_valid() {
            return None;
        }
        let k = self.conflict(o);
        if k >= 1.0 - EPSILON {
            return None;
        }
        let a = self.agreement(o);
        Some(Mass {
            b: a.b / (1.0 - k),
            d: a.d / (1.0 - k),
            u: a.u / (1.0 - k),
        })
    }

    /// Yager's rule: the conflict goes to unknown. Returns the combined
    /// mass and the conflict; `None` if either mass is not valid.
    // mitra: implements mass::yager
    pub fn yager(&self, o: &Mass) -> Option<(Mass, f64)> {
        if !self.is_valid() || !o.is_valid() {
            return None;
        }
        let k = self.conflict(o);
        let a = self.agreement(o);
        Some((Mass { u: a.u + k, ..a }, k))
    }
}
