//! A solver for games that are easy to *enumerate forward* but awkward to give a
//! dense perfect-hash index (e.g. capture games like morris, where men leave the
//! board). It discovers the reachable state set by forward search from the start,
//! then runs the same loopy retrograde analysis over that explicit graph.
//!
//! Trades the dense-array memory of [`crate::solve`] for a hash map keyed on the
//! state. Fine up to ~10^7–10^8 reachable states; beyond that a tight index +
//! external memory is needed (see the roadmap).

use crate::Outcome;
use std::collections::HashMap;
use std::hash::Hash;

/// A game defined only by its rules — no index required. The state must be
/// hashable so reachable positions can be deduplicated.
pub trait RulesGame {
    type State: Clone + Eq + Hash;
    fn start(&self) -> Self::State;
    /// Positions reachable by one legal move (opponent then to move).
    fn successors(&self, state: &Self::State) -> Vec<Self::State>;
    /// `Some(outcome)` (from the side to move) if terminal, else `None`.
    fn terminal(&self, state: &Self::State) -> Option<Outcome>;
}

/// The solved value of every reachable position.
pub struct ReachableSolution<S> {
    value: HashMap<S, Outcome>,
}

impl<S: Eq + Hash> ReachableSolution<S> {
    pub fn get(&self, state: &S) -> Option<Outcome> {
        self.value.get(state).copied()
    }
    pub fn len(&self) -> usize {
        self.value.len()
    }
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
    pub fn count(&self, outcome: Outcome) -> usize {
        self.value.values().filter(|&&v| v == outcome).count()
    }

    /// Iterate every reachable position and its solved value.
    pub fn iter(&self) -> impl Iterator<Item = (&S, Outcome)> {
        self.value.iter().map(|(s, &o)| (s, o))
    }
}

/// Strongly solve a game by discovering its reachable positions and running
/// retrograde analysis over that graph. `cap` bounds the discovery (states); the
/// solve aborts and returns `None` if it is exceeded, so a mis-estimated game
/// can't OOM the machine.
pub fn solve_reachable_capped<G: RulesGame>(
    game: &G,
    cap: usize,
) -> Option<ReachableSolution<G::State>> {
    // 1. Forward discovery: assign every reachable state a dense id.
    let mut id: HashMap<G::State, u32> = HashMap::new();
    let mut states: Vec<G::State> = Vec::new();
    let start = game.start();
    id.insert(start.clone(), 0);
    states.push(start);
    let mut i = 0;
    while i < states.len() {
        if states.len() > cap {
            return None;
        }
        let s = states[i].clone();
        if game.terminal(&s).is_none() {
            for ns in game.successors(&s) {
                if !id.contains_key(&ns) {
                    id.insert(ns.clone(), states.len() as u32);
                    states.push(ns);
                }
            }
        }
        i += 1;
    }

    // 2. Build the graph: terminal labels, out-degrees, reverse edges.
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Label {
        Unknown,
        Win,
        Loss,
        Draw,
    }
    let n = states.len();
    let mut label = vec![Label::Unknown; n];
    let mut unresolved = vec![0u32; n];
    let mut rev: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut queue: Vec<u32> = Vec::new();

    for (idx, s) in states.iter().enumerate() {
        match game.terminal(s) {
            Some(o) => {
                label[idx] = match o {
                    Outcome::Win => Label::Win,
                    Outcome::Loss => Label::Loss,
                    Outcome::Draw => Label::Draw,
                };
                if matches!(label[idx], Label::Win | Label::Loss) {
                    queue.push(idx as u32);
                }
            }
            None => {
                let succ = game.successors(s);
                unresolved[idx] = succ.len() as u32;
                for ns in &succ {
                    let j = id[ns] as usize;
                    rev[j].push(idx as u32);
                }
            }
        }
    }

    // 3. Retrograde: Loss makes predecessors Win; a Win decrements predecessors,
    //    Loss when all moves are exhausted; anything unresolved is a Draw.
    let mut head = 0;
    while head < queue.len() {
        let t = queue[head] as usize;
        head += 1;
        let vt = label[t];
        let preds = std::mem::take(&mut rev[t]);
        for &p in &preds {
            let p = p as usize;
            if label[p] != Label::Unknown {
                continue;
            }
            match vt {
                Label::Loss => {
                    label[p] = Label::Win;
                    queue.push(p as u32);
                }
                Label::Win => {
                    unresolved[p] -= 1;
                    if unresolved[p] == 0 {
                        label[p] = Label::Loss;
                        queue.push(p as u32);
                    }
                }
                Label::Draw | Label::Unknown => {}
            }
        }
    }

    let mut value = HashMap::with_capacity(n);
    for (idx, s) in states.into_iter().enumerate() {
        let o = match label[idx] {
            Label::Win => Outcome::Win,
            Label::Loss => Outcome::Loss,
            _ => Outcome::Draw,
        };
        value.insert(s, o);
    }
    Some(ReachableSolution { value })
}

/// Convenience wrapper with a generous default cap.
pub fn solve_reachable<G: RulesGame>(game: &G) -> ReachableSolution<G::State> {
    solve_reachable_capped(game, 200_000_000)
        .expect("reachable state count exceeded the default cap")
}
