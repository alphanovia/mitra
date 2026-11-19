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
//!
//! When the pieces trusted at 1 can't all be right, no `T` is possible and
//! Dempster's rule is undefined. The ranking is then the limit of the
//! rule as every bias of 1 goes down to 1 - e, e going to 0: only the
//! ways with the fewest trusted pieces wrong count, and trusted pieces
//! that play the same part share the doubt equally. `Ranking::doubted`
//! says so.

/// The most pieces with a bias strictly between 0 and 1 that are ranked:
/// each doubles the work.
pub const MAX_OPEN: usize = 20;

/// Belief and plausibility that each piece is wrong.
///
#[derive(Debug, Clone, PartialEq)]
// mitra: implements culprits::Ranking
pub struct Ranking {
    pub belief: Vec<f64>,
    pub plausibility: Vec<f64>,
    /// The mass that says every piece of some clause is right.
    pub conflict: f64,
    /// How many pieces trusted at 1 had to be taken as wrong: 0 unless
    /// the pieces trusted at 1 can't all be right.
    pub doubted: usize,
}

/// Why there is no ranking.
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoRanking {
    /// More than [`MAX_OPEN`] pieces with a bias strictly between 0 and 1,
    /// counting those trusted at 1 when they can't all be right.
    TooMany,
    /// A bias outside [0, 1], or a clause naming a piece that is not there.
    Invalid,
}

/// Rank `biases.len()` pieces. `biases[i]` is the trust in piece `i`, from
/// 0 to 1, or `None` for a piece nobody vouched for. Each clause lists
/// pieces that can't all be right.
// mitra: implements culprits::rank
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
    let sure: Vec<usize> = (0..n).filter(|i| biases[*i] == Some(1.0)).collect();
    if let Some(r) = weigh(biases, clauses, &open, &sure, false) {
        return Ok(r);
    }
    // The pieces trusted at 1 can't all be right: doubt them too.
    if open.len() + sure.len() > MAX_OPEN {
        return Err(NoRanking::TooMany);
    }
    Ok(weigh(biases, clauses, &open, &sure, true)
        .expect("with every piece wrong, no clause is whole"))
}

/// The ranking over the ways of taking the `open` pieces as right or
/// wrong, and the `sure` ones too when `doubt` is set. Only the ways with
/// the fewest `sure` pieces wrong count. `None` when there is no way.
fn weigh(
    biases: &[Option<f64>],
    clauses: &[Vec<usize>],
    open: &[usize],
    sure: &[usize],
    doubt: bool,
) -> Option<Ranking> {
    let n = biases.len();
    let free: Vec<usize> = if doubt {
        open.iter().chain(sure).copied().collect()
    } else {
        open.to_vec()
    };
    let mut right: Vec<bool> = biases.iter().map(|b| *b == Some(1.0)).collect();
    // One tally per number of sure pieces taken as wrong.
    let levels = if doubt { sure.len() + 1 } else { 1 };
    let mut tally = vec![(0.0, vec![0.0; n], vec![0.0; n]); levels];
    for choice in 0u32..(1 << free.len()) {
        let (mut p, mut wrong) = (1.0, 0);
        for (k, i) in free.iter().enumerate() {
            let taken = choice >> k & 1 == 1;
            right[*i] = taken;
            match biases[*i] {
                Some(b) if b < 1.0 => p *= if taken { b } else { 1.0 - b },
                _ => wrong += usize::from(!taken),
            }
        }
        if p <= 0.0 || clauses.iter().any(|c| c.iter().all(|i| right[*i])) {
            continue;
        }
        let (possible, belief, plausibility) = &mut tally[wrong];
        *possible += p;
        for x in (0..n).filter(|x| !right[*x]) {
            plausibility[x] += p;
            let forced = |c: &&Vec<usize>| c.contains(&x) && c.iter().all(|i| *i == x || right[*i]);
            if clauses.iter().any(|c| forced(&c)) {
                belief[x] += p;
            }
        }
    }
    let doubted = tally.iter().position(|t| t.0 > 0.0)?;
    let (possible, mut belief, mut plausibility) = tally.swap_remove(doubted);
    for v in belief.iter_mut().chain(plausibility.iter_mut()) {
        *v /= possible;
    }
    Some(Ranking {
        belief,
        plausibility,
        conflict: if doubted > 0 { 1.0 } else { 1.0 - possible },
        doubted,
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
    // mitra: evidence for culprits::rank given biases = [None, Some(0.8)], clauses = [[0, 1]]
    #[test]
    fn code_against_one_failure() {
        let r = rank(&[None, Some(0.8)], &[vec![0, 1]]).unwrap();
        assert!(close(r.belief[0], 0.8) && close(r.plausibility[0], 1.0));
        assert!(close(r.belief[1], 0.0) && close(r.plausibility[1], 0.2));
        assert!(close(r.conflict, 0.0));
    }

    /// A review at 0.9, a pass at 0.99 and a failure at 0.8 that can't all
    /// be true: hand-computed with Dempster's rule.
    // mitra: evidence for culprits::rank given biases = [Some(0.9), Some(0.99), Some(0.8)], clauses = [[0, 1, 2]]
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
    // mitra: evidence for culprits::rank given biases = [Some(0.3), Some(1.0), None, Some(0.0)], clauses = []
    #[test]
    fn no_clause_no_belief() {
        let r = rank(&[Some(0.3), Some(1.0), None, Some(0.0)], &[]).unwrap();
        assert!(r.belief.iter().all(|b| *b == 0.0));
        assert!(close(r.conflict, 0.0));
    }

    /// P2: a piece at bias 0 in a clause has plausibility 1.
    // mitra: evidence for culprits::rank given biases = [Some(0.0), Some(0.6), Some(0.7)], clauses = [[0, 1, 2]]
    #[test]
    fn distrust_shows() {
        let r = rank(&[Some(0.0), Some(0.6), Some(0.7)], &[vec![0, 1, 2]]).unwrap();
        assert!(close(r.plausibility[0], 1.0));
        assert!(r.plausibility[0] > r.plausibility[1]);
    }

    /// P3: full trust in the evidence puts the blame on the code.
    // mitra: evidence for culprits::rank given biases = [None, Some(1.0)], clauses = [[0, 1]] gives Ok(Ranking([1.0, 0.0], [1.0, 0.0], 0.0, 0))
    #[test]
    fn full_trust() {
        let r = rank(&[None, Some(1.0)], &[vec![0, 1]]).unwrap();
        // Nothing is open, so the numbers are exact.
        let expected = Ranking {
            belief: vec![1.0, 0.0],
            plausibility: vec![1.0, 0.0],
            conflict: 0.0,
            doubted: 0,
        };
        assert_eq!(r, expected);
    }

    /// Pieces trusted at 1 that can't all be right share the doubt
    /// equally: one of them is wrong, and nothing says which.
    #[test]
    fn full_trust_in_every_piece_is_a_tie() {
        let r = rank(&[Some(1.0), Some(1.0), Some(1.0)], &[vec![0, 1, 2]]).unwrap();
        assert_eq!(r.doubted, 1);
        assert!(close(r.conflict, 1.0));
        for i in 0..3 {
            assert!(close(r.belief[i], 1.0 / 3.0) && close(r.plausibility[i], 1.0 / 3.0));
        }
        // A piece that is not fully trusted takes all the doubt first.
        let r = rank(&[Some(1.0), Some(1.0), Some(0.9)], &[vec![0, 1, 2]]).unwrap();
        assert_eq!(r.doubted, 0);
        assert!(close(r.belief[2], 1.0) && close(r.plausibility[0], 0.0));
        // The code beside them stays out of it: no clause needs it wrong.
        let r = rank(&[None, Some(1.0), Some(1.0)], &[vec![1, 2], vec![0, 2]]).unwrap();
        assert!(close(r.plausibility[1], 0.5) && close(r.plausibility[2], 0.5));
        assert!(close(r.belief[0], 0.5) && close(r.plausibility[0], 1.0));
    }

    /// Two trusted passes, one distrusted pass, a trusted failure and a
    /// trusted review: by hand, over the ways with one trusted piece wrong.
    // mitra: evidence for culprits::rank given biases = [Some(1.0), Some(0.1), Some(1.0), Some(1.0)], clauses = [[0, 1, 3], [0, 2, 3]]
    #[test]
    fn a_tie_beside_a_distrusted_piece() {
        // 0 review, 1 pass at 0.1, 2 pass, 3 failure.
        let biases = [Some(1.0), Some(0.1), Some(1.0), Some(1.0)];
        let r = rank(&biases, &[vec![0, 1, 3], vec![0, 2, 3]]).unwrap();
        // Review wrong: 1. Failure wrong: 1. Pass 2 wrong: pass 1 too, 0.9.
        let z = 2.9;
        assert!(close(r.plausibility[0], 1.0 / z) && close(r.plausibility[3], 1.0 / z));
        assert!(close(r.plausibility[2], 0.9 / z));
        assert!(close(r.plausibility[1], 2.7 / z) && close(r.belief[1], 0.9 / z));
    }

    /// P4: a piece in no clause is never believed wrong.
    // mitra: evidence for culprits::rank given biases = [None, Some(0.5), Some(0.5)], clauses = [[0, 1]]
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
    // mitra: evidence for culprits::rank given biases = [None, Some(0.8), Some(0.8)], clauses = [[0, 1], [0, 2]]
    #[test]
    fn two_failures_agree() {
        let r = rank(&[None, Some(0.8), Some(0.8)], &[vec![0, 1], vec![0, 2]]).unwrap();
        assert!(close(r.belief[0], 1.0 - 0.2 * 0.2));
    }

    // mitra: evidence for culprits::rank given biases = [Some(1.5)], clauses = [] gives Err(Invalid)
    #[test]
    fn rejects_bias_above_one() {
        assert_eq!(rank(&[Some(1.5)], &[]), Err(NoRanking::Invalid));
    }

    // mitra: evidence for culprits::rank given biases = [Some(0.5)], clauses = [[3]] gives Err(Invalid)
    #[test]
    fn rejects_unknown_piece() {
        assert_eq!(rank(&[Some(0.5)], &[vec![3]]), Err(NoRanking::Invalid));
    }

    // Not in the model: 21 pieces written out would say less than this.
    #[test]
    fn rejects_too_many() {
        let many = vec![Some(0.5); MAX_OPEN + 1];
        assert_eq!(rank(&many, &[]), Err(NoRanking::TooMany));
    }
}
