//! Y (Schensted & Titus): the engine's connection-game check. Two invariants the
//! Y theorem guarantees — and that a wrong board model would violate — plus
//! regression anchors on the reachable counts.

use game_solver::dense_solve::solve_dense;
use game_solver::games::y::Y;
use game_solver::reachable::RulesGame;
use game_solver::{solve_reachable, Game, Outcome};

/// Strategy-stealing: the first player wins every Y board.
#[test]
fn first_player_wins_every_size() {
    for n in 1..=5 {
        let game = Y::new(n);
        let sol = solve_reachable(&game);
        assert_eq!(
            sol.get(&RulesGame::start(&game)),
            Some(Outcome::Win),
            "Y side-{n}: first player should win"
        );
    }
}

/// The Y theorem: a full board always has exactly one winner, and the two players
/// can never connect at once — so no reachable position is a draw. A draw here
/// would mean the board topology or win detection is wrong.
#[test]
fn no_reachable_draws() {
    for n in 1..=5 {
        let game = Y::new(n);
        let sol = solve_reachable(&game);
        assert_eq!(sol.count(Outcome::Draw), 0, "Y side-{n}: Y admits no draws");
    }
}

/// Regression anchors: reachable positions and the win/loss split per side,
/// computed by this engine. A change here means the board model changed.
#[test]
fn reachable_counts_are_stable() {
    // (side, reachable, wins, losses)
    let expected = [
        (1usize, 2usize, 1usize, 1usize),
        (2, 13, 7, 6),
        (3, 257, 163, 94),
        (4, 16_505, 10_630, 5_875),
        (5, 3_337_584, 2_155_091, 1_182_493),
    ];
    for (n, reachable, wins, losses) in expected {
        let game = Y::new(n);
        let sol = solve_reachable(&game);
        assert_eq!(sol.len(), reachable, "Y side-{n}: reachable count");
        assert_eq!(sol.count(Outcome::Win), wins, "Y side-{n}: win count");
        assert_eq!(sol.count(Outcome::Loss), losses, "Y side-{n}: loss count");
        assert_eq!(wins + losses, reachable, "Y side-{n}: win+loss must cover all (no draws)");
    }
}

/// The tight dense index must be a bijection on `[0, num_states())`.
#[test]
fn dense_index_is_a_bijection() {
    for n in 1..=4 {
        let game = Y::new(n);
        let num = game.num_states();
        for i in 0..num {
            let p = game.from_index(i).expect("every tight-index slot decodes");
            assert_eq!(game.index(&p), i, "Y side-{n}: index∘from_index must be identity");
        }
    }
}

/// The dense solver (over the tight index) and the reachable solver must agree on
/// the value of *every* reachable position — the cross-check that the index and
/// the dense solve are correct before trusting them on side-6.
#[test]
fn dense_and_reachable_agree() {
    let game = Y::new(4);
    let reach = solve_reachable(&game);
    let dense = solve_dense(&game, |_, _| {});
    assert_eq!(dense.count(Outcome::Draw), 0, "dense side-4: Y admits no draws");
    for (pos, want) in reach.iter() {
        let got = dense.value_at(game.index(pos));
        assert_eq!(got, want, "value disagreement at {pos:?}");
        // from_index∘index round-trips for reachable positions too.
        assert_eq!(game.from_index(game.index(pos)), Some(*pos));
    }
    assert_eq!(dense.value_at(game.index(&Game::start(&game))), Outcome::Win);
}

/// Side-5 through the dense path: the start is a first-player win and nothing draws.
#[test]
fn dense_side5_first_player_wins_no_draws() {
    let game = Y::new(5);
    let dense = solve_dense(&game, |_, _| {});
    assert_eq!(dense.value_at(Game::index(&game, &Game::start(&game))), Outcome::Win);
    assert_eq!(dense.count(Outcome::Draw), 0, "dense side-5: Y admits no draws");
}

/// The slice-by-slice solver must agree with the trusted dense solver on the full
/// index aggregate (every position is counted, win/loss totals match) and on the
/// start value — the validation that licenses running it on side-6.
#[test]
fn sliced_matches_dense_aggregate() {
    for n in 1..=5 {
        let game = Y::new(n);
        let dense = solve_dense(&game, |_, _| {});
        let sliced = game.solve_sliced(|_, _| {});
        assert_eq!(sliced.total, game.num_states(), "side-{n}: slices cover the whole index");
        assert_eq!(sliced.wins, dense.count(Outcome::Win), "side-{n}: win totals agree");
        assert_eq!(sliced.losses, dense.count(Outcome::Loss), "side-{n}: loss totals agree");
        assert_eq!(sliced.start, Outcome::Win, "side-{n}: first player wins");
    }
}
