# Awesome Game Solving [![Awesome](https://awesome.re/badge.svg)](https://awesome.re)

> Strongly and weakly solved games, endgame tablebases, retrograde analysis, and the tools to probe them.

Most "awesome" game lists collect engines that play well. This one collects games that have been *solved*: perfect play known for every position, computed and stored instead of searched at runtime.

The techniques travel further than the games suggest. A checkers proof, a 7-piece chess tablebase, and a complete solution to a tiny shogi variant all run on one idea: enumerate the positions, work backward from the terminal ones, and propagate the result. This list collects the solved games, the tablebase formats that store them, the retrograde-analysis literature behind them, and the public APIs and explorers you can query today. [Dōbutsu shōgi](#worked-example-dōbutsu-shōgi) is the worked example, solved end to end.

## Contents

- [What "solved" means](#what-solved-means)
- [Solved games](#solved-games)
- [Endgame tablebases](#endgame-tablebases)
- [Retrograde analysis](#retrograde-analysis)
- [Probe APIs and explorers](#probe-apis-and-explorers)
- [Worked example: Dōbutsu shōgi](#worked-example-dōbutsu-shōgi)
- [Contributing](#contributing)

## What "solved" means

Three strengths, from weakest to strongest ([Wikipedia](https://en.wikipedia.org/wiki/Solved_game)):

- **Ultra-weakly solved** — the game-theoretic value of the starting position is known (who wins, or draw), but not how. Hex is ultra-weakly solved for all board sizes by a strategy-stealing argument that names no moves.
- **Weakly solved** — the value *and* a strategy to achieve it from the start are known. Checkers is weakly solved.
- **Strongly solved** — the value is known for every reachable position, not just the start. This is what a tablebase gives you.

## Solved games

| Game | Result under perfect play | Strength | Year | Source |
|---|---|---|---|---|
| Tic-tac-toe | Draw | Strong | — | trivial |
| Connect Four | First player wins | Weak | 1988 | [Allen](https://tromp.github.io/c4/c4.html), Allis |
| Qubic (4×4×4 tic-tac-toe) | First player wins | Weak | 1980 | Patashnik |
| Nine Men's Morris | Draw | Strong | 1993 | [Gasser](https://en.wikipedia.org/wiki/Nine_men%27s_morris#Mathematics) |
| Gomoku (free-style, 15×15) | First player wins | Weak | 1993 | [Allis](https://en.wikipedia.org/wiki/Gomoku#Standard) |
| Awari / Oware | Draw | Strong | 2002 | [Romein & Bal](https://en.wikipedia.org/wiki/Oware#Solving) |
| Checkers (English draughts) | Draw | Weak | 2007 | [Schaeffer / Chinook](https://webdocs.cs.ualberta.ca/~chinook/) |
| Pentago | First player wins | Strong | 2014 | [Irving](https://perfect-pentago.net/) |
| Antichess / Losing chess | First player wins (1.e3) | Weak | 2016 | [Watkins](https://en.wikipedia.org/wiki/Losing_chess#Computer_analysis) |
| Othello / Reversi (8×8) | Draw | Weak | 2023 | [Takizawa, arXiv:2310.19387](https://arxiv.org/abs/2310.19387) |
| Dōbutsu shōgi | Second player (gote) wins | Strong | 2009 | [Tanaka](https://en.wikipedia.org/wiki/D%C5%8Dbutsu_sh%C5%8Dgi) · [worked example below](#worked-example-dōbutsu-shōgi) |
| Hex (all 9×9 openings) | First player wins | Weak | 2013 | [Pawlewicz & Hayward](https://webdocs.cs.ualberta.ca/~hayward/hex/) |

Hex is also *ultra-weakly* solved for every board size: the first player wins by strategy stealing ([Nash, 1952](https://en.wikipedia.org/wiki/Hex_(board_game))), with no explicit strategy. On 10×10 only a single opening has been solved so far.

## Endgame tablebases

Precomputed perfect play for positions with few pieces. Chess is the deep end; the formats differ in what they store and how they compress it.

- [Endgame tablebase (Wikipedia)](https://en.wikipedia.org/wiki/Endgame_tablebase) — the history and the metrics (DTM, DTZ, WDL).
- [Endgame Tablebases (Chessprogramming wiki)](https://www.chessprogramming.org/Endgame_Tablebases) — the engineering reference.
- **[Syzygy](https://www.chessprogramming.org/Syzygy_Bases)** (Ronald de Man, 2013) — WDL + DTZ50, 7 pieces, the compact modern default (~18 GB for 6-piece, ~17 TB for 7-piece). What most engines and Lichess use.
- **Gaviota** (Miguel Ballicora) — distance-to-mate, ignores the 50-move rule. Good for analysis that wants the shortest forced mate.
- **Nalimov** (Eugene Nalimov, ~2000) — distance-to-mate, 6 pieces, large (~1.2 TB for 6-piece). The format everything before Syzygy used.
- **Lomonosov** (Moscow State University, 2012) — full 7-piece, distance-to-mate, ~140 TB.
- **8-piece** — in progress; Lichess hosts a [partial 8-piece tablebase](https://lichess.org/@/Lichess/blog/op1-partial-8-piece-tablebase-available/1ptPBDpC).

Variant and small-game tablebases:

- [clausecker/dobutsu](https://github.com/clausecker/dobutsu) — C tablebase and engine for Dōbutsu shōgi.
- Lichess hosts variant tablebases for antichess (4-piece) and atomic (6-piece) alongside standard.

## Retrograde analysis

The algorithm under every tablebase: start from positions with a known result (mate, capture, terminal), then walk backward, marking predecessors until the value stops changing.

- [Retrograde Analysis (Chessprogramming wiki)](https://www.chessprogramming.org/Retrograde_Analysis) — the algorithm, variants, and references.
- **Bellman, 1965** — proposed using a database and backward induction to solve chess and checkers endgames.
- **Ströhlein, 1970** — first implementation, in a doctoral thesis; solved KQK, KRK, KPK, and other small endgames.
- **Thompson, 1986** — *Retrograde Analysis of Certain Endgames*; the KQKR work that beat a grandmaster from the database.
- **[Schaeffer et al., 2007](https://www.science.org/doi/10.1126/science.1144079)** — *Checkers Is Solved* (Science). Retrograde endgame DBs meeting a forward proof tree.
- **[Endgame Analysis of Dou Shou Qi, 2016](https://arxiv.org/abs/1604.07312)** — retrograde analysis applied to the Jungle-game family, a close cousin of Dōbutsu shōgi.

## Probe APIs and explorers

Query a solved position right now, no local tables required.

- **[Lichess tablebase API](https://github.com/lichess-org/lila-tablebase)** — `GET https://tablebase.lichess.ovh/standard?fen=...` returns WDL/DTZ for standard (7-piece), antichess, and atomic. Rate-limited; be polite.
- **[syzygy-tables.info](https://syzygy-tables.info)** ([source](https://github.com/niklasf/syzygy-tables.info)) — browser UI and public API over the 7-piece Syzygy bases.
- **[python-chess Syzygy probing](https://python-chess.readthedocs.io/en/latest/syzygy.html)** — read Syzygy bases directly from Python.
- **[Fathom](https://github.com/jdart1/Fathom)** — standalone C library for probing Syzygy from an engine.

## Worked example: Dōbutsu shōgi

Dōbutsu shōgi ("Let's Catch the Lion!") is a 3×4 children's shogi variant, and a complete strong solution of it shows every layer of this list at once. [brianhliou/dobutsu-shogi](https://github.com/brianhliou/dobutsu-shogi) builds the whole pipeline from scratch:

- **Retrograde analysis** over all 246,803,167 reachable positions, in Rust, verified against the [clausecker](https://github.com/clausecker/dobutsu) reference (50k positions, zero mismatches).
- **A tablebase** stored as a 333 MB `.ctb` file via a minimal perfect hash.
- **A probe API** over the solved positions.
- **[A live explorer](https://dobutsu.brianhliou.com)** in the style of the Lichess opening explorer, walking the second player's forced win.
- **[A long-form write-up](https://brianhliou.com/posts/dobutsu-shogi/)** working from the primary sources to the result: gote wins, in 78 moves from the start.

A small board, solved completely, with the retrograde → tablebase → probe → explorer path you can read end to end.

## Contributing

Add a game only if its result is established in a citable source (paper, thesis, or a solver with published verification). Link the primary source, not a summary of it. One entry per row; keep the result and strength columns honest (ultra-weak / weak / strong). Pull requests welcome.

## License

[![CC0](https://licensebuttons.net/p/zero/1.0/88x31.png)](https://creativecommons.org/publicdomain/zero/1.0/)

To the extent possible under law, the contributors have waived all copyright and related rights to this work.
