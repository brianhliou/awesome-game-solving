//! Y — the connection game of Schensted & Titus (1953). A triangular board of
//! hexagonal cells; players alternately place a stone of their colour on an empty
//! cell, and a player wins by connecting all THREE sides of the triangle with one
//! connected group of their stones. Hex reduces to Y.
//!
//! Y is the friendliest possible target for retrograde analysis:
//!   * **Pure placement** — stones are never moved or removed, so the board fills
//!     monotonically and the position graph is acyclic. The loopy retrograde never
//!     has to resolve a repetition cycle.
//!   * **No draws** — by the Y theorem (a Hex-style topological argument) a full
//!     board always has exactly one player connecting all three sides, and the two
//!     players can never connect simultaneously. So every reachable position is a
//!     Win or a Loss. A `Draw` in the solved output is therefore a *bug* — wrong
//!     board topology or wrong win detection — and we assert it cannot happen.
//!
//! Prior art: GamesCrafters' GamesmanClassic carries an undocumented Y solver
//! (`mgameofy.c`, A. Esteban, 2023) that declares `kTieIsPossible = TRUE`, which is
//! incorrect for Y. This plugin instead treats the no-draw property as a hard
//! correctness invariant.
//!
//! Two solving paths share the same rules (see the inherent methods below):
//!   * [`RulesGame`] + `solve_reachable` — hash-map discovery, used for sides 1–5
//!     in-core (side-5 is 3.34M reachable positions).
//!   * [`Game`] + `solve_dense` — a tight, count-aware dense index (built on the
//!     combinadic in [`crate::index`]) that fits side-6's 2.17B positions under a
//!     `u32`, so the one-byte-per-index dense solver can reach it. Side-7's index
//!     exceeds `u32`; that is the external-memory / slice target.

use crate::index::{
    mask_to_points, rank_black_in_empties, rank_subset, unrank_black_in_empties, unrank_subset,
    Binom,
};
use crate::reachable::RulesGame;
use crate::{Game, Outcome};

const P1: u8 = 1; // first player
const P2: u8 = 2; // second player

/// One ply-slice of the dense index: all positions with `k1` first-player and `k2`
/// second-player stones. Sizes `white_choices = C(cells, k1)` ways to place white,
/// each with `black_choices = C(cells - k1, k2)` ways to place black in the empties.
struct Slice {
    k1: usize,
    k2: usize,
    offset: u64,
    black_choices: u64,
}

/// A Y board of side `n`: `n` rows, row `r` holding `r + 1` cells, `n(n+1)/2` total.
/// Cells are indexed row-major: `idx(r, c) = r*(r+1)/2 + c` for `0 <= c <= r`.
pub struct Y {
    pub n: usize,
    pub cells: usize,
    adj: Vec<u64>, // adj[i] = bitmask of the cells adjacent to cell i
    edge_a: u64,   // bottom row (r == n-1)
    edge_b: u64,   // left edge (c == 0)
    edge_c: u64,   // right edge (c == r)
    // Dense-index support (built only when cells <= 32, the u32-bitboard limit the
    // combinadic in `index.rs` uses). `slices[m]` covers the m-stone ply.
    binom: Binom,
    slices: Vec<Slice>,
    num: u64,
}

/// A position: the cells each player occupies, as bitmasks over `0..cells`. The
/// side to move is derived from the stone counts (the first player moves first).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pos {
    pub p1: u64,
    pub p2: u64,
}

impl Y {
    /// Build the side-`n` board: precompute the 6-neighbour hex adjacency, the three
    /// edge masks, and (for `cells <= 32`) the dense-index slice table.
    pub fn new(n: usize) -> Self {
        let cells = n * (n + 1) / 2;
        assert!(cells <= 64, "Y::new: side {n} has {cells} cells, over the 64-bit board");
        let idx = |r: usize, c: usize| -> usize { r * (r + 1) / 2 + c };

        let mut adj = vec![0u64; cells];
        let (mut edge_a, mut edge_b, mut edge_c) = (0u64, 0u64, 0u64);
        // Triangular-grid hex adjacency: same-row left/right, and the two cells in
        // each adjacent row. This is the topology under which Hex reduces to Y.
        let offsets: [(isize, isize); 6] =
            [(0, -1), (0, 1), (-1, -1), (-1, 0), (1, 0), (1, 1)];
        for r in 0..n {
            for c in 0..=r {
                let i = idx(r, c);
                for (dr, dc) in offsets {
                    let nr = r as isize + dr;
                    let nc = c as isize + dc;
                    if nr >= 0 && (nr as usize) < n && nc >= 0 && nc <= nr {
                        adj[i] |= 1u64 << idx(nr as usize, nc as usize);
                    }
                }
                if r == n - 1 {
                    edge_a |= 1u64 << i;
                }
                if c == 0 {
                    edge_b |= 1u64 << i;
                }
                if c == r {
                    edge_c |= 1u64 << i;
                }
            }
        }

        // Dense-index slice table: one slice per ply m = 0..=cells, with
        // k1 = ceil(m/2) first-player stones and k2 = floor(m/2) second-player.
        let binom = Binom::new();
        let mut slices = Vec::new();
        let mut num = 0u64;
        if cells <= 32 {
            for m in 0..=cells {
                let k1 = m.div_ceil(2);
                let k2 = m / 2;
                let white_choices = binom.c(cells, k1);
                let black_choices = binom.c(cells - k1, k2);
                slices.push(Slice { k1, k2, offset: num, black_choices });
                num += white_choices * black_choices;
            }
        }

        Y { n, cells, adj, edge_a, edge_b, edge_c, binom, slices, num }
    }

    fn to_move(&self, p: &Pos) -> u8 {
        if p.p1.count_ones() == p.p2.count_ones() {
            P1
        } else {
            P2
        }
    }

    /// Does `mask` contain a connected group touching all three edges?
    fn connects(&self, mask: u64) -> bool {
        let mut remaining = mask;
        while remaining != 0 {
            // Flood-fill the component containing the lowest remaining cell.
            let mut comp = 0u64;
            let mut frontier = remaining & remaining.wrapping_neg();
            while frontier != 0 {
                comp |= frontier;
                let mut next = 0u64;
                let mut f = frontier;
                while f != 0 {
                    next |= self.adj[f.trailing_zeros() as usize];
                    f &= f - 1;
                }
                frontier = next & mask & !comp;
            }
            if comp & self.edge_a != 0 && comp & self.edge_b != 0 && comp & self.edge_c != 0 {
                return true;
            }
            remaining &= !comp;
        }
        false
    }

    // --- Shared rules, used by both the RulesGame and Game trait impls. ---

    fn start_pos(&self) -> Pos {
        Pos { p1: 0, p2: 0 }
    }

    fn moves(&self, p: &Pos) -> Vec<Pos> {
        // Only called on non-terminal positions, where the board is not full, so an
        // empty cell always exists. The mover places one stone.
        let board = if self.cells == 64 {
            u64::MAX
        } else {
            (1u64 << self.cells) - 1
        };
        let mut empties = board & !(p.p1 | p.p2);
        let mover = self.to_move(p);
        let mut out = Vec::with_capacity(empties.count_ones() as usize);
        while empties != 0 {
            let bit = empties & empties.wrapping_neg();
            let mut next = *p;
            if mover == P1 {
                next.p1 |= bit;
            } else {
                next.p2 |= bit;
            }
            out.push(next);
            empties &= empties - 1;
        }
        out
    }

    fn outcome(&self, p: &Pos) -> Option<Outcome> {
        // The player who just moved is the only one who can have completed a
        // connection; if either colour connects, the side to move has lost. Y has
        // no draw branch.
        if self.connects(p.p1) || self.connects(p.p2) {
            Some(Outcome::Loss)
        } else {
            None
        }
    }
}

impl RulesGame for Y {
    type State = Pos;
    fn start(&self) -> Pos {
        self.start_pos()
    }
    fn successors(&self, p: &Pos) -> Vec<Pos> {
        self.moves(p)
    }
    fn terminal(&self, p: &Pos) -> Option<Outcome> {
        self.outcome(p)
    }
}

impl Game for Y {
    type State = Pos;

    fn num_states(&self) -> u64 {
        self.num
    }

    fn index(&self, p: &Pos) -> u64 {
        let m = (p.p1.count_ones() + p.p2.count_ones()) as usize;
        let s = &self.slices[m];
        let white_pts = mask_to_points(p.p1 as u32);
        let white_rank = rank_subset(&self.binom, &white_pts);
        let black_rank = rank_black_in_empties(&self.binom, p.p1 as u32, p.p2 as u32);
        s.offset + white_rank * s.black_choices + black_rank
    }

    fn from_index(&self, i: u64) -> Option<Pos> {
        // Locate the slice whose [offset, offset+size) contains i.
        let m = match self.slices.iter().rposition(|s| s.offset <= i) {
            Some(m) => m,
            None => return None,
        };
        let s = &self.slices[m];
        let local = i - s.offset;
        let white_rank = local / s.black_choices;
        let black_rank = local % s.black_choices;
        let white_pts = unrank_subset(&self.binom, white_rank, self.cells, s.k1);
        let white: u32 = white_pts.iter().fold(0u32, |acc, &p| acc | (1u32 << p));
        let black =
            unrank_black_in_empties(&self.binom, white, black_rank, self.cells, s.k2);
        Some(Pos { p1: white as u64, p2: black as u64 })
    }

    fn start(&self) -> Pos {
        self.start_pos()
    }
    fn successors(&self, p: &Pos) -> Vec<Pos> {
        self.moves(p)
    }
    fn terminal(&self, p: &Pos) -> Option<Outcome> {
        self.outcome(p)
    }
}

/// The result of a slice-by-slice solve: the start value and aggregate counts over
/// the full tight index, with a per-ply breakdown.
pub struct SlicedSolution {
    pub start: Outcome,
    pub total: u64,
    pub wins: u64,
    pub losses: u64,
    /// `(ply m, slice size, wins, losses)`, ply-ascending.
    pub per_slice: Vec<(usize, u64, u64, u64)>,
}

impl Y {
    /// Strongly solve Y by a single backward sweep over the ply-slices, holding only
    /// two adjacent slices in memory (peak ≈ twice the largest slice). Y's monotone
    /// fill makes the position graph acyclic and layered — every successor of a
    /// ply-`m` position lies in ply `m+1` — so solving from the full board down to
    /// the empty board needs no fixpoint iteration. This is what reaches side-6
    /// (2.17B positions) on a laptop where the global dense solver would not.
    ///
    /// There is no draw branch (Y cannot draw); a non-terminal position with no
    /// move (a full board that fails to connect — i.e. a board-model bug) trips an
    /// assertion rather than silently resolving.
    pub fn solve_sliced(&self, mut report: impl FnMut(usize, u64)) -> SlicedSolution {
        assert!(!self.slices.is_empty(), "Y::solve_sliced: dense index unavailable (cells > 32)");
        const WIN: u8 = 1;
        const LOSS: u8 = 2;
        let cells = self.cells;

        let mut next: Vec<u8> = Vec::new(); // values of slice m+1, local-indexed
        let mut per_slice = vec![(0usize, 0u64, 0u64, 0u64); cells + 1];
        let (mut total_w, mut total_l) = (0u64, 0u64);

        for m in (0..=cells).rev() {
            let offset = self.slices[m].offset;
            let next_offset = if m < cells { self.slices[m + 1].offset } else { self.num };
            let size = (next_offset - offset) as usize;
            let mut cur = vec![0u8; size];
            let (mut w, mut l) = (0u64, 0u64);

            for local in 0..size {
                let p = self.from_index(offset + local as u64).expect("slice index decodes");
                let v = if self.outcome(&p).is_some() {
                    // Terminal: the opponent just connected, so the side to move lost.
                    LOSS
                } else {
                    let succ = self.moves(&p);
                    assert!(!succ.is_empty(), "non-terminal Y position with no move — board model bug");
                    let next_off = self.slices[m + 1].offset;
                    // Win iff some move hands the opponent a losing position.
                    let mut any_loss = false;
                    for ns in &succ {
                        let nidx = (self.index(ns) - next_off) as usize;
                        if next[nidx] == LOSS {
                            any_loss = true;
                            break;
                        }
                    }
                    if any_loss { WIN } else { LOSS }
                };
                cur[local] = v;
                if v == WIN {
                    w += 1;
                } else {
                    l += 1;
                }
            }

            total_w += w;
            total_l += l;
            per_slice[m] = (m, size as u64, w, l);
            report(m, size as u64);
            next = cur; // slice m becomes the "next" slice for m-1
        }

        // After the sweep, `next` holds slice 0: the single empty-board position.
        let start = if next[0] == WIN { Outcome::Win } else { Outcome::Loss };
        SlicedSolution { start, total: total_w + total_l, wins: total_w, losses: total_l, per_slice }
    }
}
