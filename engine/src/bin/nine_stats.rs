//! Scope the nine men's morris solve with exact combinatorial numbers, so the
//! build-vs-bank decision rests on figures rather than estimates.
//!
//! The movement phase (both hands empty) decomposes into slices keyed by
//! (white-count, black-count). Each slice's size is exact: C(24, nw)·C(24-nw, nb)
//! boards, times two for the side to move. The symmetry fold (16-fold board
//! group) is shown as an ~/16 estimate (the exact fold via Burnside is slightly
//! larger because symmetric positions are undercounted, but within a few %).
//!
//! Run: `cargo run --release --bin nine_stats`.

use game_solver::index::Binom;

fn fmt(n: u128) -> String {
    // thousands separators
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn gib(bytes: u128) -> String {
    format!("{:.2} GiB", bytes as f64 / (1u128 << 30) as f64)
}

fn main() {
    let b = Binom::new();
    let n = 24usize;

    // ---- movement phase: exact slice sizes ----
    let mut slices: Vec<(usize, usize, u128)> = Vec::new();
    let mut mv_all = 0u128; // nw,nb in 0..=9
    let mut mv_work = 0u128; // nw,nb in 3..=9 (the stored, non-terminal region)
    for nw in 0..=9 {
        for nb in 0..=9 {
            if nw + nb > n {
                continue;
            }
            let size = b.c(n, nw) as u128 * b.c(n - nw, nb) as u128;
            slices.push((nw, nb, size));
            mv_all += size;
            if nw >= 3 && nb >= 3 {
                mv_work += size;
            }
        }
    }
    slices.sort_by(|a, c| c.2.cmp(&a.2));

    println!("=== nine men's morris — movement phase (both hands empty) ===\n");
    println!("boards counted once (one side to move):");
    println!("  all slices nw,nb in 0..9 : {}", fmt(mv_all));
    println!("  stored region nw,nb 3..9 : {}", fmt(mv_work));
    println!("\nstates (x2 for side to move):");
    println!("  stored region, unfolded  : {}", fmt(mv_work * 2));
    println!("  stored region, /16 fold ~: {}", fmt(mv_work * 2 / 16));

    println!("\nlargest movement slices (boards, one side to move):");
    println!("   slice        boards        states(x2)     ~folded(/16 of states)");
    for (nw, nb, sz) in slices.iter().take(12) {
        println!(
            "   {nw}v{nb}   {:>14}   {:>14}   {:>12}",
            fmt(*sz),
            fmt(sz * 2),
            fmt(sz * 2 / 16)
        );
    }

    // ---- peak RAM for the hardest slice (solve it without folding) ----
    let (lnw, lnb, lsz) = slices[0];
    let largest_states = lsz * 2;
    println!("\npeak RAM to solve the hardest slice ({lnw}v{lnb}) without symmetry folding:");
    println!("  value array  (1 byte/state): {}", gib(largest_states));
    println!("  + counter    (1 byte/state): {}", gib(largest_states));
    println!("  => ~{} working set (fits a 32 GiB box; <1 GiB if folded)", gib(largest_states * 2));

    // ---- stored tablebase size (whole movement DB) ----
    let mv_states = mv_work * 2;
    println!("\nstored movement tablebase (2 bits/state):");
    println!("  unfolded : {}", gib(mv_states / 4));
    println!("  /16 fold~: {}", gib(mv_states / 4 / 16));

    // ---- placement phase: loose index upper bound ----
    // The full dense index (everything) minus the movement-phase index. This is a
    // loose UPPER BOUND on reachable placement states (it includes parity-violating
    // hand combinations that never occur); the real reachable placement set is much
    // smaller because placements strictly alternate.
    let six = game_solver::games::GenericMorris::nine_mens();
    let full_index = game_solver::Game::num_states(&six) as u128;
    let mv_index = mv_all * 2; // movement buckets in the full index
    println!("\n=== placement phase (hands not yet empty) ===");
    println!("  full dense index (all phases): {}", fmt(full_index));
    println!("  movement buckets in it       : {}", fmt(mv_index));
    println!("  placement buckets (loose UB) : {}", fmt(full_index - mv_index));
    println!("  (real reachable placement is far smaller — placements alternate,");
    println!("   so most (w_hand,b_hand) combinations in this bound never occur.)");

    // ---- compute estimate ----
    let edges = mv_states * 14; // ~14 successors/state, order of magnitude
    let rate = 50_000_000u128; // edges/sec/core processed, conservative
    let core_secs = edges / rate;
    println!("\n=== rough compute (movement retrograde, O(edges)) ===");
    println!("  ~{} edges; at ~{}M edges/s/core => ~{} core-seconds", fmt(edges), rate / 1_000_000, fmt(core_secs));
    println!("  on 16 cores, unfolded: ~{} min   (folding cuts ~16x more)", core_secs / 16 / 60);
}
