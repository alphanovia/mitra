//! Which pieces are most plausibly wrong, when some can't all be true.
//!
//! The frame is every way of calling each piece right or wrong. A piece
//! with a bias `b` is a simple support function: mass `b` on "this piece
//! is right", the rest on the whole frame. A piece without a bias says
//! nothing. A clause is a set of pieces that can't all be right. The
//! masses are combined by Dempster's rule and conditioned on the clauses;
//! the mass that contradicts a clause is the conflict.
//!
//! For each piece this gives the belief and the plausibility that it is
//! wrong. Because every mass supports "right", the combination has a
//! closed form over the sets `T` of pieces taken as right:
//! - `T` is possible when it contains no whole clause;
//! - a piece outside `T` may be wrong (plausibility);
//! - it must be wrong when a clause has all its other pieces in `T`
//!   (belief).

/// The most pieces with a bias strictly between 0 and 1 that are ranked:
/// each doubles the work.
pub const MAX_OPEN: usize = 20;

/// Belief and plausibility that each piece is wrong.
#[derive(Debug, Clone, PartialEq)]
pub struct Ranking {
    pub belief: Vec<f64>,
    pub plausibility: Vec<f64>,
    /// The mass that says every piece of some clause is right.
    pub conflict: f64,
}

/// Why there is no ranking.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoRanking {
    /// The biases leave no way for the clauses to hold: every piece of
    /// some clause is fully trusted.
    TotalConflict,
    /// More than [`MAX_OPEN`] pieces with a bias strictly between 0 and 1.
    TooMany,
    /// A bias outside [0, 1], or a clause naming a piece that is not there.
    Invalid,
}

/// Rank `biases.len()` pieces. `biases[i]` is the trust in piece `i`, from
/// 0 to 1, or `None` for a piece nobody vouched for. Each clause lists
/// pieces that can't all be right.
pub fn rank(biases: &[Option<f64>], clauses: &[Vec<usize>]) -> Result<Ranking, NoRanking> {
    let n = biases.len();
    let valid = |b: &Option<f64>| b.is_none_or(|b| (0.0..=1.0).contains(&b));
    if !biases.iter().all(valid) || clauses.iter().flatten().any(|i| *i >= n) {
        return Err(NoRanking::Invalid);
    }
    // Pieces at 1 are always taken as right, pieces at 0 or without a bias
    // never; only the others are open.
    let open: Vec<usize> = (0..n)
        .filter(|i| biases[*i].is_some_and(|b| b > 0.0 && b < 1.0))
        .collect();
    if open.len() > MAX_OPEN {
        return Err(NoRanking::TooMany);
    }
    let mut right: Vec<bool> = biases.iter().map(|b| *b == Some(1.0)).collect();
    let (mut belief, mut plausibility) = (vec![0.0; n], vec![0.0; n]);
    let mut possible = 0.0;
    for choice in 0u32..(1 << open.len()) {
        let mut p = 1.0;
        for (k, i) in open.iter().enumerate() {
            let taken = choice >> k & 1 == 1;
            right[*i] = taken;
            let b = biases[*i].expect("open pieces have a bias");
            p *= if taken { b } else { 1.0 - b };
        }
        if clauses.iter().any(|c| c.iter().all(|i| right[*i])) {
            continue;
        }
        possible += p;
        for x in (0..n).filter(|x| !right[*x]) {
            plausibility[x] += p;
            let forced = |c: &&Vec<usize>| c.contains(&x) && c.iter().all(|i| *i == x || right[*i]);
            if clauses.iter().any(|c| forced(&c)) {
                belief[x] += p;
            }
        }
    }
    if possible <= 0.0 {
        return Err(NoRanking::TotalConflict);
    }
    for v in belief.iter_mut().chain(plausibility.iter_mut()) {
        *v /= possible;
    }
    Ok(Ranking {
        belief,
        plausibility,
        conflict: 1.0 - possible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    /// A failure at bias 0.8 against the code: the code is wrong with
    /// belief 0.8, the evidence with plausibility 0.2.
    #[test]
    fn code_against_one_failure() {
        let r = rank(&[None, Some(0.8)], &[vec![0, 1]]).unwrap();
        assert!(close(r.belief[0], 0.8) && close(r.plausibility[0], 1.0));
        assert!(close(r.belief[1], 0.0) && close(r.plausibility[1], 0.2));
        assert!(close(r.conflict, 0.0));
    }

    /// A review at 0.9, a pass at 0.99 and a failure at 0.8 that can't all
    /// be true: hand-computed with Dempster's rule.
    #[test]
    fn three_pieces_in_conflict() {
        let r = rank(&[Some(0.9), Some(0.99), Some(0.8)], &[vec![0, 1, 2]]).unwrap();
        let z = 1.0 - 0.9 * 0.99 * 0.8;
        assert!(close(r.conflict, 0.7128));
        assert!(close(r.plausibility[2], 0.2 / z));
        assert!(close(r.belief[2], 0.9 * 0.99 * 0.2 / z));
        assert!(close(r.plausibility[0], 0.1 / z));
        assert!(close(r.plausibility[1], 0.01 / z));
    }

    /// P1: without a clause nothing is believed wrong, whatever the biases.
    #[test]
    fn no_clause_no_belief() {
        let r = rank(&[Some(0.3), Some(1.0), None, Some(0.0)], &[]).unwrap();
        assert!(r.belief.iter().all(|b| *b == 0.0));
        assert!(close(r.conflict, 0.0));
    }

    /// P2: a piece at bias 0 in a clause has plausibility 1.
    #[test]
    fn distrust_shows() {
        let r = rank(&[Some(0.0), Some(0.6), Some(0.7)], &[vec![0, 1, 2]]).unwrap();
        assert!(close(r.plausibility[0], 1.0));
        assert!(r.plausibility[0] > r.plausibility[1]);
    }

    /// P3: full trust in the evidence puts the blame on the code; full
    /// trust in every piece is a total conflict.
    #[test]
    fn full_trust() {
        let r = rank(&[None, Some(1.0)], &[vec![0, 1]]).unwrap();
        assert!(close(r.belief[0], 1.0) && close(r.plausibility[1], 0.0));
        let r = rank(&[Some(1.0), Some(1.0)], &[vec![0, 1]]);
        assert_eq!(r, Err(NoRanking::TotalConflict));
    }

    /// P4: a piece in no clause is never believed wrong.
    #[test]
    fn outside_every_clause() {
        let r = rank(&[None, Some(0.5), Some(0.5)], &[vec![0, 1]]).unwrap();
        assert!(close(r.belief[2], 0.0));
    }

    /// P5: lowering a bias never lowers the plausibility of being wrong.
    #[test]
    fn monotone_in_the_bias() {
        let at = |b: f64| {
            let r = rank(&[Some(b), Some(0.7), None], &[vec![0, 1], vec![1, 2]]).unwrap();
            r.plausibility[0]
        };
        let mut last = at(1.0);
        for k in (0..10).rev() {
            let now = at(k as f64 / 10.0);
            assert!(now >= last - 1e-12, "{k}");
            last = now;
        }
    }

    /// P6: the order of pieces and clauses does not matter.
    #[test]
    fn order_free() {
        let a = rank(&[Some(0.9), Some(0.5), None], &[vec![0, 1], vec![1, 2]]).unwrap();
        let b = rank(&[None, Some(0.5), Some(0.9)], &[vec![1, 0], vec![2, 1]]).unwrap();
        for (i, j) in [(0, 2), (1, 1), (2, 0)] {
            assert!(close(a.belief[i], b.belief[j]));
            assert!(close(a.plausibility[i], b.plausibility[j]));
        }
    }

    /// Two failures against the code: both would have to be wrong for the
    /// code to be right.
    #[test]
    fn two_failures_agree() {
        let r = rank(&[None, Some(0.8), Some(0.8)], &[vec![0, 1], vec![0, 2]]).unwrap();
        assert!(close(r.belief[0], 1.0 - 0.2 * 0.2));
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(rank(&[Some(1.5)], &[]), Err(NoRanking::Invalid));
        assert_eq!(rank(&[Some(0.5)], &[vec![3]]), Err(NoRanking::Invalid));
        let many = vec![Some(0.5); MAX_OPEN + 1];
        assert_eq!(rank(&many, &[]), Err(NoRanking::TooMany));
    }
}
