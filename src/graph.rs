//! Graphs: which nodes reach each other. Judgement uses it for rules
//! defined through themselves, the model for what rests on itself.

/// The component of each node, where `out` holds the nodes each goes
/// to: two nodes are in one component when each reaches the other.
/// Kosaraju's walk, on stacks of its own, so that a long chain does not
/// run out of the call stack.
pub(crate) fn components(out: &[Vec<usize>]) -> Vec<usize> {
    let n = out.len();
    let (mut finished, mut seen) = (Vec::with_capacity(n), vec![false; n]);
    for start in 0..n {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut walk = vec![(start, 0)];
        while let Some(&(node, next)) = walk.last() {
            let top = walk.len() - 1;
            match out[node].get(next) {
                Some(&to) => {
                    walk[top].1 += 1;
                    if !seen[to] {
                        seen[to] = true;
                        walk.push((to, 0));
                    }
                }
                None => {
                    finished.push(node);
                    walk.pop();
                }
            }
        }
    }
    let mut back = vec![Vec::new(); n];
    for (from, to) in out.iter().enumerate() {
        for &to in to {
            back[to].push(from);
        }
    }
    let mut component = vec![usize::MAX; n];
    for (c, &start) in finished.iter().rev().enumerate() {
        if component[start] != usize::MAX {
            continue;
        }
        component[start] = c;
        let mut walk = vec![start];
        while let Some(node) = walk.pop() {
            for &from in &back[node] {
                if component[from] == usize::MAX {
                    component[from] = c;
                    walk.push(from);
                }
            }
        }
    }
    component
}
