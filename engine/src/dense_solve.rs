//! A memory-frugal strong solver for densely-indexed games. Where
//! [`crate::solve`] materialises every reverse edge (`Vec<Vec<u32>>`) and
//! [`crate::solve_reachable`] holds a hash map of every state, this keeps only a
//! flat value array — one entry per index — and reaches the fixpoint by repeated
//! forward relaxation. That is the difference between fitting a ~10^7 game and a
//! ~10^9 one.
//!
//! It is the least-fixpoint of the same monotone operator the other solvers use,
//! so it returns identical values; it trades reverse-edge memory for re-scanning
//! unresolved states each round. (For the very largest boards an O(edges)
//! predecessor pass is the next optimisation; this is the validated baseline.)

use crate::{Game, Outcome};

const UNKNOWN: u8 = 0;
const WIN: u8 = 1;
const LOSS: u8 = 2;
const DRAW: u8 = 3;

/// The solved value of every index, plus a little instrumentation.
pub struct DenseSolution {
    /// One byte per index: [`WIN`], [`LOSS`], or [`DRAW`] (never `UNKNOWN`).
    pub values: Vec<u8>,
    pub rounds: u32,
    pub terminal_wins: u64,
    pub terminal_losses: u64,
}

impl DenseSolution {
    pub fn value_at(&self, index: u64) -> Outcome {
        match self.values[index as usize] {
            WIN => Outcome::Win,
            LOSS => Outcome::Loss,
            _ => Outcome::Draw,
        }
    }

    pub fn count(&self, outcome: Outcome) -> u64 {
        let want = match outcome {
            Outcome::Win => WIN,
            Outcome::Loss => LOSS,
            Outcome::Draw => DRAW,
        };
        self.values.iter().filter(|&&v| v == want).count() as u64
    }

    /// Pack the values to two bits each (00 unused, 01 win, 10 loss, 11 draw),
    /// little-endian within each byte. This is the on-disk tablebase payload.
    pub fn pack_2bit(&self) -> Vec<u8> {
        let mut out = vec![0u8; self.values.len().div_ceil(4)];
        for (i, &v) in self.values.iter().enumerate() {
            out[i >> 2] |= v << ((i & 3) * 2);
        }
        out
    }
}

/// Strongly solve a densely-indexed game by forward-relaxation to fixpoint.
///
/// `report` is called once per round with `(round, remaining_unknown)` so a
/// long solve can log progress. The index space must fit `u32` (≤ ~4.29e9
/// positions); larger boards need the slice/predecessor solver.
pub fn solve_dense<G: Game>(game: &G, mut report: impl FnMut(u32, usize)) -> DenseSolution {
    let n = game.num_states();
    assert!(n <= u32::MAX as u64, "index space exceeds u32; use the slice solver");
    let n = n as usize;

    let mut values = vec![UNKNOWN; n];
    let mut unknown: Vec<u32> = Vec::new();
    let mut terminal_wins = 0u64;
    let mut terminal_losses = 0u64;

    // Pass 0: classify terminals; everything else starts unknown.
    for i in 0..n {
        match game.from_index(i as u64) {
            None => values[i] = DRAW, // inert slot
            Some(s) => match game.terminal(&s) {
                Some(Outcome::Win) => {
                    values[i] = WIN;
                    terminal_wins += 1;
                }
                Some(Outcome::Loss) => {
                    values[i] = LOSS;
                    terminal_losses += 1;
                }
                Some(Outcome::Draw) => values[i] = DRAW,
                None => unknown.push(i as u32),
            },
        }
    }

    // Relax until a full pass changes nothing. A position is a Win if some move
    // reaches a Loss (for the opponent), a Loss if every move reaches a Win.
    let mut rounds = 0u32;
    loop {
        rounds += 1;
        let mut changed = false;
        let mut survivors: Vec<u32> = Vec::with_capacity(unknown.len());
        for &i in &unknown {
            let s = game.from_index(i as u64).expect("unknown slot decodes");
            let succ = game.successors(&s);
            let mut any_loss = false;
            let mut all_win = true;
            for ns in &succ {
                match values[game.index(ns) as usize] {
                    LOSS => any_loss = true,
                    WIN => {}
                    _ => all_win = false, // UNKNOWN or DRAW: not (yet) a win
                }
            }
            if any_loss {
                values[i as usize] = WIN;
                changed = true;
            } else if all_win && !succ.is_empty() {
                values[i as usize] = LOSS;
                changed = true;
            } else {
                survivors.push(i);
            }
        }
        unknown = survivors;
        report(rounds, unknown.len());
        if !changed {
            break;
        }
    }

    // Anything still unresolved is drawn (a cycle that never forced a decision).
    for &i in &unknown {
        values[i as usize] = DRAW;
    }

    DenseSolution { values, rounds, terminal_wins, terminal_losses }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::TicTacToe;

    #[test]
    fn tic_tac_toe_is_a_draw() {
        // The dense fixpoint solver must agree with the known tic-tac-toe result.
        let g = TicTacToe;
        let sol = solve_dense(&g, |_, _| {});
        let start = Game::index(&g, &Game::start(&g));
        assert_eq!(sol.value_at(start), Outcome::Draw);
    }
}
