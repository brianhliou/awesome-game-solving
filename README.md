# Solved Games

> Strongly and weakly solved games, endgame tablebases, retrograde analysis, and the tools to probe them.

Most "awesome" game lists collect engines that play well. This one collects games that have been *solved*: perfect play known for every position, computed and stored instead of searched at runtime.

The techniques travel further than the games suggest. A checkers proof, a 7-piece chess tablebase, and a complete solution to a tiny shogi variant all run on one idea: enumerate the positions, work backward from the terminal ones, and propagate the result. This list collects the solved games, the tablebase formats that store them, the retrograde-analysis literature behind them, and the public APIs and explorers you can query today. [Dōbutsu shōgi](#worked-example-dōbutsu-shōgi) is the worked example, solved end to end.

**▶ [Play the perfect-play explorer](https://brianhliou.github.io/solved-games/explorer/)** — we strongly solved [six men's morris](data/games/six-mens-morris.yaml) (a draw over all 42,372,745 positions, a first published solution) with the [`game-solver`](engine/) engine in this repo, and built an interactive tablebase explorer for the family.

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

<!-- BEGIN GENERATED:solved-games — source of truth is data/games/*.yaml; regenerate with `npm run build`. Do not hand-edit this table. -->

### Alignment & *m,n,k* games

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Tic-tac-toe | Draw | Strong | Brute-force enumeration | — | [folklore](https://mathworld.wolfram.com/Tic-Tac-Toe.html) |
| Qubic | First player wins | Weak | Knowledge-based | 1980 | [Patashnik 1980](https://www.jstor.org/stable/2689613) |
| Gomoku (free-style) | First player wins | Weak | Threat-space search | 1993 | [Allis et al. 1993](https://aaai.org/papers/0001-go-moku-solved-by-new-search-techniques/) |
| Connect Four | First player wins | Strong | Brute-force enumeration | 1995 | [Tromp 1995](https://tromp.github.io/c4/connect4_thesis.pdf) |
| Teeko | Draw | Strong | Retrograde analysis | 1998 | [Steele 1998](https://boardgamegeek.com/thread/816476/steele-guy-november-23-1998-re-teeko-hakmem) |
| Renju (連珠) | First player wins | Weak | Threat-space search | 2001 | [Wágner et al. 2001](https://journals.sagepub.com/doi/abs/10.3233/ICG-2001-24104) |
| Pentago | First player wins | Strong | Retrograde analysis | 2014 | [Irving 2014](https://arxiv.org/abs/1404.0743) |
| Quarto! | Draw | Strong | Alpha-beta + DB | 2023 | [Goossens 2023](https://doi.org/10.5281/zenodo.20425801) |

### Morris / mill family

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Three men's morris | First player wins | Strong | Retrograde analysis | — | [folklore](https://en.wikipedia.org/wiki/Three_men's_morris) |
| Nine Men's Morris | Draw | Strong | Retrograde analysis | 1993 | [Gasser 1993](https://library.slmath.org/books/Book29/files/gasser.pdf) |
| Lasker Morris | Draw | Strong | Retrograde analysis | 2003 | [Stahlhacke 2003](https://althofer.de/stahlhacke-lasker-morris-2003.pdf) |
| Morabaraba | First player wins | Strong | Retrograde analysis | 2015 | [Gévay 2015](https://arxiv.org/abs/1408.0032) |
| Six men's morris | Draw | Strong | Retrograde analysis | 2026 | [Liou 2026](https://github.com/brianhliou/solved-games/tree/main/engine) |

### Connection games

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Hex (7×7) | First player wins | Weak | Strategy-stealing | 2003 | [Yang et al. 2003](https://webdocs.cs.ualberta.ca/~hayward/papers/solving7x7hex.pdf) |
| Hex (9×9, all openings) | First player wins | Weak | Depth-first PNS | 2013 | [Pawlewicz et al. 2013](https://webdocs.cs.ualberta.ca/~hayward/papers/pawlhayw.pdf) |

### Mancala

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Kalah | First player wins | Weak | Alpha-beta + DB | 2000 | [Irving et al. 2000](https://naml.us/paper/kalah/) |
| Awari (Oware) | Draw | Strong | Retrograde analysis | 2002 | [Romein et al. 2002](https://doi.org/10.1109/MC.2003.1236468) |

### Capture & board control

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Mū tōrere | Draw | Strong | Brute-force enumeration | 1987 | [Ascher 1987](https://www.tandfonline.com/doi/abs/10.1080/0025570X.1987.11977283) |
| Checkers (English draughts) | Draw | Weak | Retrograde analysis | 2007 | [Schaeffer et al. 2007](https://www.science.org/doi/10.1126/science.1144079) |
| Fanorona | Draw | Weak | Proof-number search | 2007 | [Schadd et al. 2007](https://dke.maastrichtuniversity.nl/m.winands/documents/Fanorona.pdf) |
| Othello | Draw | Weak | Alpha-beta + DB | 2023 | [Takizawa 2023](https://arxiv.org/abs/2310.19387) |

### Chess & chess variants

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Hexapawn | Second player wins | Strong | Brute-force enumeration | 1962 | [Gardner 1962](https://people.csail.mit.edu/brooks/idocs/GardnerHexapawn.pdf) |
| Gardner minichess | Draw | Weak | Knowledge-based | 2013 | [Mhalla et al. 2013](https://arxiv.org/abs/1307.7118) |
| Losing chess | First player wins | Weak | Proof-number search | 2016 | [Watkins 2016](https://content.iospress.com/articles/icga-journal/icg170017) |

### Hunt & unequal forces

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Maharajah and the Sepoys | Second player wins | Weak | Knowledge-based | 1892 | [Falkener 1892](https://archive.org/details/gamesancientorie00falkuoft) |
| Tigers and Goats | Draw | Weak | Retrograde analysis | 2007 | [Jin et al. 2007](https://library.slmath.org/books/Book56/files/22jin.pdf) |

### Combinatorial game theory

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Nim | First player wins | Strong | Mathematical proof | 1901 | [Bouton 1901](https://www.jstor.org/stable/1967631) |
| Wythoff's game | First player wins | Strong | Mathematical proof | 1907 | [Wythoff 1907](https://www.scirp.org/reference/referencespapers?referenceid=59121) |
| Sim | Second player wins | Strong | Brute-force enumeration | 1974 | [Mead et al. 1974](https://www.tandfonline.com/doi/abs/10.1080/0025570X.1974.11976415) |
| Pentominoes (two-player) | First player wins | Weak | Brute-force enumeration | 1996 | [Orman 1996](https://library.slmath.org/books/Book29/files/orman.pdf) |
| Dots and Boxes | First player wins | Weak | Alpha-beta + DB | 2014 | [Fraser 2014](https://groups.google.com/g/rec.games.abstract/c/-z5Y9JRlZJ8) |
| Amazons (small boards) | First player wins | Weak | Alpha-beta + DB | 2015 | [Song et al. 2015](https://webdocs.cs.ualberta.ca/~mmueller/ps/2014/2014-TCIAIG-amazons_solver-preprint.pdf) |
| Domineering | First player wins | Weak | Alpha-beta + DB | 2016 | [Uiterwijk 2016](https://arxiv.org/abs/1602.05404) |

### Shogi variants

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| [Dōbutsu shōgi](#worked-example-dōbutsu-shōgi) | Second player wins | Strong | Retrograde analysis | 2009 | [Tanaka 2009](https://ipsj.ixsq.nii.ac.jp/records/62415) · [worked example below](#worked-example-dōbutsu-shōgi) |

### Go

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Go (5×5) | First player wins | Weak | Alpha-beta + DB | 2002 | [van der Werf 2002](https://journals.sagepub.com/doi/10.3233/ICG-2003-26205) |

### Imperfect information

| Game | Result under perfect play | Strength | Method | Year | Source |
|---|---|---|---|---|---|
| Heads-up Limit Texas Hold'em (HULHE) | First player wins (ε-Nash) | Weak, approx. | Monte-Carlo CFR | 2015 | [Bowling et al. 2015](https://www.science.org/doi/10.1126/science.1259433) |
<!-- END GENERATED:solved-games -->

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
