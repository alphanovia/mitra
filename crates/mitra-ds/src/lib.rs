//! Dempster–Shafer on the binary frame {correct, wrong}.
//!
//! A [`Mass`] splits belief between *correct* (`b`), *wrong* (`d`) and
//! *unknown* (`u`), with `b + d + u = 1`. A piece of evidence at
//! confidence c gives `c` to its side and `1 − c` to unknown. Two masses
//! combine by Dempster's rule (conflict renormalised away, undefined at
//! total conflict) or by Yager's rule (conflict moved to unknown, and
//! reported).
//!
//! The crate is dogfooded: its model is in the `mitra:` comments above
//! each function and test, and `.mitra/evidence.mitra` holds the results of
//! its own test runs. In the model a mass is three numbers `b d u`, since
//! Mitra has no records. Each function has a `uniform` claim: the classes
//! of inputs on which it is assumed to behave the same, so that one test
//! stands for a class.

/// Belief split on {correct, wrong}: `b + d + u = 1`, each in [0, 1].
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
    ///
    /// mitra: -- Correct: `Some((c, 0, 1 - c))` if ok, `Some((0, c, 1 - c))`
    /// mitra: -- if not, for c in [0, 1]; `None` otherwise.
    /// mitra: opaque judged (ok : Bool) (c : Float64) : Value
    /// mitra: claim judged_correct : forall ok, forall c, correct (judged ok c)
    /// mitra: -- One class per path through the code.
    /// mitra: def judged_class (ok : Bool) (c : Float64) : {"rejected", "correct", "wrong"} :=
    /// mitra:   if c >= 0.0 && c <= 1.0 then (if ok then "correct" else "wrong") else "rejected"
    /// mitra: claim judged_uniform : uniform judged by judged_class
    pub fn judged(correct: bool, c: f64) -> Option<Mass> {
        if !(0.0..=1.0).contains(&c) {
            return None;
        }
        let (b, d) = if correct { (c, 0.0) } else { (0.0, c) };
        Some(Mass { b, d, u: 1.0 - c })
    }

    /// Each part in [0, 1] and the parts sum to 1, up to [`EPSILON`].
    ///
    /// mitra: opaque is_valid (b d u : Float64) : Bool
    /// mitra: def valid_spec (b d u : Float64) : Bool :=
    /// mitra:   let e := 0.000000001; let s := b + d + u - 1.0;
    /// mitra:   b >= -e && b <= 1.0 + e && d >= -e && d <= 1.0 + e
    /// mitra:   && u >= -e && u <= 1.0 + e && s <= e && -s <= e
    /// mitra: claim is_valid_spec : forall b, forall d, forall u, is_valid b d u = valid_spec b d u
    /// mitra: def valid_class (b d u : Float64) : {"valid", "part out of range", "sum off"} :=
    /// mitra:   let e := 0.000000001;
    /// mitra:   if !(b >= -e && b <= 1.0 + e && d >= -e && d <= 1.0 + e && u >= -e && u <= 1.0 + e)
    /// mitra:   then "part out of range" else if valid_spec b d u then "valid" else "sum off"
    /// mitra: claim is_valid_uniform : uniform is_valid by valid_class
    pub fn is_valid(&self) -> bool {
        let parts = [self.b, self.d, self.u];
        parts.iter().all(|x| (-EPSILON..=1.0 + EPSILON).contains(x))
            && (parts.iter().sum::<f64>() - 1.0).abs() <= EPSILON
    }

    /// The mass two sources give to opposite sides: `b₁d₂ + d₁b₂`.
    ///
    /// mitra: opaque conflict (b1 d1 u1 b2 d2 u2 : Float64) : Float64
    /// mitra: def conflict_spec (b1 d1 u1 b2 d2 u2 : Float64) : Float64 := b1 * d2 + d1 * b2
    /// mitra: claim conflict_spec_ok : forall b1, forall d1, forall u1, forall b2, forall d2, forall u2,
    /// mitra:   conflict b1 d1 u1 b2 d2 u2 = conflict_spec b1 d1 u1 b2 d2 u2
    /// mitra: -- One formula, no branch: a single class.
    /// mitra: def conflict_class (b1 d1 u1 b2 d2 u2 : Float64) : {"any"} := "any"
    /// mitra: claim conflict_uniform : uniform conflict by conflict_class
    pub fn conflict(&self, o: &Mass) -> f64 {
        self.b * o.d + self.d * o.b
    }

    /// The parts both sources agree on, before any normalisation.
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
    ///
    /// mitra: -- Correct: `None` if either mass is invalid or the conflict k is
    /// mitra: -- at least 1 - EPSILON; else the agreement divided by 1 - k.
    /// mitra: opaque dempster (b1 d1 u1 b2 d2 u2 : Float64) : Value
    /// mitra: claim dempster_correct : forall b1, forall d1, forall u1, forall b2, forall d2, forall u2,
    /// mitra:   correct (dempster b1 d1 u1 b2 d2 u2)
    /// mitra: def dempster_class (b1 d1 u1 b2 d2 u2 : Float64)
    /// mitra:   : {"first invalid", "second invalid", "total conflict", "partial conflict"} :=
    /// mitra:   if !valid_spec b1 d1 u1 then "first invalid"
    /// mitra:   else if !valid_spec b2 d2 u2 then "second invalid"
    /// mitra:   else if conflict_spec b1 d1 u1 b2 d2 u2 >= 1.0 - 0.000000001 then "total conflict"
    /// mitra:   else "partial conflict"
    /// mitra: claim dempster_uniform : uniform dempster by dempster_class
    /// mitra: claim dempster_uses : only uses dempster [is_valid, conflict]
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
    ///
    /// mitra: -- Correct: `None` if either mass is invalid; else the agreement,
    /// mitra: -- with the conflict k added to unknown, and k.
    /// mitra: opaque yager (b1 d1 u1 b2 d2 u2 : Float64) : Value
    /// mitra: claim yager_correct : forall b1, forall d1, forall u1, forall b2, forall d2, forall u2,
    /// mitra:   correct (yager b1 d1 u1 b2 d2 u2)
    /// mitra: def yager_class (b1 d1 u1 b2 d2 u2 : Float64) : {"first invalid", "second invalid", "valid"} :=
    /// mitra:   if !valid_spec b1 d1 u1 then "first invalid"
    /// mitra:   else if !valid_spec b2 d2 u2 then "second invalid" else "valid"
    /// mitra: claim yager_uniform : uniform yager by yager_class
    /// mitra: claim yager_uses : only uses yager [is_valid, conflict]
    pub fn yager(&self, o: &Mass) -> Option<(Mass, f64)> {
        if !self.is_valid() || !o.is_valid() {
            return None;
        }
        let k = self.conflict(o);
        let a = self.agreement(o);
        Some((Mass { u: a.u + k, ..a }, k))
    }
}
