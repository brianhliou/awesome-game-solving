//! Off-main-thread Y solver. The dense solve for side 5 is a few million
//! positions and takes a couple of seconds; running it here keeps the page
//! responsive (no frozen tab) while it computes. The worker only *solves* — it
//! posts the raw value table back, and the main thread wraps it for probing.

import init, { solve_y } from "game-solver-wasm";
import wasmUrl from "game-solver-wasm/game_solver_wasm_bg.wasm?url";

interface SolveRequest {
  id: number;
  side: number;
}

let ready: Promise<unknown> | null = null;
const post = (msg: unknown, transfer?: Transferable[]) =>
  (self as unknown as Worker).postMessage(msg, transfer ?? []);

self.onmessage = (ev: MessageEvent<SolveRequest>) => {
  const { id, side } = ev.data;
  if (!ready) ready = init({ module_or_path: wasmUrl });
  ready
    .then(() => {
      const t0 = performance.now();
      const values = solve_y(side);
      const ms = performance.now() - t0;
      // Transfer the buffer rather than copy it across the boundary.
      post({ id, side, values, ms }, [values.buffer]);
    })
    .catch((e) => post({ id, side, error: String(e) }));
};
