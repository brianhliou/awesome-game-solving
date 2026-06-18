# Contributing

Thanks for helping build the record of solved games.

## What belongs here

A game earns an entry when its result under perfect play is established in a
citable source: a peer-reviewed paper, a thesis, or a solver with published
verification. "Widely believed" results wait until someone has actually computed
or proven them.

Unsolved games belong only on the frontier list, with the barrier that keeps
them open.

## How to add or correct an entry

1. Add or edit the game's record in `data/games/<id>.yaml` (see
   `data/schema.md` for the fields).
2. Link the primary source, not a summary of it. Wikipedia works as a pointer,
   but `sources` should reach the actual paper, thesis, or solver where one
   exists.
3. Keep `strength` honest: ultra-weak, weak, and strong are different claims
   (see the README).
4. One game per file. Variants with different results (board sizes, rule sets)
   get their own records.

## Verification

Each entry carries a `verified` flag. A maintainer flips it to `true` once the
result, year, and citation have been checked against the primary source. New
entries arrive `verified: false`; that's expected.

Open a pull request. A disagreement about a result is settled by the source,
not the argument.
