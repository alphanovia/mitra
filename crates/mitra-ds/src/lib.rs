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

pub mod culprits;

/// Belief split on {correct, wrong}: `b + d + u = 1`, each in [0, 1].
///
/// mitra: record Mass(b: Float64, d: Float64, u: Float64)
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
    /// mitra: opaque judged(ok: Bool, c: Float64) -> Option(Mass)
    /// mitra: def judged_spec(ok: Bool, c: Float64) -> Option(Mass) :=
    /// mitra:   if c >= 0.0 && c <= 1.0 then Some(if ok then Mass(c, 0.0, 1.0 - c) else Mass(0.0, c, 1.0 - c)) else None
    /// mitra: claim judged_is : forall ok, forall c, judged(ok, c) = judged_spec(ok, c)
    /// mitra: -- One class per path through the code.
    /// mitra: def judged_class(ok: Bool, c: Float64) -> {"rejected", "correct", "wrong"} :=
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
    /// mitra: opaque is_valid(m: Mass) -> Bool
    /// mitra: def in_range(x: Float64) -> Bool := x >= -0.000000001 && x <= 1.0 + 0.000000001
    /// mitra: def valid_spec(m: Mass) -> Bool :=
    /// mitra:   let s := m.b + m.d + m.u - 1.0;
    /// mitra:   in_range(m.b) && in_range(m.d) && in_range(m.u) && s <= 0.000000001 && -s <= 0.000000001
    /// mitra: claim is_valid_is : forall m, is_valid(m) = valid_spec(m)
    /// mitra: def valid_class(m: Mass) -> {"valid", "part out of range", "sum off"} :=
    /// mitra:   if !(in_range(m.b) && in_range(m.d) && in_range(m.u)) then "part out of range"
    /// mitra:   else if valid_spec(m) then "valid" else "sum off"
    /// mitra: claim is_valid_uniform : uniform is_valid by valid_class
    pub fn is_valid(&self) -> bool {
        let parts = [self.b, self.d, self.u];
        parts.iter().all(|x| (-EPSILON..=1.0 + EPSILON).contains(x))
            && (parts.iter().sum::<f64>() - 1.0).abs() <= EPSILON
    }

    /// The mass two sources give to opposite sides: `b₁d₂ + d₁b₂`.
    ///
    /// mitra: opaque conflict(m: Mass, o: Mass) -> Float64
    /// mitra: def conflict_spec(m: Mass, o: Mass) -> Float64 := m.b * o.d + m.d * o.b
    /// mitra: claim conflict_is : forall m, forall o, conflict(m, o) = conflict_spec(m, o)
    /// mitra: -- One formula, no branch: a single class.
    /// mitra: claim conflict_uniform : uniform conflict
    /// mitra: claim conflict_symmetric : forall m, forall o, conflict(m, o) == conflict(o, m)
    pub fn conflict(&self, o: &Mass) -> f64 {
        self.b * o.d + self.d * o.b
    }

    /// The parts both sources agree on, before any normalisation.
    ///
    /// mitra: -- Private: modelled by a spec only, the rules that use it are judged.
    /// mitra: def agreement_spec(m: Mass, o: Mass) -> Mass :=
    /// mitra:   Mass(m.b * o.b + m.b * o.u + m.u * o.b, m.d * o.d + m.d * o.u + m.u * o.d, m.u * o.u)
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
    /// mitra: def dempster_spec(m: Mass, o: Mass) -> Option(Mass) :=
    /// mitra:   let k := conflict_spec(m, o); let a := agreement_spec(m, o);
    /// mitra:   if !valid_spec(m) || !valid_spec(o) || k >= 1.0 - 0.000000001 then None
    /// mitra:   else Some(Mass(a.b / (1.0 - k), a.d / (1.0 - k), a.u / (1.0 - k)))
    /// mitra: opaque dempster(m: Mass, o: Mass) -> Option(Mass)
    /// mitra: claim dempster_is : forall m, forall o, dempster(m, o) = dempster_spec(m, o)
    /// mitra: def dempster_class(m: Mass, o: Mass) -> {"first invalid", "second invalid", "total conflict", "partial conflict"} :=
    /// mitra:   if !valid_spec(m) then "first invalid" else if !valid_spec(o) then "second invalid"
    /// mitra:   else if conflict_spec(m, o) >= 1.0 - 0.000000001 then "total conflict" else "partial conflict"
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
    /// mitra: def yager_spec(m: Mass, o: Mass) -> Option((Mass, Float64)) :=
    /// mitra:   let k := conflict_spec(m, o); let a := agreement_spec(m, o);
    /// mitra:   if !valid_spec(m) || !valid_spec(o) then None else Some((Mass(a.b, a.d, a.u + k), k))
    /// mitra: opaque yager(m: Mass, o: Mass) -> Option((Mass, Float64))
    /// mitra: claim yager_is : forall m, forall o, yager(m, o) = yager_spec(m, o)
    /// mitra: def yager_class(m: Mass, o: Mass) -> {"first invalid", "second invalid", "valid"} :=
    /// mitra:   if !valid_spec(m) then "first invalid" else if !valid_spec(o) then "second invalid" else "valid"
    /// mitra: claim yager_uniform : uniform yager by yager_class
    /// mitra: claim yager_uses : only uses yager [is_valid, conflict]
    /// mitra: -- The conflict `yager` reports is the one `conflict` computes.
    /// mitra: claim yager_reports_conflict : forall m, forall o,
    /// mitra:   match yager(m, o) { None => true, Some(r) => r.1 == conflict(m, o) }
    /// mitra: -- Dempster's rule is Yager's, with the conflict renormalised away.
    /// mitra: claim dempster_renormalises_yager : forall m, forall o,
    /// mitra:   match dempster(m, o) { None => true, Some(x) => match yager(m, o) {
    /// mitra:     None => false, Some(y) => x.b == y.0.b / (1.0 - y.1) && x.d == y.0.d / (1.0 - y.1) } }
    /// mitra: -- Sorted by its own classes: the classes of `dempster` and `yager`
    /// mitra: -- can't all meet (an input can't be invalid for one and valid for the other).
    /// mitra: claim renormalises_uniform : uniform dempster_renormalises_yager by dempster_class
    pub fn yager(&self, o: &Mass) -> Option<(Mass, f64)> {
        if !self.is_valid() || !o.is_valid() {
            return None;
        }
        let k = self.conflict(o);
        let a = self.agreement(o);
        Some((Mass { u: a.u + k, ..a }, k))
    }
}
