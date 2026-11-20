//! How plausibly each piece of an error is the wrong one.

use super::*;

/// Fill in, for each problem, how plausibly each of its pieces is the
/// wrong one. Problems that share a piece are weighed together.
pub(super) fn rank(model: &Model, problems: &mut [Problem]) {
    // The same observation recorded twice is one piece: same judgment,
    // inputs, source and revision. Its bias is the highest given.
    let same = |a: &Evidence, b: &Evidence| {
        (&a.about, &a.pattern, &a.judgment, &a.source, &a.rev)
            == (&b.about, &b.pattern, &b.judgment, &b.source, &b.rev)
    };
    let set_aside = model.misfits.iter().map(|m| &m.0);
    let entry = |line: usize| {
        model
            .evidence
            .iter()
            .chain(set_aside.clone())
            .find(|e| e.line == line)
    };
    let canon = |p: &Piece| match p {
        Piece::Evidence(line) => {
            // What the model itself declares is a piece of its own kind.
            let files = model.written.as_ref().map(|w| w.1.len());
            let stated = entry(*line).filter(|_| files.is_some_and(|n| *line > n));
            if let Some(c) = stated.and_then(|e| model.claim(&e.about)) {
                return match c.target {
                    Target::Uses { .. } => Piece::Uses(c.func.clone()),
                    _ => Piece::Assumption(c.name.clone()),
                };
            }
            let first = entry(*line).and_then(|e| model.evidence.iter().find(|o| same(o, e)));
            Piece::Evidence(first.map_or(*line, |o| o.line))
        }
        other => other.clone(),
    };
    let declared = |name: &str| {
        let files = model.written.as_ref().map_or(0, |w| w.1.len());
        let mut all = model.evidence.iter().chain(set_aside.clone());
        all.find(|e| e.about == name && e.line > files)
    };
    let bias = |p: &Piece| match p {
        Piece::Evidence(line) => entry(*line).map(|e| {
            let twins = model.evidence.iter().filter(|o| same(o, e));
            twins.map(|o| o.confidence).fold(e.confidence, f64::max)
        }),
        Piece::Claim(c) => Some(model.claim(c).and_then(|c| c.trust).unwrap_or(1.0)),
        // What the model states is trusted like evidence: 1 unless said.
        Piece::Signature(_) => Some(1.0),
        Piece::Helper(f) => {
            let said = model.helper_trust.iter().find(|h| &h.0 == f);
            Some(said.map_or(1.0, |h| h.1))
        }
        // The tie is trusted as the word of who read the code.
        Piece::Tie(f) => {
            let about = format!("{f}::tied");
            let said = |e: &&Evidence| e.about == about && e.judgment == Judgment::Holds(true);
            let all = model.evidence.iter().filter(said);
            all.map(|e| e.confidence).reduce(f64::max)
        }
        // The prose is what is judged: no trust is given to it. The word
        // that the claims say it is trusted as its vouch is, if any.
        Piece::Requirement(c) => model.claim(c).and_then(|c| c.trust),
        Piece::Capture(c) => {
            let about = format!("{c}::captured");
            let vouch = |e: &&Evidence| e.about == about && e.judgment == Judgment::Holds(true);
            let all = model.evidence.iter().filter(vouch);
            all.map(|e| e.confidence).reduce(f64::max)
        }
        Piece::Assumption(_) | Piece::Uses(_) => {
            let name = p.declared_claim().expect("declared");
            Some(declared(&name).map_or(1.0, |e| e.confidence))
        }
        Piece::Code(_) => None,
    };
    for p in problems.iter_mut() {
        for clause in &mut p.clauses {
            *clause = clause.iter().map(canon).collect();
            clause.dedup();
        }
    }
    type Weighed = Result<(Vec<Candidate>, f64), Unranked>;
    let weigh = |members: &[usize], problems: &[Problem]| -> Weighed {
        let mut pieces: Vec<Piece> = Vec::new();
        let mut clauses: Vec<Vec<usize>> = Vec::new();
        for m in members {
            for clause in &problems[*m].clauses {
                let mut ids = Vec::new();
                for piece in clause {
                    let known = pieces.iter().position(|q| q == piece);
                    ids.push(known.unwrap_or_else(|| {
                        pieces.push(piece.clone());
                        pieces.len() - 1
                    }));
                }
                clauses.push(ids);
            }
        }
        let biases: Vec<Option<f64>> = pieces.iter().map(bias).collect();
        match crate::verdicts::culprits::rank(&biases, &clauses) {
            Ok(r) => {
                let candidate = |(i, piece): (usize, Piece)| Candidate {
                    piece,
                    belief: r.belief[i],
                    plausibility: r.plausibility[i],
                };
                let all = pieces.into_iter().enumerate().map(candidate).collect();
                Ok((all, r.conflict))
            }
            Err(_) => Err(Unranked::TooMany),
        }
    };
    let n = problems.len();
    // Groups of problems that share a piece.
    let mut group: Vec<usize> = (0..n).collect();
    let shares = |a: &Problem, b: &Problem| {
        let mut of_b = b.clauses.iter().flatten();
        of_b.any(|x| a.clauses.iter().flatten().any(|y| x == y))
    };
    for i in 0..n {
        for j in 0..i {
            if group[i] != group[j] && shares(&problems[i], &problems[j]) {
                let (from, to) = (group[i], group[j]);
                group
                    .iter_mut()
                    .filter(|g| **g == from)
                    .for_each(|g| *g = to);
            }
        }
    }
    let mut done = vec![false; n];
    for i in 0..n {
        if done[i] {
            continue;
        }
        let members: Vec<usize> = (0..n).filter(|j| group[*j] == group[i]).collect();
        let together = weigh(&members, problems);
        for m in &members {
            done[*m] = true;
            let result = together.clone();
            let p = &mut problems[*m];
            match result {
                Ok((all, conflict)) => {
                    let mut mine: Vec<Candidate> = Vec::new();
                    for piece in p.clauses.iter().flatten() {
                        if !mine.iter().any(|c| &c.piece == piece) {
                            mine.extend(all.iter().find(|c| &c.piece == piece).cloned());
                        }
                    }
                    let key = |c: &Candidate| (c.plausibility, c.belief);
                    mine.sort_by(|a, b| key(b).partial_cmp(&key(a)).expect("no NaN"));
                    p.candidates = Ok(mine);
                    p.conflict = conflict;
                }
                Err(e) => p.candidates = Err(e),
            }
        }
    }
}

/// A set as text, or its size when the text would be too long to read.
pub(crate) fn brief(set: &Tuples, domain: &Tuples) -> String {
    if set.is_empty() {
        return "nothing".to_string();
    }
    let text = set.show(domain);
    if text.len() <= WIDE {
        return text;
    }
    // The bound counts an input twice when two boxes overlap: list a
    // small set to count it exactly.
    match set.size_bound() {
        Some(n) if n <= POINTS => format!("{} inputs", set.count().unwrap_or(n)),
        Some(n) => format!("{n} inputs"),
        None => "a set too long to print".to_string(),
    }
}

/// A set printed longer than this is summed up.
pub(crate) const WIDE: usize = 160;

/// The most evidence lines an error quotes.
pub(crate) const QUOTED: usize = 5;
