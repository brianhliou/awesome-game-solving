//! Three men's morris (Tapatan ruleset: 3x3 grid, diagonals are both mill lines
//! and move connections). Each player has three men. Phase 1: alternately place
//! all three. Phase 2: slide a man to an adjacent point. A player wins by forming
//! a mill (all three men on a line) or by leaving the opponent with no legal move.
//! With only three men and a movement phase, positions can repeat — so this is the
//! engine's first genuinely *cyclic* game, where unresolved positions are draws.
//!
//! The mill lines and adjacency are the eight tic-tac-toe lines; adjacency is the
//! king-move graph they induce (centre connects to all eight).

use crate::{Game, Outcome, RulesGame};

const EMPTY: u8 = 0;
const WHITE: u8 = 1; // first player
const BLACK: u8 = 2;
const MEN: u8 = 3;

const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

// Adjacency bitmasks (king moves on the 3x3, derived from consecutive cells on
// the lines above): centre (4) touches all eight; corners/edges touch three.
const ADJ: [u16; 9] = [26, 21, 50, 81, 495, 276, 152, 336, 176];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pos {
    pub cells: [u8; 9],
    pub w_hand: u8, // white men not yet placed
    pub b_hand: u8,
    pub turn: u8, // WHITE or BLACK
}

pub struct ThreeMensMorris;

impl Pos {
    fn count(&self, player: u8) -> u8 {
        self.cells.iter().filter(|&&c| c == player).count() as u8
    }
    fn has_mill(&self, player: u8) -> bool {
        LINES
            .iter()
            .any(|l| l.iter().all(|&i| self.cells[i] == player))
    }
    fn hand(&self, player: u8) -> u8 {
        if player == WHITE {
            self.w_hand
        } else {
            self.b_hand
        }
    }
    fn other(player: u8) -> u8 {
        if player == WHITE {
            BLACK
        } else {
            WHITE
        }
    }
    /// Does `player` (to move) have at least one legal slide? (Movement phase only.)
    fn has_slide(&self, player: u8) -> bool {
        for i in 0..9 {
            if self.cells[i] == player {
                let m = ADJ[i];
                for j in 0..9 {
                    if (m >> j) & 1 == 1 && self.cells[j] == EMPTY {
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl Game for ThreeMensMorris {
    type State = Pos;

    fn num_states(&self) -> u64 {
        // base-3 board (3^9) x w_hand(0..3) x b_hand(0..3) x turn(2)
        3u64.pow(9) * 4 * 4 * 2
    }

    fn index(&self, p: &Pos) -> u64 {
        let board = p.cells.iter().fold(0u64, |a, &c| a * 3 + c as u64);
        let t = if p.turn == WHITE { 0 } else { 1 };
        ((board * 4 + p.w_hand as u64) * 4 + p.b_hand as u64) * 2 + t
    }

    fn from_index(&self, i: u64) -> Option<Pos> {
        let t = i % 2;
        let i = i / 2;
        let b_hand = (i % 4) as u8;
        let i = i / 4;
        let w_hand = (i % 4) as u8;
        let mut board = i / 4;
        let mut cells = [EMPTY; 9];
        for slot in cells.iter_mut().rev() {
            *slot = (board % 3) as u8;
            board /= 3;
        }
        let p = Pos {
            cells,
            w_hand,
            b_hand,
            turn: if t == 0 { WHITE } else { BLACK },
        };
        // No captures in three men's morris, so men-on-board + men-in-hand is
        // always exactly MEN. Anything else is unreachable.
        if p.count(WHITE) + w_hand == MEN && p.count(BLACK) + b_hand == MEN {
            Some(p)
        } else {
            None
        }
    }

    fn start(&self) -> Pos {
        Pos {
            cells: [EMPTY; 9],
            w_hand: MEN,
            b_hand: MEN,
            turn: WHITE,
        }
    }

    fn successors(&self, p: &Pos) -> Vec<Pos> {
        let stm = p.turn;
        let opp = Pos::other(stm);
        let mut out = Vec::new();
        if p.hand(stm) > 0 {
            // Placement: drop a man on any empty point.
            for i in 0..9 {
                if p.cells[i] == EMPTY {
                    let mut n = *p;
                    n.cells[i] = stm;
                    if stm == WHITE {
                        n.w_hand -= 1;
                    } else {
                        n.b_hand -= 1;
                    }
                    n.turn = opp;
                    out.push(n);
                }
            }
        } else {
            // Movement: slide a man to an adjacent empty point.
            for i in 0..9 {
                if p.cells[i] == stm {
                    let m = ADJ[i];
                    for j in 0..9 {
                        if (m >> j) & 1 == 1 && p.cells[j] == EMPTY {
                            let mut n = *p;
                            n.cells[i] = EMPTY;
                            n.cells[j] = stm;
                            n.turn = opp;
                            out.push(n);
                        }
                    }
                }
            }
        }
        out
    }

    fn terminal(&self, p: &Pos) -> Option<Outcome> {
        let wm = p.has_mill(WHITE);
        let bm = p.has_mill(BLACK);
        if wm && bm {
            return Some(Outcome::Draw); // unreachable; inert
        }
        let stm = p.turn;
        // The opponent just moved; if they completed a mill, the side to move lost.
        let opponent_mill = if stm == WHITE { bm } else { wm };
        if opponent_mill {
            return Some(Outcome::Loss);
        }
        // Movement phase with no legal slide: the side to move is stuck and loses.
        if p.w_hand == 0 && p.b_hand == 0 && !p.has_slide(stm) {
            return Some(Outcome::Loss);
        }
        None
    }
}

// Also expose the rules through the index-free solver, so it can be cross-checked
// against the dense `solve` (both must agree the start is a win).
impl RulesGame for ThreeMensMorris {
    type State = Pos;
    fn start(&self) -> Pos {
        Game::start(self)
    }
    fn successors(&self, s: &Pos) -> Vec<Pos> {
        Game::successors(self, s)
    }
    fn terminal(&self, s: &Pos) -> Option<Outcome> {
        Game::terminal(self, s)
    }
}
