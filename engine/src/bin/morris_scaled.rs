//! Validate the scaled solver path (dense combinatorial index + memory-frugal
//! retrograde) before it is ever pointed at the nine men's board.
//!
//! The six men's *result* is already triple-confirmed by `solve_reachable`
//! (bitmask, array, and now the generic-u32 rules all agree on 42,372,745 → a
//! draw), so here we validate the two genuinely new pieces and the new solver:
//!
//!   [1] the generic u32 rules reproduce the known six men's result;
//!   [2] the dense index is a bijection on that reachable set;
//!   [3] the dense fixpoint solver agrees with the forward-only oracle on every
//!       reachable position — checked on the smaller, equally-loopy four and five
//!       men's boards, where it finishes in minutes (the six men's full index is
//!       205M slots and the round-based sweep over it takes hours; the slice
//!       solver, not this baseline, is what scales to nine men's).
//!
//! Run: `cargo run --release --bin morris_scaled`.

use game_solver::dense_solve::solve_dense;
use game_solver::games::generic_morris::Board;
use game_solver::games::GenericMorris;
use game_solver::{solve_reachable, Game, Outcome, RulesGame};
use std::time::Instant;

fn main() {
    let six = GenericMorris::six_mens();
    let nine = GenericMorris::nine_mens();
    println!("index space sizes (full, before symmetry/slicing):");
    println!("  six men's : {:>15}", Game::num_states(&six));
    println!("  nine men's: {:>15}   <- too loose for a flat sweep; needs slices", Game::num_states(&nine));

    // ---- [1] generic rules reproduce the trusted six men's result ----
    println!("\n[1] oracle — solve_reachable on generic six men's");
    let t = Instant::now();
    let oracle = solve_reachable(&six);
    let owin = oracle.count(Outcome::Win) as u64;
    let oloss = oracle.count(Outcome::Loss) as u64;
    let odraw = oracle.count(Outcome::Draw) as u64;
    let start = RulesGame::start(&six);
    println!(
        "    reachable {}  (win {}, loss {}, draw {})  start = {:?}   [{:.1?}]",
        oracle.len(), owin, oloss, odraw, oracle.get(&start).unwrap(), t.elapsed()
    );
    assert_eq!(oracle.len(), 42_372_745, "generic rules disagree with the known six men's count");
    assert_eq!(oracle.get(&start), Some(Outcome::Draw));

    // ---- [2] dense index is a bijection on the reachable set ----
    println!("\n[2] dense index — bijection check on the reachable set");
    let total = Game::num_states(&six) as usize;
    let mut seen = vec![0u64; total.div_ceil(64)];
    let (mut collisions, mut rt_fail, mut max_idx) = (0u64, 0u64, 0u64);
    for (s, _) in oracle.iter() {
        let idx = Game::index(&six, s);
        max_idx = max_idx.max(idx);
        if Game::from_index(&six, idx).as_ref() != Some(s) {
            rt_fail += 1;
        }
        let (w, b) = ((idx >> 6) as usize, idx & 63);
        if (seen[w] >> b) & 1 == 1 { collisions += 1; } else { seen[w] |= 1 << b; }
    }
    let distinct: u64 = seen.iter().map(|x| x.count_ones() as u64).sum();
    println!(
        "    reachable {}  distinct indices {}  collisions {}  round-trip failures {}  max_index {}",
        oracle.len(), distinct, collisions, rt_fail, max_idx
    );
    assert_eq!(collisions, 0, "index is NOT injective on the reachable set");
    assert_eq!(rt_fail, 0, "index/from_index is NOT a clean round trip");
    assert_eq!(distinct as usize, oracle.len());
    drop(oracle);
    drop(seen);

    // ---- [3] dense fixpoint solver == oracle, on smaller loopy boards ----
    println!("\n[3] dense fixpoint solver vs forward-only oracle (smaller morris boards)");
    for men in [4u8, 5] {
        dense_vs_oracle(men);
    }

    println!("\nALL CHECKS PASS: generic rules, dense index, and dense solver are validated.");
    println!("The six men's result (a draw over 42,372,745 positions) is reproduced by the scaled rules.");
}

/// Solve `men`-men's morris on the 16-point board two ways and require agreement.
fn dense_vs_oracle(men: u8) {
    let g = GenericMorris::new(Board::rings(2, men));
    let label = format!("{men} men's (16-pt board)");
    let total = Game::num_states(&g);

    let t = Instant::now();
    let oracle = solve_reachable(&g);
    let (owin, oloss, odraw) = (
        oracle.count(Outcome::Win) as u64,
        oracle.count(Outcome::Loss) as u64,
        oracle.count(Outcome::Draw) as u64,
    );
    let start = RulesGame::start(&g);
    let oracle_start = oracle.get(&start).unwrap();
    println!(
        "  {label}: index {total}, reachable {}  (oracle: win {owin}, loss {oloss}, draw {odraw})  [{:.1?}]",
        oracle.len(), t.elapsed()
    );

    let t = Instant::now();
    let sol = solve_dense(&g, |_, _| {});
    let (mut rw, mut rl, mut rd, mut mism) = (0u64, 0u64, 0u64, 0u64);
    for (s, o) in oracle.iter() {
        let v = sol.value_at(Game::index(&g, s));
        if v != o { mism += 1; }
        match v { Outcome::Win => rw += 1, Outcome::Loss => rl += 1, Outcome::Draw => rd += 1 }
    }
    println!(
        "    dense: rounds {}, on reachable win {rw} loss {rl} draw {rd}  mismatches {mism}  start {:?}  [{:.1?}]",
        sol.rounds, sol.value_at(Game::index(&g, &start)), t.elapsed()
    );
    assert_eq!(mism, 0, "{label}: dense solver disagrees with the oracle");
    assert_eq!((rw, rl, rd), (owin, oloss, odraw), "{label}: WLD split differs");
    assert_eq!(sol.value_at(Game::index(&g, &start)), oracle_start, "{label}: start value differs");
}
