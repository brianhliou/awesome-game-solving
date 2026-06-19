//! Thin typed wrapper over the WASM engine. The browser does no morris rules of
//! its own — legal moves and perfect-play values come from the validated Rust
//! engine; this module only reshapes the flat `u32` wire format and derives a
//! human-readable description of each move by diffing positions.

import init, { Explorer } from "game-solver-wasm";
// Let Vite serve the .wasm as an asset (works in dev and build); the bindings'
// default import.meta.url resolution doesn't reach node_modules under dev.
import wasmUrl from "game-solver-wasm/game_solver_wasm_bg.wasm?url";

export const WHITE = 1;
export const BLACK = 2;

/** A morris position. `turn` is WHITE or BLACK. */
export interface Pos {
  white: number;
  black: number;
  wHand: number;
  bHand: number;
  turn: number;
}

/** 0 win, 1 loss, 2 draw, 3 unknown (from the relevant side's perspective). */
export type Val = 0 | 1 | 2 | 3;

export interface Move {
  from: number | null; // source point, or null for a placement
  to: number; // destination point
  captured: number | null; // opponent point removed, or null
  value: Val; // worth to the mover
  result: Pos;
}

let explorer: Explorer | null = null;

export async function loadEngine(rings: number, men: number, tablebaseUrl: string): Promise<void> {
  await init({ module_or_path: wasmUrl });
  explorer = new Explorer(rings, men);
  const bytes = await fetchGunzip(tablebaseUrl);
  explorer.set_tablebase(bytes);
}

async function fetchGunzip(url: string): Promise<Uint8Array> {
  const resp = await fetch(url);
  if (!resp.ok || !resp.body) throw new Error(`failed to fetch ${url}: ${resp.status}`);
  const stream = resp.body.pipeThrough(new DecompressionStream("gzip"));
  const buf = await new Response(stream).arrayBuffer();
  return new Uint8Array(buf);
}

export function startPos(): Pos {
  const a = explorer!.start();
  return { white: a[0], black: a[1], wHand: a[2], bHand: a[3], turn: a[4] };
}

export function positionValue(p: Pos): Val {
  return explorer!.value(p.white, p.black, p.wHand, p.bHand, p.turn) as Val;
}

/** Terminal value for the side to move (0/1/2), or 3 if the position is not terminal. */
export function terminalValue(p: Pos): Val {
  return explorer!.terminal(p.white, p.black, p.wHand, p.bHand, p.turn) as Val;
}

export function legalMoves(p: Pos): Move[] {
  const flat = explorer!.legal_moves(p.white, p.black, p.wHand, p.bHand, p.turn);
  const moves: Move[] = [];
  for (let i = 0; i < flat.length; i += 6) {
    const result: Pos = {
      white: flat[i],
      black: flat[i + 1],
      wHand: flat[i + 2],
      bHand: flat[i + 3],
      turn: flat[i + 4],
    };
    moves.push(describeMove(p, result, flat[i + 5] as Val));
  }
  return moves;
}

/** Index of the single set bit in `mask` (mask is known to have exactly one). */
function bitIndex(mask: number): number {
  let i = 0;
  while (i < 32 && ((mask >>> i) & 1) === 0) i++;
  return i;
}

function describeMove(before: Pos, after: Pos, value: Val): Move {
  const moverWhite = before.turn === WHITE;
  const moverBefore = moverWhite ? before.white : before.black;
  const moverAfter = moverWhite ? after.white : after.black;
  const oppBefore = moverWhite ? before.black : before.white;
  const oppAfter = moverWhite ? after.black : after.white;

  const added = moverAfter & ~moverBefore; // destination
  const removed = moverBefore & ~moverAfter; // source (0 for a placement)
  const capturedMask = oppBefore & ~oppAfter; // 0 if nothing captured

  return {
    to: bitIndex(added),
    from: removed ? bitIndex(removed) : null,
    captured: capturedMask ? bitIndex(capturedMask) : null,
    value,
    result: after,
  };
}
