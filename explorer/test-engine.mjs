// Headless check of the WASM + tablebase pipeline (no browser): probe the start,
// confirm it's a draw, and sanity-check the legal-move values.
import { readFile } from "node:fs/promises";
import init, { Explorer } from "game-solver-wasm";

const wasmBytes = await readFile(
  new URL("../wasm/pkg/game_solver_wasm_bg.wasm", import.meta.url),
);
await init({ module_or_path: wasmBytes });

const ex = new Explorer(2, 6);
const tb = await readFile(new URL("../engine/artifacts/morris6.wld", import.meta.url));
ex.set_tablebase(new Uint8Array(tb));
console.log("tablebase loaded:", ex.has_tablebase(), "num_states:", ex.num_states());

const start = ex.start();
console.log("start [w,b,wHand,bHand,turn]:", Array.from(start));

const v = ex.value(start[0], start[1], start[2], start[3], start[4]);
const W = ["win", "loss", "draw", "unknown"];
console.log("start value:", W[v]);

const flat = ex.legal_moves(start[0], start[1], start[2], start[3], start[4]);
const n = flat.length / 6;
const vals = [];
for (let i = 0; i < flat.length; i += 6) vals.push(W[flat[i + 5]]);
console.log(`start moves: ${n}`, vals.join(","));

// Descend one move and confirm the child value is consistent (opponent's view).
const m0 = { w: flat[0], b: flat[1], wh: flat[2], bh: flat[3], t: flat[4] };
console.log("after first placement, opponent value:", W[ex.value(m0.w, m0.b, m0.wh, m0.bh, m0.t)]);

if (W[v] !== "draw") {
  console.error("FAIL: start should be a draw");
  process.exit(1);
}
console.log("OK: start is a draw, pipeline works.");
