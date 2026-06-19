//! A morris family solver parameterised by board size, so one implementation
//! covers six men's morris (two-ring, 16 points) and nine men's morris
//! (three-ring, 24 points). The rules mirror [`super::morris`] exactly; this
//! module adds two things that engine had no need for at 42M states but does at
//! ~10^10:
//!
//! 1. a **dense combinatorial index** (via [`crate::index`]) so positions address
//!    a flat array instead of a hash map, and
//! 2. the board's **symmetry group** (D4 × ring-reversal = 16 maps) and a
//!    canonicaliser, so equivalent positions can be folded.
//!
//! The board is built ring-by-ring and every generated symmetry is checked to be
//! a genuine automorphism (preserves adjacency and the set of mills), so a wrong
//! generator fails loudly rather than silently corrupting a solve.

use crate::index::{
    mask_to_points, rank_black_in_empties, rank_subset, unrank_black_in_empties, unrank_subset,
    Binom,
};
use crate::{Game, Outcome, RulesGame};
use std::collections::HashSet;

pub const WHITE: u8 = 1;
pub const BLACK: u8 = 2;

/// A morris board: point adjacency, the set of mills, and the symmetry group.
pub struct Board {
    pub n: usize,           // number of points
    pub men: u8,            // men per side
    pub adj: Vec<u32>,      // adjacency bitmask per point
    pub mills: Vec<u32>,    // each mill as a 3-bit mask
    pub sym: Vec<Vec<u8>>,  // symmetry group: point permutations (length n each)
}

impl Board {
    /// Build a `rings`-ring morris board (2 = six men's, 3 = nine men's).
    pub fn rings(rings: usize, men: u8) -> Board {
        let n = 8 * rings;
        let mut adj = vec![0u32; n];

        // Within each ring: a corner/mid 8-cycle.
        for r in 0..rings {
            let base = r * 8;
            for i in 0..8 {
                let a = base + i;
                let b = base + ((i + 1) % 8);
                adj[a] |= 1 << b;
                adj[b] |= 1 << a;
            }
        }
        // Spokes between consecutive rings, at the side-midpoints (1,3,5,7).
        for r in 0..rings.saturating_sub(1) {
            for &m in &[1usize, 3, 5, 7] {
                let a = r * 8 + m;
                let b = (r + 1) * 8 + m;
                adj[a] |= 1 << b;
                adj[b] |= 1 << a;
            }
        }

        // Mills: the four sides of each ring, plus spokes spanning three rings.
        let mut mills = Vec::new();
        for r in 0..rings {
            let base = r * 8;
            for s in 0..4 {
                let p0 = base + (2 * s);
                let p1 = base + (2 * s + 1);
                let p2 = base + ((2 * s + 2) % 8);
                mills.push((1u32 << p0) | (1 << p1) | (1 << p2));
            }
        }
        for r in 0..rings.saturating_sub(2) {
            for &m in &[1usize, 3, 5, 7] {
                mills.push((1u32 << (r * 8 + m)) | (1 << ((r + 1) * 8 + m)) | (1 << ((r + 2) * 8 + m)));
            }
        }

        let sym = build_symmetry(rings, n, &adj, &mills);
        Board { n, men, adj, mills, sym }
    }

    pub fn six_mens() -> Board {
        Board::rings(2, 6)
    }

    pub fn nine_mens() -> Board {
        Board::rings(3, 9)
    }
}

/// Apply a point permutation to a bitmask.
#[inline]
fn permute(perm: &[u8], mask: u32) -> u32 {
    let mut out = 0u32;
    let mut m = mask;
    while m != 0 {
        let p = m.trailing_zeros() as usize;
        out |= 1 << perm[p];
        m &= m - 1;
    }
    out
}

/// Is `perm` a board automorphism (preserves adjacency and the mill set)?
fn is_automorphism(perm: &[u8], adj: &[u32], mills: &[u32]) -> bool {
    let n = perm.len();
    for a in 0..n {
        // adjacency of a must map onto adjacency of perm[a]
        if permute(perm, adj[a]) != adj[perm[a] as usize] {
            return false;
        }
    }
    let mill_set: HashSet<u32> = mills.iter().copied().collect();
    mills.iter().all(|&m| mill_set.contains(&permute(perm, m)))
}

/// Generate the board symmetry group as the closure of three generators:
/// quarter-turn rotation, reflection, and ring reversal (inside-out). Validated
/// to be 16 genuine automorphisms.
fn build_symmetry(rings: usize, n: usize, adj: &[u32], mills: &[u32]) -> Vec<Vec<u8>> {
    let identity: Vec<u8> = (0..n as u8).collect();

    // rotate a quarter turn: within each ring i -> (i+2) mod 8
    let mut rot = vec![0u8; n];
    // reflect: within each ring i -> (8 - i) mod 8
    let mut refl = vec![0u8; n];
    // ring reversal: ring r -> ring (rings-1-r), same in-ring position
    let mut ringrev = vec![0u8; n];
    for r in 0..rings {
        let base = r * 8;
        for i in 0..8 {
            rot[base + i] = (base + ((i + 2) % 8)) as u8;
            refl[base + i] = (base + ((8 - i) % 8)) as u8;
            ringrev[base + i] = ((rings - 1 - r) * 8 + i) as u8;
        }
    }
    let generators = [rot, refl, ringrev];
    for g in &generators {
        assert!(is_automorphism(g, adj, mills), "generator is not a board automorphism");
    }

    // Closure under composition (perm a then b => (b∘a)[x] = b[a[x]]).
    let compose = |a: &[u8], b: &[u8]| -> Vec<u8> { a.iter().map(|&x| b[x as usize]).collect() };
    let mut group: HashSet<Vec<u8>> = HashSet::new();
    group.insert(identity.clone());
    let mut frontier = vec![identity];
    while let Some(p) = frontier.pop() {
        for g in &generators {
            let q = compose(&p, g);
            if group.insert(q.clone()) {
                frontier.push(q);
            }
        }
    }
    let group: Vec<Vec<u8>> = group.into_iter().collect();
    assert_eq!(group.len(), 16, "expected a 16-element symmetry group");
    for g in &group {
        debug_assert!(is_automorphism(g, adj, mills));
    }
    group
}

/// A morris position. The side to move is encoded in `turn`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct State {
    pub white: u32,
    pub black: u32,
    pub w_hand: u8,
    pub b_hand: u8,
    pub turn: u8,
}

/// A generic morris game over a [`Board`], with a precomputed dense index.
pub struct GenericMorris {
    pub board: Board,
    binom: Binom,
    // Dense index: positions are grouped into buckets keyed by
    // (turn, w_hand, nw, b_hand, nb); within a bucket they are ranked by the
    // combinatorial number system. Buckets are stored in ascending-offset order.
    buckets: Vec<Bucket>,
    bucket_of: Vec<u32>, // key code -> bucket id (u32::MAX if absent)
    total: u64,
}

#[derive(Clone, Copy)]
struct Bucket {
    offset: u64,
    turn: u8,
    w_hand: u8,
    nw: u8,
    b_hand: u8,
    nb: u8,
    black_radix: u64, // C(n - nw, nb)
}

impl GenericMorris {
    pub fn new(board: Board) -> Self {
        let binom = Binom::new();
        let men = board.men as usize;
        let n = board.n;
        let radix = men + 1;
        let mut buckets = Vec::new();
        let mut bucket_of = vec![u32::MAX; 2 * radix * radix * radix * radix];
        let mut offset = 0u64;
        for turn in 0..2u8 {
            for w_hand in 0..=board.men {
                for nw in 0..=(board.men - w_hand) {
                    for b_hand in 0..=board.men {
                        for nb in 0..=(board.men - b_hand) {
                            if nw as usize + nb as usize > n {
                                continue;
                            }
                            let white_ways = binom.c(n, nw as usize);
                            let black_radix = binom.c(n - nw as usize, nb as usize);
                            let size = white_ways * black_radix;
                            if size == 0 {
                                continue;
                            }
                            let code = key_code(radix, turn, w_hand, nw, b_hand, nb);
                            bucket_of[code] = buckets.len() as u32;
                            buckets.push(Bucket {
                                offset,
                                turn,
                                w_hand,
                                nw,
                                b_hand,
                                nb,
                                black_radix,
                            });
                            offset += size;
                        }
                    }
                }
            }
        }
        GenericMorris { board, binom, buckets, bucket_of, total: offset }
    }

    pub fn six_mens() -> Self {
        GenericMorris::new(Board::six_mens())
    }

    pub fn nine_mens() -> Self {
        GenericMorris::new(Board::nine_mens())
    }

    /// The canonical representative of a position under the 16-fold board
    /// symmetry (hands and turn are symmetry-invariant). Chooses the
    /// lexicographically least `(white, black)` over the orbit.
    pub fn canon(&self, s: &State) -> State {
        let mut best_w = s.white;
        let mut best_b = s.black;
        for perm in &self.board.sym {
            let w = permute(perm, s.white);
            let b = permute(perm, s.black);
            if (w, b) < (best_w, best_b) {
                best_w = w;
                best_b = b;
            }
        }
        State { white: best_w, black: best_b, ..*s }
    }

    // --- forward rules (mirror of super::morris, generalised to u32 / Board) ---

    #[inline]
    fn completes_mill(&self, mask: u32, point: usize) -> bool {
        self.board
            .mills
            .iter()
            .any(|&m| (m >> point) & 1 == 1 && mask & m == m)
    }

    fn removable(&self, opp: u32) -> u32 {
        let mut free = 0u32;
        let mut m = opp;
        while m != 0 {
            let p = m.trailing_zeros() as usize;
            if !self.completes_mill(opp, p) {
                free |= 1 << p;
            }
            m &= m - 1;
        }
        if free == 0 {
            opp
        } else {
            free
        }
    }

    #[inline]
    fn has_slide(&self, stm: u32, occupied: u32) -> bool {
        let mut m = stm;
        while m != 0 {
            let f = m.trailing_zeros() as usize;
            if self.board.adj[f] & !occupied != 0 {
                return true;
            }
            m &= m - 1;
        }
        false
    }

    fn after(&self, s: &State, mover: u32, mover_hand: u8, opp: u32) -> State {
        if s.turn == WHITE {
            State { white: mover, black: opp, w_hand: mover_hand, b_hand: s.b_hand, turn: BLACK }
        } else {
            State { white: opp, black: mover, w_hand: s.w_hand, b_hand: mover_hand, turn: WHITE }
        }
    }
}

#[inline]
fn key_code(radix: usize, turn: u8, w_hand: u8, nw: u8, b_hand: u8, nb: u8) -> usize {
    ((((turn as usize) * radix + w_hand as usize) * radix + nw as usize) * radix + b_hand as usize)
        * radix
        + nb as usize
}

impl RulesGame for GenericMorris {
    type State = State;

    fn start(&self) -> State {
        State { white: 0, black: 0, w_hand: self.board.men, b_hand: self.board.men, turn: WHITE }
    }

    fn successors(&self, s: &State) -> Vec<State> {
        let n = self.board.n;
        let (stm, opp, stm_hand) = if s.turn == WHITE {
            (s.white, s.black, s.w_hand)
        } else {
            (s.black, s.white, s.b_hand)
        };
        let occ = s.white | s.black;
        let mut out = Vec::new();

        // Base moves: (resulting mover mask, destination point).
        let mut bases: Vec<(u32, usize)> = Vec::new();
        if stm_hand > 0 {
            for p in 0..n {
                if (occ >> p) & 1 == 0 {
                    bases.push((stm | (1 << p), p));
                }
            }
        } else {
            let mut m = stm;
            while m != 0 {
                let f = m.trailing_zeros() as usize;
                m &= m - 1;
                let adj = self.board.adj[f];
                let mut t_mask = adj & !occ;
                while t_mask != 0 {
                    let t = t_mask.trailing_zeros() as usize;
                    t_mask &= t_mask - 1;
                    bases.push(((stm & !(1 << f)) | (1 << t), t));
                }
            }
        }

        for (mover, dest) in bases {
            let new_hand = if stm_hand > 0 { stm_hand - 1 } else { 0 };
            if self.completes_mill(mover, dest) && opp != 0 {
                let rem = self.removable(opp);
                let mut q_mask = rem;
                while q_mask != 0 {
                    let q = q_mask.trailing_zeros() as usize;
                    q_mask &= q_mask - 1;
                    out.push(self.after(s, mover, new_hand, opp & !(1 << q)));
                }
            } else {
                out.push(self.after(s, mover, new_hand, opp));
            }
        }
        out
    }

    fn terminal(&self, s: &State) -> Option<Outcome> {
        if s.w_hand == 0 && s.b_hand == 0 {
            let stm = if s.turn == WHITE { s.white } else { s.black };
            if stm.count_ones() < 3 {
                return Some(Outcome::Loss);
            }
            if !self.has_slide(stm, s.white | s.black) {
                return Some(Outcome::Loss);
            }
        }
        None
    }
}

impl Game for GenericMorris {
    type State = State;

    fn num_states(&self) -> u64 {
        self.total
    }

    fn index(&self, s: &State) -> u64 {
        let nw = s.white.count_ones() as u8;
        let nb = s.black.count_ones() as u8;
        let turn = if s.turn == WHITE { 0 } else { 1 };
        let radix = self.board.men as usize + 1;
        let code = key_code(radix, turn, s.w_hand, nw, s.b_hand, nb);
        let bid = self.bucket_of[code];
        debug_assert!(bid != u32::MAX, "no bucket for state {s:?}");
        let bk = self.buckets[bid as usize];
        let pts = mask_to_points(s.white);
        let rw = rank_subset(&self.binom, &pts);
        let rb = rank_black_in_empties(&self.binom, s.white, s.black);
        bk.offset + rw * bk.black_radix + rb
    }

    fn from_index(&self, i: u64) -> Option<State> {
        // Largest bucket offset <= i (buckets are in ascending-offset order).
        let bid = match self.buckets.binary_search_by(|b| b.offset.cmp(&i)) {
            Ok(k) => k,
            Err(k) => k - 1, // i falls inside bucket k-1
        };
        let bk = self.buckets[bid];
        let within = i - bk.offset;
        let rw = within / bk.black_radix;
        let rb = within % bk.black_radix;
        let n = self.board.n;
        let white_pts = unrank_subset(&self.binom, rw, n, bk.nw as usize);
        let white = white_pts.iter().fold(0u32, |m, &p| m | (1 << p));
        let black = unrank_black_in_empties(&self.binom, white, rb, n, bk.nb as usize);
        let turn = if bk.turn == 0 { WHITE } else { BLACK };
        Some(State { white, black, w_hand: bk.w_hand, b_hand: bk.b_hand, turn })
    }

    fn start(&self) -> State {
        RulesGame::start(self)
    }

    fn successors(&self, s: &State) -> Vec<State> {
        RulesGame::successors(self, s)
    }

    fn terminal(&self, s: &State) -> Option<Outcome> {
        RulesGame::terminal(self, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_mens_board_matches_handcoded() {
        // The ring-built six men's board must reproduce super::morris's tables.
        let b = Board::six_mens();
        let expected_adj: [u32; 16] = [
            130, 517, 10, 2068, 40, 8272, 160, 32833, 33280, 1282, 2560, 5128, 10240, 20512,
            40960, 16768,
        ];
        assert_eq!(b.adj, expected_adj.to_vec());
        let mut mills = b.mills.clone();
        mills.sort();
        let mut expected = vec![7u32, 28, 112, 193, 1792, 7168, 28672, 49408];
        expected.sort();
        assert_eq!(mills, expected);
    }

    #[test]
    fn nine_mens_board_shape() {
        let b = Board::nine_mens();
        assert_eq!(b.n, 24);
        assert_eq!(b.mills.len(), 16); // 12 ring sides + 4 spokes
        assert_eq!(b.sym.len(), 16);
        // every spoke mill spans the three rings at one mid
        assert!(b.mills.contains(&((1 << 1) | (1 << 9) | (1 << 17))));
    }

    #[test]
    fn index_round_trips_on_start_and_a_few_states() {
        let g = GenericMorris::six_mens();
        let s = RulesGame::start(&g);
        let i = g.index(&s);
        assert_eq!(Game::from_index(&g, i).unwrap(), s);

        // a handful of successors round-trip too
        for ns in RulesGame::successors(&g, &s) {
            let j = g.index(&ns);
            assert_eq!(Game::from_index(&g, j).unwrap(), ns, "round trip failed for {ns:?}");
        }
    }

    #[test]
    fn canon_is_idempotent_and_orbit_invariant() {
        let g = GenericMorris::six_mens();
        let s = State { white: (1 << 0) | (1 << 1), black: 1 << 8, w_hand: 4, b_hand: 5, turn: WHITE };
        let c = g.canon(&s);
        assert_eq!(g.canon(&c), c, "canon not idempotent");
        // every symmetric image canonicalises to the same representative
        for perm in &g.board.sym {
            let img = State { white: permute(perm, s.white), black: permute(perm, s.black), ..s };
            assert_eq!(g.canon(&img), c, "orbit member canonicalised differently");
        }
    }
}
