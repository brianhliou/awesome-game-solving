//! Six men's morris — the two-ring (16-point) board, 6 men per side, no diagonals,
//! no flying. Standard capture rules: forming a mill (three in a line) removes one
//! opponent man (one not itself in a mill, unless all are); a player loses when
//! reduced below 3 men or left with no legal move.
//!
//! Solved through [`crate::RulesGame`] / [`crate::solve_reachable`] — capture games
//! drop men off the board, so a dense index is awkward; we enumerate the reachable
//! graph instead.
//!
//! Board points (two concentric squares joined at the side-midpoints):
//! ```text
//!   0 --- 1 --- 2          outer ring: 0..7  (corners 0,2,4,6; mids 1,3,5,7)
//!   |   8-9-10  |          inner ring: 8..15 (corners 8,10,12,14; mids 9,11,13,15)
//!   7  15   11  3          spokes join outer mid -> inner mid: 1-9, 3-11, 5-13, 7-15
//!   |  14-13-12 |
//!   6 --- 5 --- 4
//! ```

use crate::{Outcome, RulesGame};

const WHITE: u8 = 1;
const BLACK: u8 = 2;
const MEN: u8 = 6;

// Adjacency bitmask per point (for sliding). Corners have 2 neighbours, mids 3.
const ADJ: [u16; 16] = [
    130, 517, 10, 2068, 40, 8272, 160, 32833, 33280, 1282, 2560, 5128, 10240, 20512, 40960, 16768,
];

// The eight mills: the four sides of each square. (The spokes are only two points
// long on a two-ring board, so they are not mills.)
const MILLS: [u16; 8] = [7, 28, 112, 193, 1792, 7168, 28672, 49408];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub white: u16, // bitmask over the 16 points
    pub black: u16,
    pub w_hand: u8, // white men not yet placed
    pub b_hand: u8,
    pub turn: u8, // WHITE or BLACK
}

pub struct SixMensMorris;

#[inline]
fn completes_mill(mask: u16, point: usize) -> bool {
    MILLS
        .iter()
        .any(|&m| (m >> point) & 1 == 1 && mask & m == m)
}

#[inline]
fn in_any_mill(mask: u16, point: usize) -> bool {
    completes_mill(mask, point)
}

/// The opponent men that may be captured: those not in a mill, unless every
/// opponent man is in a mill (then any may be taken).
fn removable(opp: u16) -> u16 {
    let mut free = 0u16;
    for p in 0..16 {
        if (opp >> p) & 1 == 1 && !in_any_mill(opp, p) {
            free |= 1 << p;
        }
    }
    if free == 0 {
        opp
    } else {
        free
    }
}

#[inline]
fn has_slide(stm: u16, occupied: u16) -> bool {
    for f in 0..16 {
        if (stm >> f) & 1 == 1 && ADJ[f] & !occupied != 0 {
            return true;
        }
    }
    false
}

impl State {
    /// Build the position after the mover's move, with the turn flipped.
    fn after(&self, mover: u16, mover_hand: u8, opp: u16) -> State {
        if self.turn == WHITE {
            State { white: mover, black: opp, w_hand: mover_hand, b_hand: self.b_hand, turn: BLACK }
        } else {
            State { white: opp, black: mover, w_hand: self.w_hand, b_hand: mover_hand, turn: WHITE }
        }
    }
}

impl RulesGame for SixMensMorris {
    type State = State;

    fn start(&self) -> State {
        State { white: 0, black: 0, w_hand: MEN, b_hand: MEN, turn: WHITE }
    }

    fn successors(&self, s: &State) -> Vec<State> {
        let (stm, opp, stm_hand) = if s.turn == WHITE {
            (s.white, s.black, s.w_hand)
        } else {
            (s.black, s.white, s.b_hand)
        };
        let occ = s.white | s.black;
        let mut out = Vec::new();

        // Base moves: (resulting mover mask, destination point).
        let mut bases: Vec<(u16, usize)> = Vec::new();
        if stm_hand > 0 {
            for p in 0..16 {
                if (occ >> p) & 1 == 0 {
                    bases.push((stm | (1 << p), p));
                }
            }
        } else {
            for f in 0..16 {
                if (stm >> f) & 1 == 1 {
                    let adj = ADJ[f];
                    for t in 0..16 {
                        if (adj >> t) & 1 == 1 && (occ >> t) & 1 == 0 {
                            bases.push(((stm & !(1 << f)) | (1 << t), t));
                        }
                    }
                }
            }
        }

        for (mover, dest) in bases {
            let new_hand = if stm_hand > 0 { stm_hand - 1 } else { 0 };
            if completes_mill(mover, dest) && opp != 0 {
                let rem = removable(opp);
                for q in 0..16 {
                    if (rem >> q) & 1 == 1 {
                        out.push(s.after(mover, new_hand, opp & !(1 << q)));
                    }
                }
            } else {
                out.push(s.after(mover, new_hand, opp));
            }
        }
        out
    }

    fn terminal(&self, s: &State) -> Option<Outcome> {
        // Only the movement phase (both hands empty) has terminal positions:
        // the side to move loses if reduced below 3 men or unable to move.
        if s.w_hand == 0 && s.b_hand == 0 {
            let stm = if s.turn == WHITE { s.white } else { s.black };
            if stm.count_ones() < 3 {
                return Some(Outcome::Loss);
            }
            if !has_slide(stm, s.white | s.black) {
                return Some(Outcome::Loss);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(points: &[usize]) -> u16 {
        points.iter().fold(0u16, |m, &p| m | (1 << p))
    }

    #[test]
    fn mill_detection() {
        let m = mask(&[0, 1, 2]); // top side of the outer square
        assert!(completes_mill(m, 1));
        assert!(!completes_mill(mask(&[0, 1]), 1));
    }

    #[test]
    fn placing_into_a_mill_captures() {
        // White at 0,1 with a man in hand; Black at 8,9,10 (a mill) and 11.
        let s = State {
            white: mask(&[0, 1]),
            black: mask(&[8, 9, 10, 11]),
            w_hand: 1,
            b_hand: 0,
            turn: WHITE,
        };
        let succ = SixMensMorris.successors(&s);
        // White can place on any of the empty points; placing at 2 completes the
        // 0-1-2 mill and must capture a Black man. Black's only non-mill man is 11
        // (8,9,10 are a mill), so exactly one capture target.
        let captures: Vec<_> = succ
            .iter()
            .filter(|n| n.black.count_ones() == 3) // a Black man was removed
            .collect();
        assert!(!captures.is_empty(), "placing into the mill should capture");
        for c in &captures {
            // the removed man must be 11 (the only one not in a mill)
            assert_eq!(c.black, mask(&[8, 9, 10]), "must capture the non-mill man");
        }
    }

    #[test]
    fn reduced_below_three_is_a_loss() {
        // Movement phase, White (to move) has only two men: a loss.
        let s = State {
            white: mask(&[0, 1]),
            black: mask(&[8, 9, 10]),
            w_hand: 0,
            b_hand: 0,
            turn: WHITE,
        };
        assert_eq!(SixMensMorris.terminal(&s), Some(Outcome::Loss));
    }

    #[test]
    fn start_is_not_terminal_and_has_moves() {
        let g = SixMensMorris;
        let s = g.start();
        assert_eq!(g.terminal(&s), None);
        assert_eq!(g.successors(&s).len(), 16); // first placement: any of 16 points
    }
}
