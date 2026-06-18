//! Strongly solve six men's morris (no published prior solution) and report the
//! reachable state count and the game-theoretic value of the starting position.

use game_solver::games::morris::SixMensMorris;
use game_solver::reachable::solve_reachable_capped;
use game_solver::{Outcome, RulesGame};

fn main() {
    let game = SixMensMorris;
    let cap = 200_000_000;
    println!("solving six men's morris (16-point two-ring board, 6 men, no flying)…");
    match solve_reachable_capped(&game, cap) {
        None => println!(
            "  reachable states exceeded the {cap} cap — needs the tight index + scaled retrograde."
        ),
        Some(tb) => {
            let start = game.start();
            println!("  reachable states: {}", tb.len());
            println!(
                "  win {}, loss {}, draw {}",
                tb.count(Outcome::Win),
                tb.count(Outcome::Loss),
                tb.count(Outcome::Draw),
            );
            println!(
                "  START value (first player to move): {:?}",
                tb.get(&start).expect("start state solved"),
            );
        }
    }
}
