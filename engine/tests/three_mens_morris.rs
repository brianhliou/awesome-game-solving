//! Locks in our strong solution of three men's morris (Tapatan ruleset).
//! Computed by the engine; cross-checks the classical "first player wins by the
//! centre" result and records the full opening classification.

use game_solver::games::three_mens_morris::Pos;
use game_solver::games::ThreeMensMorris;
use game_solver::{solve, solve_reachable, Game, Outcome};

const BLACK: u8 = 2;

fn after_white_opening(game: &ThreeMensMorris, point: usize) -> Pos {
    let mut p = game.start();
    p.cells[point] = 1; // WHITE
    p.w_hand -= 1;
    p.turn = BLACK;
    p
}

#[test]
fn first_player_wins() {
    let game = ThreeMensMorris;
    let tb = solve(&game);
    assert_eq!(tb.value(game.index(&game.start())), Outcome::Win);
}

#[test]
fn reachable_solver_agrees_with_dense_solver() {
    // The index-free reachable solver must reproduce the dense solver's verdict.
    let game = ThreeMensMorris;
    let dense = solve(&game);
    let reachable = solve_reachable(&game);
    assert_eq!(reachable.get(&game.start()), Some(Outcome::Win));
    assert_eq!(reachable.get(&game.start()), Some(dense.value(game.index(&game.start()))));
}

#[test]
fn centre_is_the_unique_winning_opening() {
    let game = ThreeMensMorris;
    let tb = solve(&game);
    // After White's opening it is Black to move: Loss => White wins, Win => White loses.
    let value_for_black = |point| tb.value(game.index(&after_white_opening(&game, point)));

    assert_eq!(value_for_black(4), Outcome::Loss, "centre should win for White");
    for corner in [0, 2, 6, 8] {
        assert_eq!(value_for_black(corner), Outcome::Draw, "corner {corner} should draw");
    }
    for edge in [1, 3, 5, 7] {
        assert_eq!(value_for_black(edge), Outcome::Win, "edge {edge} should lose for White");
    }
}
