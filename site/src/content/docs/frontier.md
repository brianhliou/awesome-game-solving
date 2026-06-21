---
title: Open frontier
description: Not just what's solved — what this project's engine could solve next, and what's out of reach and why. The forward-looking half of the reference.
---

Most "solved games" resources are backward-looking: a list of what's done. This
is the forward half — where the open-source [engine](https://github.com/brianhliou/solved-games/tree/main/engine)
behind this site could plausibly produce a *new* result, and what stays out of
reach, and why.

## Active target — Y, side 7

This project strongly solved Y through side 6 (2,173,243,128 positions). **Side 7
(28 cells) is the next rung — and the first that outgrows the current solver.**

| | |
|---|---|
| Status | Open (solved here through side 6) |
| Size | ~10¹²–10¹³ positions (estimated) |
| Barrier | The dense index exceeds 2³² — needs 64-bit indexing and a value table far larger than RAM |
| Proposed method | Disk-paged (external-memory) slice retrograde — the extension the engine is already aimed at |
| Why it matters | Extends the first verified strong solution of Y one rung, and proves the slice solver scales past memory |

The full ladder is on the [Y family page](/families/y/).

## On the radar

:::caution[Targets, not claims]
These candidates are under evaluation — each needs its citations verified before
it becomes a full frontier entry. Only the Y target above is confirmed.
:::

Candidates where this engine could plausibly produce a new result, in rough
priority:

- **Y** — the standout; no published *constructive* solution at any board size, and a native fit for retrograde. (Active, above.)
- **Kōnane, narrow boards** — 3×*n* / 4×*n* / 5×*n* are named open problems in the literature, within in-core reach; a retrograde solve is methodologically distinct from the existing combinatorial-game-theory attack.
- **Twelve men's morris** — the same 24-point board family as the [morris work](/families/morris/), the natural "next morris," pending a board-equivalence check against Morabaraba and the disk-paged slice extension.

## Out of reach — and why

The engine is a **retrograde** (backward-from-terminal) solver, which is a hard
filter. Two walls account for most unsolved games:

- **Wrong algorithm.** Games whose terminal positions are still full, complex boards give retrograde nothing to anchor on backward — Hex (10×10, 11×11), Havannah, TwixT, Lines of Action, Quoridor. These need *forward* proof-number search; imperfect-information games (e.g. dark chess) need counterfactual-regret methods instead.
- **Scale or structure.** Some are many orders beyond the engine's ~10¹⁰–10¹¹ in-core ceiling, or break acyclic retrograde with cycles or non-monotone piece counts — Go (ko-cycles), minishogi (drops), international draughts, 8-piece chess, Amazons, Arimaa. A mature external-memory slice solver would bring a few of these (Breakthrough, draughts and chess endgames) back within reach.

---

*The full per-target frontier track — one entry each, with verified citations and
cost models — is in progress.*
