# Vision

> The canonical reference for solved games: every known result, the method that produced it, and the code to verify it.

## Why this exists

Game-solving results are scattered. Wikipedia's "Solved game" page is a short table with no methods, no complexity figures, and few links to code or tablebases. The deep knowledge lives in places that don't talk to each other: the Chessprogramming wiki, HexWiki, individual arXiv papers, a researcher's personal page, a GitHub repo with no README. No single structured, citable record exists of what has been solved, how, and where to verify it.

This repo is that record.

## Scope

In scope: any game with a known game-theoretic result under perfect play, and the techniques used to get there. A game earns an entry when its result is established in a citable source (a paper, a thesis, or a solver with published verification).

For each solved game the record tracks the result (who wins, or draw), the solution strength (ultra-weak / weak / strong), the method, the state-space and game-tree complexity, the year, the solver, the primary source, and links to any tablebase, code, or explorer.

## Non-goals

This is not a general games catalog. BoardGameGeek and Wikipedia own that, and a new entry there gains nothing. Unsolved games appear only on the frontier list (what's open and why), never as catalog entries.

This is not a wiki. A wiki is a permanent moderation commitment, and a small team loses that fight to Wikipedia by default. Contributions come as pull requests against structured data, gated on a citation. The data files are the source of truth; the tables and the site generate from them.

## What it contains

- **The registry** — one structured record per solved game. Machine-readable first, so the tables, the site, and anyone's downstream tool all read from the same data.
- **The solvability ladder** — games ordered by state-space complexity, with the solved/unsolved frontier marked and the barrier named for each (storage, compute, branching factor).
- **The methods reference** — retrograde analysis, proof-number search, tablebase construction, perfect-hash compression, parallel in-core solving, each cross-linked to the games that used it.
- **Worked examples** — end-to-end solve walkthroughs. Dōbutsu shōgi is the flagship, with a live explorer over the full solution.
- **The frontier** — what's unsolved, what the next target is, and the barrier that holds it there.

## How it grows

- **v0** — the curated README: the list, the tablebase formats, the retrograde lineage, one worked example.
- **v1** — entries become structured data; the README tables and a GitHub Pages site generate from it. This is the part nobody else has.
- **v2 and beyond** — an interactive solvability ladder, a query API over the data, a maintained frontier tracker. Built when the v0/v1 record earns stars and contributions, not before.

## Principles

- **Citation-gated.** Every result links to a primary source. No entry on hearsay.
- **Structured data first.** The dataset is the product; the prose renders from it.
- **Honest strengths.** Ultra-weak, weak, and strong are different claims. Keep them distinct.
- **Depth over breadth.** One vertical covered completely beats a shallow catalog of everything.
