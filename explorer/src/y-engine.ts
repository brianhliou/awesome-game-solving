//! Thin typed wrapper over the Y WASM engine. The browser does no Y rules of its
//! own: legal moves and perfect-play values come from the validated Rust engine.
//!
//! The strong solve runs in a Web Worker (see `y-worker.ts`) so the page never
//! freezes while it computes; the worker posts back the raw value table, and we
//! wrap it here with `YExplorer.from_solution` for synchronous probing on the
//! main thread. The whole table is recomputed live — nothing is precomputed or
//! shipped.

import init, { YExplorer, y_num_states } from "game-solver-wasm";
import wasmUrl from "game-solver-wasm/game_solver_wasm_bg.wasm?url";

/** 0 win, 1 loss, 2 draw, 3 unknown (from the relevant side's perspective). */
export type Val = 0 | 1 | 2 | 3;

/** A Y position: two stone bitmasks. Side to move is derived from the counts. */
export interface YPos {
  p1: number;
  p2: number;
}

export interface YMove {
  to: number; // the cell the stone is placed on
  value: Val; // worth to the mover
  result: YPos;
}

interface SolveResult {
  values: Uint8Array;
  ms: number;
}

let initialized = false;
let explorer: YExplorer | null = null;
let worker: Worker | null = null;
let reqSeq = 0;
let lastSolve = { count: 0, ms: 0 };
const pending = new Map<number, { resolve: (r: SolveResult) => void; reject: (e: Error) => void }>();

function ensureWorker(): Worker {
  if (worker) return worker;
  worker = new Worker(new URL("./y-worker.ts", import.meta.url), { type: "module" });
  worker.onmessage = (ev: MessageEvent) => {
    const d = ev.data as { id: number; values?: Uint8Array; ms?: number; error?: string };
    const p = pending.get(d.id);
    if (!p) return;
    pending.delete(d.id);
    if (d.error) p.reject(new Error(d.error));
    else p.resolve({ values: d.values!, ms: d.ms! });
  };
  return worker;
}

function solveInWorker(side: number): Promise<SolveResult> {
  const w = ensureWorker();
  const id = ++reqSeq;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    w.postMessage({ id, side });
  });
}

/**
 * Build (and strongly solve, in a worker) the side-`n` board. `onStart` fires
 * once the position count is known but before the solve completes, so the UI can
 * show "solving N positions…" with a live timer.
 */
export async function selectSide(n: number, onStart?: (count: number) => void): Promise<void> {
  if (!initialized) {
    await init({ module_or_path: wasmUrl });
    initialized = true;
  }
  if (onStart) onStart(y_num_states(n));
  const { values, ms } = await solveInWorker(n);
  if (explorer) {
    explorer.free();
    explorer = null;
  }
  explorer = YExplorer.from_solution(n, values);
  lastSolve = { count: values.length, ms };
}

/** Stats from the most recent solve: positions resolved and wall-clock ms. */
export function solveInfo(): { count: number; ms: number } {
  return lastSolve;
}

export function side(): number {
  return explorer!.side();
}

export function startPos(): YPos {
  const a = explorer!.start();
  return { p1: a[0], p2: a[1] };
}

export function positionValue(p: YPos): Val {
  return explorer!.value(p.p1, p.p2) as Val;
}

/** Terminal value for the side to move (0/1/2), or 3 if not terminal. */
export function terminalValue(p: YPos): Val {
  return explorer!.terminal(p.p1, p.p2) as Val;
}

export function legalMoves(p: YPos): YMove[] {
  const flat = explorer!.legal_moves(p.p1, p.p2);
  const before = (p.p1 | p.p2) >>> 0;
  const out: YMove[] = [];
  for (let i = 0; i < flat.length; i += 3) {
    const result: YPos = { p1: flat[i], p2: flat[i + 1] };
    const placed = ((result.p1 | result.p2) >>> 0) & ~before;
    out.push({ to: bitIndex(placed), value: flat[i + 2] as Val, result });
  }
  return out;
}

function bitIndex(mask: number): number {
  let i = 0;
  while (i < 32 && ((mask >>> i) & 1) === 0) i++;
  return i;
}
