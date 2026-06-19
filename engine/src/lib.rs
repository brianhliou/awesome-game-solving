//! A generic retrograde-analysis engine for strongly solving small, finite,
//! perfect-information, two-player games.
//!
//! The engine knows no rules. It operates entirely through the [`Game`] trait:
//! a game supplies a way to enumerate, index, and step its positions, and the
//! engine supplies the strong solve (a game-theoretic value for *every* reachable
//! position) via backward induction that handles cycles (draws by repetition).
//!
//! This is the same shape as GAMESMAN / OpenSpiel / Ludii: one solver, many games.

pub mod dense_solve;
pub mod games;
pub mod index;
pub mod reachable;

pub use reachable::{solve_reachable, solve_reachable_capped, ReachableSolution, RulesGame};

/// The game-theoretic value of a position, from the perspective of the side to move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

/// A two-player, finite, perfect-information, zero-sum game.
///
/// Implementors provide the rules; the engine provides the solve. The only
/// per-game work is `successors`, `terminal`, and a dense `index`/`from_index`
/// pair that maps positions onto `0..num_states()`.
pub trait Game {
    /// A position. The side to move is encoded within the state.
    type State: Clone;

    /// The number of indices. Every state maps into `0..num_states()`.
    fn num_states(&self) -> u64;

    /// Dense perfect-hash of a state into `0..num_states()`.
    fn index(&self, state: &Self::State) -> u64;

    /// Inverse of [`index`](Game::index). Returns `None` for an index that
    /// decodes to an illegal or unreachable position (the engine treats those
    /// as inert).
    fn from_index(&self, index: u64) -> Option<Self::State>;

    /// The standard starting position.
    fn start(&self) -> Self::State;

    /// The positions reachable by one legal move. After a move the *opponent*
    /// is to move, so each returned state is from the opponent's perspective.
    fn successors(&self, state: &Self::State) -> Vec<Self::State>;

    /// `Some(outcome)` (from the side to move) if `state` is terminal, else `None`.
    fn terminal(&self, state: &Self::State) -> Option<Outcome>;
}

/// A strongly solved game: the value of every position.
pub struct Tablebase {
    values: Vec<Outcome>,
}

impl Tablebase {
    /// The value of the position with the given index.
    pub fn value(&self, index: u64) -> Outcome {
        self.values[index as usize]
    }

    /// Number of indexed positions.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// How many positions carry the given value.
    pub fn count(&self, outcome: Outcome) -> usize {
        self.values.iter().filter(|&&v| v == outcome).count()
    }
}

/// Probe the solved value of a position.
pub fn probe<G: Game>(game: &G, tb: &Tablebase, state: &G::State) -> Outcome {
    tb.value(game.index(state))
}

/// Strongly solve a game by retrograde analysis.
///
/// Backward induction from terminal positions, robust to cycles: a position is a
/// Win if *some* move reaches a position the opponent loses; a Loss if *every*
/// move reaches a position the opponent wins; otherwise (a move leads to a draw,
/// or a cycle never resolves) a Draw. Implemented as a reverse-edge BFS:
///
/// * seed the queue with terminal wins/losses,
/// * a resolved Loss makes every predecessor a Win,
/// * a resolved Win decrements each predecessor's unresolved-move count; when it
///   hits zero, that predecessor is a Loss,
/// * anything still unresolved at the end is a Draw.
pub fn solve<G: Game>(game: &G) -> Tablebase {
    let n = game.num_states() as usize;

    // Internal labels during the sweep; `None` == unresolved (becomes Draw).
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Label {
        Unknown,
        Win,
        Loss,
        Draw,
    }

    let mut label = vec![Label::Unknown; n];
    let mut unresolved_moves = vec![0u32; n]; // out-degree, decremented as moves resolve to Win
    let mut rev: Vec<Vec<u32>> = vec![Vec::new(); n]; // predecessors: rev[j] = states with j as a successor
    let mut queue: Vec<u32> = Vec::new();

    // Pass 1: classify every index, record terminals, and build reverse edges.
    for i in 0..n {
        let state = match game.from_index(i as u64) {
            Some(s) => s,
            None => {
                label[i] = Label::Draw; // illegal/unreachable: inert sink
                continue;
            }
        };
        match game.terminal(&state) {
            Some(o) => {
                label[i] = match o {
                    Outcome::Win => Label::Win,
                    Outcome::Loss => Label::Loss,
                    Outcome::Draw => Label::Draw,
                };
                // Only decided (win/loss) terminals drive propagation.
                if matches!(label[i], Label::Win | Label::Loss) {
                    queue.push(i as u32);
                }
            }
            None => {
                let succ = game.successors(&state);
                unresolved_moves[i] = succ.len() as u32;
                for s2 in &succ {
                    let j = game.index(s2) as usize;
                    rev[j].push(i as u32);
                }
            }
        }
    }

    // Pass 2: propagate backward from decided positions.
    let mut head = 0;
    while head < queue.len() {
        let t = queue[head] as usize;
        head += 1;
        let vt = label[t];
        // Take the predecessor list out to satisfy the borrow checker cheaply.
        let preds = std::mem::take(&mut rev[t]);
        for &p in &preds {
            let p = p as usize;
            if label[p] != Label::Unknown {
                continue;
            }
            match vt {
                // The position to move at `t` loses, so a predecessor moves to `t` and wins.
                Label::Loss => {
                    label[p] = Label::Win;
                    queue.push(p as u32);
                }
                // Moving to `t` hands the opponent a win; a bad move for `p`.
                Label::Win => {
                    unresolved_moves[p] -= 1;
                    if unresolved_moves[p] == 0 {
                        label[p] = Label::Loss;
                        queue.push(p as u32);
                    }
                }
                // Draws never propagate (they leave predecessors free to draw).
                Label::Draw | Label::Unknown => {}
            }
        }
    }

    let values = label
        .into_iter()
        .map(|l| match l {
            Label::Win => Outcome::Win,
            Label::Loss => Outcome::Loss,
            _ => Outcome::Draw, // Unknown (cyclic) and Draw both resolve to Draw
        })
        .collect();
    Tablebase { values }
}
