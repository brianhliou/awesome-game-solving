//! WASM bindings that put the *validated* Rust engine in the browser, so the
//! explorer never re-implements morris rules in JS (a re-implementation could
//! silently disagree with the solve). The browser gets legal-move generation and
//! perfect-play probing straight from the engine; the tablebase is loaded as raw
//! bytes and probed via the engine's own dense index.
//!
//! Wire format keeps the JS boundary dumb: positions are `[white, black, w_hand,
//! b_hand, turn]`, move lists are flat `u32` groups of 6 (`position…, value`),
//! and values are `0=win 1=loss 2=draw 3=unknown` from the relevant side.

use game_solver::dense_solve::{solve_dense, DenseSolution};
use game_solver::games::generic_morris::{Board, GenericMorris, State};
use game_solver::games::y::{Pos as YPos, Y};
use game_solver::{Game, Outcome, RulesGame};
use wasm_bindgen::prelude::*;

/// One morris board + its loaded WLD tablebase.
#[wasm_bindgen]
pub struct Explorer {
    game: GenericMorris,
    tb: Vec<u8>,
}

#[wasm_bindgen]
impl Explorer {
    /// `rings` = 2 (six men's, 16 points) or 3 (nine men's, 24 points); `men` per side.
    #[wasm_bindgen(constructor)]
    pub fn new(rings: usize, men: u8) -> Explorer {
        Explorer { game: GenericMorris::new(Board::rings(rings, men)), tb: Vec::new() }
    }

    /// Number of dense-index slots (the tablebase addresses these). f64 because it
    /// can exceed u32.
    pub fn num_states(&self) -> f64 {
        Game::num_states(&self.game) as f64
    }

    pub fn board_points(&self) -> usize {
        self.game.board.n
    }

    /// Load the packed 2-bit WLD tablebase (gzip-decompressed bytes).
    pub fn set_tablebase(&mut self, bytes: Vec<u8>) {
        self.tb = bytes;
    }

    pub fn has_tablebase(&self) -> bool {
        !self.tb.is_empty()
    }

    /// The starting position `[white, black, w_hand, b_hand, turn]`.
    pub fn start(&self) -> Vec<u32> {
        pack_state(&RulesGame::start(&self.game)).to_vec()
    }

    /// Perfect-play value for the side to move: 0 win, 1 loss, 2 draw, 3 unknown.
    pub fn value(&self, white: u32, black: u32, w_hand: u32, b_hand: u32, turn: u32) -> u32 {
        self.value_of(&unpack_state(white, black, w_hand, b_hand, turn))
    }

    /// Legal moves as flat `u32` groups of 6: `[white, black, w_hand, b_hand,
    /// turn, value]`, where `value` is the move's worth *to the mover*
    /// (0 win, 1 loss, 2 draw, 3 unknown).
    pub fn legal_moves(&self, white: u32, black: u32, w_hand: u32, b_hand: u32, turn: u32) -> Vec<u32> {
        let s = unpack_state(white, black, w_hand, b_hand, turn);
        let mut out = Vec::new();
        for ns in RulesGame::successors(&self.game, &s) {
            // After the move the opponent is to move, so the stored value of `ns`
            // is from the opponent's view; invert it to score the mover's choice.
            let mover_val = invert(self.value_of(&ns));
            out.extend_from_slice(&pack_state(&ns));
            out.push(mover_val);
        }
        out
    }

    /// `Some` terminal value (0/1/2) for the side to move, or 3 if not terminal.
    pub fn terminal(&self, white: u32, black: u32, w_hand: u32, b_hand: u32, turn: u32) -> u32 {
        let s = unpack_state(white, black, w_hand, b_hand, turn);
        match RulesGame::terminal(&self.game, &s) {
            Some(Outcome::Win) => 0,
            Some(Outcome::Loss) => 1,
            Some(Outcome::Draw) => 2,
            None => 3,
        }
    }
}

impl Explorer {
    fn value_of(&self, s: &State) -> u32 {
        if self.tb.is_empty() {
            return 3;
        }
        let idx = Game::index(&self.game, s);
        let byte_i = (idx >> 2) as usize;
        if byte_i >= self.tb.len() {
            return 3;
        }
        match (self.tb[byte_i] >> ((idx & 3) * 2)) & 3 {
            1 => 0, // win
            2 => 1, // loss
            3 => 2, // draw
            _ => 3, // unreachable / not in db
        }
    }
}

#[inline]
fn pack_state(s: &State) -> [u32; 5] {
    [s.white, s.black, s.w_hand as u32, s.b_hand as u32, s.turn as u32]
}

#[inline]
fn unpack_state(white: u32, black: u32, w_hand: u32, b_hand: u32, turn: u32) -> State {
    State { white, black, w_hand: w_hand as u8, b_hand: b_hand as u8, turn: turn as u8 }
}

/// Flip a value between the two players (win<->loss; draw and unknown fixed).
#[inline]
fn invert(v: u32) -> u32 {
    match v {
        0 => 1,
        1 => 0,
        other => other,
    }
}

/// Number of dense-index slots for the side-`n` Y board, i.e. how many positions
/// `solve_y` will resolve. Cheap (a few binomials) — the UI shows it before the
/// solve starts. `f64` to stay safe past `u32`, though interactive sides fit easily.
#[wasm_bindgen]
pub fn y_num_states(side: usize) -> f64 {
    Game::num_states(&Y::new(side)) as f64
}

/// Strongly solve the side-`n` Y board and return the raw per-index value bytes
/// (`1=win 2=loss 3=draw`, one byte each). This is the heavy call; the explorer
/// runs it inside a Web Worker so the page never blocks, then hands the bytes to
/// [`YExplorer::from_solution`] on the main thread for synchronous probing.
#[wasm_bindgen]
pub fn solve_y(side: usize) -> Vec<u8> {
    solve_dense(&Y::new(side), |_, _| {}).values
}

/// One Y board (the Schensted-Titus connection game) plus its solved value table.
///
/// Y at side <= 5 is small enough (side-5 = 3.49M dense-index positions) to solve
/// in the browser, so — unlike the morris explorer, which ships a precomputed tablebase —
/// the table is computed live by [`solve_y`] (in a worker) and wrapped here. The
/// wire format is `[p1, p2]` for a position (two stone bitmasks; the side to move
/// is derived from the counts) and groups of 3 (`p1, p2, value`) for moves, value
/// `0=win 1=loss 2=draw 3=unknown` for the mover.
#[wasm_bindgen]
pub struct YExplorer {
    game: Y,
    sol: DenseSolution,
}

#[wasm_bindgen]
impl YExplorer {
    /// Build the side-`n` board and strongly solve it inline (dense solver). This
    /// blocks the caller; prefer the worker path ([`solve_y`] +
    /// [`YExplorer::from_solution`]) for interactive use. Kept as a fallback.
    #[wasm_bindgen(constructor)]
    pub fn new(side: usize) -> YExplorer {
        let game = Y::new(side);
        let sol = solve_dense(&game, |_, _| {});
        YExplorer { game, sol }
    }

    /// Wrap a precomputed value table (from [`solve_y`], typically solved off the
    /// main thread) into a probe-able explorer. `values` must have one byte per
    /// dense index of the side-`n` board, exactly as `solve_y` returns.
    pub fn from_solution(side: usize, values: Vec<u8>) -> YExplorer {
        let game = Y::new(side);
        let sol = DenseSolution { values, rounds: 0, terminal_wins: 0, terminal_losses: 0 };
        YExplorer { game, sol }
    }

    pub fn side(&self) -> usize {
        self.game.n
    }

    pub fn cells(&self) -> usize {
        self.game.cells
    }

    /// The starting position `[p1, p2]` (empty board).
    pub fn start(&self) -> Vec<u32> {
        let s = RulesGame::start(&self.game);
        vec![s.p1 as u32, s.p2 as u32]
    }

    /// Perfect-play value for the side to move: 0 win, 1 loss, 2 draw, 3 unknown.
    pub fn value(&self, p1: u32, p2: u32) -> u32 {
        self.val(p1, p2)
    }

    /// `Some` terminal value (0/1/2) for the side to move, or 3 if not terminal.
    pub fn terminal(&self, p1: u32, p2: u32) -> u32 {
        let s = YPos { p1: p1 as u64, p2: p2 as u64 };
        match RulesGame::terminal(&self.game, &s) {
            Some(Outcome::Win) => 0,
            Some(Outcome::Loss) => 1,
            Some(Outcome::Draw) => 2,
            None => 3,
        }
    }

    /// Legal moves as flat `u32` groups of 3: `[p1, p2, value]`, where `value` is
    /// the move's worth *to the mover* (0 win, 1 loss, 2 draw, 3 unknown).
    pub fn legal_moves(&self, p1: u32, p2: u32) -> Vec<u32> {
        let s = YPos { p1: p1 as u64, p2: p2 as u64 };
        let mut out = Vec::new();
        for ns in RulesGame::successors(&self.game, &s) {
            // After the move the opponent is to move, so invert to score the mover.
            let mover_val = invert(self.val(ns.p1 as u32, ns.p2 as u32));
            out.push(ns.p1 as u32);
            out.push(ns.p2 as u32);
            out.push(mover_val);
        }
        out
    }
}

impl YExplorer {
    fn val(&self, p1: u32, p2: u32) -> u32 {
        let s = YPos { p1: p1 as u64, p2: p2 as u64 };
        match self.sol.value_at(Game::index(&self.game, &s)) {
            Outcome::Win => 0,
            Outcome::Loss => 1,
            Outcome::Draw => 2,
        }
    }
}
