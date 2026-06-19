# Record schema

One YAML file per solved game, at `games/<id>.yaml`.

| Field | Required | Notes |
|---|---|---|
| `id` | yes | kebab-case, matches the filename |
| `name` | yes | display name, with diacritics |
| `also_known_as` | no | list of alternate names |
| `family` | yes | abstract, chess-variant, shogi-variant, connection, go, mancala, combinatorial, imperfect-info, … |
| `genre` | yes | grouping for the rendered tables; one of the slugs in **Genres** below. Distinct from `family` (a looser provenance tag) — `genre` is the display axis. |
| `board` | no | size or shape (e.g. 3×4, 8×8) |
| `result` | yes | `first-player-win` \| `second-player-win` \| `draw` |
| `result_detail` | no | human-readable, e.g. "Gote (second player) wins" |
| `strength` | yes | `ultra-weak` \| `weak` \| `strong` |
| `approximate` | no | bool; `true` when the solution is approximate rather than exact — e.g. an imperfect-information game solved to within an exploitability bound (an ε-Nash / "essentially weakly solved" result), not a discrete value |
| `epsilon` | no | the approximation bound when `approximate: true` (e.g. "0.986 mbb/g exploitability") |
| `method` | yes | list: retrograde-analysis, proof-number-search, alpha-beta+db, knowledge-based, monte-carlo-cfr, … |
| `distance_to_result` | no | e.g. "78 plies from the start" |
| `complexity.state_space` | no | integer; say whether reachable or total in `notes` |
| `complexity.game_tree` | no | integer or null |
| `year` | yes | year of the solution |
| `solved_by` | yes | person or team |
| `cite` | no | short label for the generated tables (e.g. "Tanaka 2009"); overrides the label auto-derived from `solved_by`. Use when `solved_by` is prose. |
| `verified` | yes | bool; true once checked against the primary source |
| `sources` | yes | list of `{title, url}`; primary source first |
| `resources` | no | `{tablebase, code, explorer, writeup}`, each a list of `{title, url}` |
| `notes` | no | anything that doesn't fit a field |

Keep `verified: false` until a maintainer confirms result + year + citation.

## Genres

The `genre` field groups games into the sections rendered in the README (and the
filter on the site). The slugs and their display order (familiar → frontier):

| Slug | Section title |
|---|---|
| `alignment` | Alignment & *m,n,k* games |
| `morris` | Morris / mill family |
| `connection` | Connection games |
| `mancala` | Mancala |
| `capture` | Capture & board control |
| `chess` | Chess & chess variants |
| `hunt` | Hunt & unequal forces |
| `cgt` | Combinatorial game theory |
| `shogi` | Shogi variants |
| `go` | Go |
| `imperfect-info` | Imperfect information |

Some games sit in two genres (e.g. Maharajah is a chess variant *and* an
unequal-forces hunt; Amazons is territorial *and* a canonical CGT game). Pick the
one that best places it for a reader; the choice is editorial, not load-bearing.
A record with an unrecognized or missing `genre` renders under a trailing
"Other" section and the build warns — so add the slug here when introducing one.

For **imperfect-information** games the ultra-weak / weak / strong ladder does not
map cleanly — position values beyond the start are not unique and outcomes are
real-valued (expected payoff) rather than discrete win/loss/draw. Record the
closest `strength` (usually `weak`), set `approximate: true` with an `epsilon`
bound, and use `result` to encode the sign of the game value. See
`heads-up-limit-holdem.yaml` for the worked example.
