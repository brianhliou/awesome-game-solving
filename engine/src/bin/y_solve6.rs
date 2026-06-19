//! Strongly solve Y at a larger side (default 6) with the slice-by-slice solver,
//! which holds only two adjacent ply-slices in memory. Reports the per-ply
//! breakdown, the win/loss totals over the tight index, and the start value.
//! Usage: `y_solve6 [side]`.

use game_solver::games::y::Y;
use game_solver::{Game, Outcome};

fn main() {
    let n: usize = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(6);
    let game = Y::new(n);
    eprintln!(
        "Y side-{n} ({} cells): {} positions in the tight index. Solving by slices...",
        game.cells,
        game.num_states()
    );

    let sol = game.solve_sliced(|m, size| {
        eprintln!("  slice m={m:>2}: {size:>13} positions");
    });

    println!("\nY side-{n}:");
    println!("  positions (tight index): {}", sol.total);
    println!("  wins {}, losses {}", sol.wins, sol.losses);
    println!("  START value (first player to move): {:?}", sol.start);
    assert_eq!(sol.start, Outcome::Win, "the first player should win every Y board");
    println!("  ✓ first player wins; no draws (Y theorem holds under the slice solve)");
}
