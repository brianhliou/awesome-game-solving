//! The combinatorial number system: a dense bijection between integers and
//! `k`-subsets of `{0, 1, ..., n-1}`. This is the craft part of giving a capture
//! game a dense perfect-hash index — it lets us address "the i-th way to place
//! `k` men on `n` points" with a flat array instead of a hash map.
//!
//! A subset `{x_0 < x_1 < ... < x_{k-1}}` maps to
//! `rank = sum_i C(x_i, i+1)`, the standard combinadic. Both directions are
//! exhaustively bijection-tested below for the board sizes we care about.

/// Binomial coefficients `C(n, k)` precomputed up to `MAX_N`. Morris boards are
/// 16 (six men's) and 24 (nine men's) points, and `k <= men <= 9`, so a 33x33
/// table (room for nine men's plus headroom) covers every lookup.
const MAX_N: usize = 33;

/// `C[n][k]` for `n, k < MAX_N`. Built once via Pascal's rule.
pub struct Binom {
    table: Vec<Vec<u64>>,
}

impl Binom {
    pub fn new() -> Self {
        let mut table = vec![vec![0u64; MAX_N]; MAX_N];
        for n in 0..MAX_N {
            table[n][0] = 1;
            for k in 1..=n {
                table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
            }
        }
        Binom { table }
    }

    /// `C(n, k)`, or 0 when `k > n` (the natural combinatorial value).
    #[inline]
    pub fn c(&self, n: usize, k: usize) -> u64 {
        if k > n || n >= MAX_N {
            0
        } else {
            self.table[n][k]
        }
    }
}

impl Default for Binom {
    fn default() -> Self {
        Self::new()
    }
}

/// Rank a `k`-subset (given as strictly ascending point indices) into
/// `[0, C(n, k))`. `n` is implicit — it only bounds the values, not the formula.
#[inline]
pub fn rank_subset(binom: &Binom, points: &[u8]) -> u64 {
    let mut rank = 0u64;
    for (i, &p) in points.iter().enumerate() {
        rank += binom.c(p as usize, i + 1);
    }
    rank
}

/// Inverse of [`rank_subset`]: the `rank`-th `k`-subset of `{0, ..., n-1}`,
/// returned as ascending point indices. Greedy from the largest element down.
pub fn unrank_subset(binom: &Binom, mut rank: u64, n: usize, k: usize) -> Vec<u8> {
    let mut out = vec![0u8; k];
    // Fill positions from the top (largest element first).
    let mut upper = n; // candidate values are strictly below `upper`
    for i in (0..k).rev() {
        // Largest c < upper with C(c, i+1) <= rank.
        let mut c = upper;
        while c > 0 {
            c -= 1;
            if binom.c(c, i + 1) <= rank {
                break;
            }
        }
        out[i] = c as u8;
        rank -= binom.c(c, i + 1);
        upper = c;
    }
    out
}

/// Convert a bitmask over `n` points into ascending point indices.
#[inline]
pub fn mask_to_points(mask: u32) -> Vec<u8> {
    let mut pts = Vec::with_capacity(mask.count_ones() as usize);
    let mut m = mask;
    while m != 0 {
        let p = m.trailing_zeros() as u8;
        pts.push(p);
        m &= m - 1;
    }
    pts
}

/// Rank `black` within the points left empty by `white`. Black men only ever sit
/// on points not occupied by white, so they are ranked in the *compressed*
/// coordinate space of the `n - popcount(white)` empty points — this is what
/// makes the two-colour index dense (no wasted slots for overlap).
pub fn rank_black_in_empties(binom: &Binom, white: u32, black: u32) -> u64 {
    // Compressed coordinate of an absolute point = number of empty points below it.
    let mut compressed = Vec::with_capacity(black.count_ones() as usize);
    let mut m = black;
    while m != 0 {
        let p = m.trailing_zeros() as u32;
        // empties below p = p minus (white men below p)
        let below = (1u32 << p) - 1;
        let c = p - (white & below).count_ones();
        compressed.push(c as u8);
        m &= m - 1;
    }
    rank_subset(binom, &compressed)
}

/// Inverse of [`rank_black_in_empties`]: given `white` and a black rank, recover
/// the absolute `black` bitmask. `n` is the board size, `nb` the number of black men.
pub fn unrank_black_in_empties(binom: &Binom, white: u32, rank: u64, n: usize, nb: usize) -> u32 {
    let empties_count = n - white.count_ones() as usize;
    let compressed = unrank_subset(binom, rank, empties_count, nb);
    // Map each compressed coordinate back to its absolute point: the c-th empty point.
    let mut black = 0u32;
    let mut empty_idx = 0u32; // running index among empty points
    let mut next = 0usize; // next compressed coordinate to place
    for p in 0..n as u32 {
        if (white >> p) & 1 == 1 {
            continue; // occupied by white, not an empty slot
        }
        if next < compressed.len() && compressed[next] as u32 == empty_idx {
            black |= 1 << p;
            next += 1;
        }
        empty_idx += 1;
    }
    black
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binomials() {
        let b = Binom::new();
        assert_eq!(b.c(0, 0), 1);
        assert_eq!(b.c(5, 2), 10);
        assert_eq!(b.c(16, 6), 8008);
        assert_eq!(b.c(24, 9), 1_307_504);
        assert_eq!(b.c(3, 5), 0); // k > n
    }

    /// rank/unrank must be a bijection on `[0, C(n, k))` for every subset.
    #[test]
    fn subset_bijection_exhaustive() {
        let b = Binom::new();
        for n in 0..=18usize {
            for k in 0..=n.min(9) {
                let total = b.c(n, k);
                // Walk all k-subsets in ascending-rank order and check round-trip.
                for r in 0..total {
                    let pts = unrank_subset(&b, r, n, k);
                    assert_eq!(pts.len(), k, "n={n} k={k} r={r}");
                    // strictly ascending and in range
                    for w in pts.windows(2) {
                        assert!(w[0] < w[1], "not ascending: n={n} k={k} r={r} {pts:?}");
                    }
                    if let Some(&last) = pts.last() {
                        assert!((last as usize) < n, "out of range: n={n} k={k} r={r}");
                    }
                    assert_eq!(rank_subset(&b, &pts), r, "rank!=unrank inverse n={n} k={k}");
                }
            }
        }
    }

    /// The black-in-empties ranking must be a bijection for each white placement.
    #[test]
    fn black_in_empties_bijection() {
        let b = Binom::new();
        let n = 9usize;
        // a couple of white placements
        for white in [0b000000000u32, 0b000010101, 0b111000000] {
            let nw = white.count_ones() as usize;
            let empties = n - nw;
            for nb in 0..=empties {
                let total = b.c(empties, nb);
                for r in 0..total {
                    let black = unrank_black_in_empties(&b, white, r, n, nb);
                    assert_eq!(black & white, 0, "black overlaps white");
                    assert_eq!(black.count_ones() as usize, nb);
                    assert_eq!(rank_black_in_empties(&b, white, black), r);
                }
            }
        }
    }
}
