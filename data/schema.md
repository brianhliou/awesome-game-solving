# Record schema

One YAML file per solved game, at `games/<id>.yaml`.

| Field | Required | Notes |
|---|---|---|
| `id` | yes | kebab-case, matches the filename |
| `name` | yes | display name, with diacritics |
| `also_known_as` | no | list of alternate names |
| `family` | yes | abstract, chess-variant, shogi-variant, connection, go, mancala, combinatorial, imperfect-info, … |
| `board` | no | size or shape (e.g. 3×4, 8×8) |
| `result` | yes | `first-player-win` \| `second-player-win` \| `draw` |
| `result_detail` | no | human-readable, e.g. "Gote (second player) wins" |
| `strength` | yes | `ultra-weak` \| `weak` \| `strong` |
| `method` | yes | list: retrograde-analysis, proof-number-search, alpha-beta+db, knowledge-based, … |
| `distance_to_result` | no | e.g. "78 plies from the start" |
| `complexity.state_space` | no | integer; say whether reachable or total in `notes` |
| `complexity.game_tree` | no | integer or null |
| `year` | yes | year of the solution |
| `solved_by` | yes | person or team |
| `verified` | yes | bool; true once checked against the primary source |
| `sources` | yes | list of `{title, url}`; primary source first |
| `resources` | no | `{tablebase, code, explorer, writeup}`, each a list of `{title, url}` |
| `notes` | no | anything that doesn't fit a field |

Keep `verified: false` until a maintainer confirms result + year + citation.
