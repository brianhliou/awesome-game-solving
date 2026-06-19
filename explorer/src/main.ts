import "./style.css";
import { SIX_MENS } from "./board";
import {
  loadEngine,
  startPos,
  legalMoves,
  positionValue,
  terminalValue,
  WHITE,
  type Pos,
  type Val,
  type Move,
} from "./engine";

const SVGNS = "http://www.w3.org/2000/svg";
const layout = SIX_MENS;

// --- DOM ---
const svg = document.getElementById("board") as unknown as SVGSVGElement;
const evalEl = document.getElementById("eval")!;
const hintEl = document.getElementById("hint")!;
const undoBtn = document.getElementById("undo") as HTMLButtonElement;
const resetBtn = document.getElementById("reset") as HTMLButtonElement;
const hintsBox = document.getElementById("hints") as HTMLInputElement;

// --- state ---
let current: Pos;
let history: Pos[] = [];
let selectedSource: number | null = null;
let pendingCapture: Move[] | null = null;

const sideName = (turn: number) => (turn === WHITE ? "White" : "Black");
const moverHand = (p: Pos) => (p.turn === WHITE ? p.wHand : p.bHand);
const valClass = (v: Val) => ["win", "loss", "draw", "unknown"][v];
const valRank = (v: Val) => [2, 0, 1, -1][v]; // win > draw > loss

function bestValueTo(moves: Move[]): Val {
  return moves.reduce<Val>((b, m) => (valRank(m.value) > valRank(b) ? m.value : b), 1 as Val);
}

function pushTo<K>(map: Map<K, Move[]>, key: K, m: Move) {
  const arr = map.get(key);
  if (arr) arr.push(m);
  else map.set(key, [m]);
}

function el(tag: string, attrs: Record<string, string | number>): SVGElement {
  const e = document.createElementNS(SVGNS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  return e;
}

function occupant(p: Pos, point: number): 0 | 1 | 2 {
  if ((p.white >>> point) & 1) return WHITE;
  if ((p.black >>> point) & 1) return 2;
  return 0;
}

interface Affordance {
  role: "place" | "source" | "dest" | "capture" | "selected";
  value: Val;
}

/** Which points are clickable right now, and how to colour them. */
function affordances(moves: Move[]): Map<number, Affordance> {
  const a = new Map<number, Affordance>();
  if (pendingCapture) {
    for (const m of pendingCapture) {
      if (m.captured != null) a.set(m.captured, { role: "capture", value: m.value });
    }
    return a;
  }
  if (moverHand(current) > 0) {
    // placement: every empty target
    const byTarget = new Map<number, Move[]>();
    for (const m of moves) {
      if (m.from === null) pushTo(byTarget, m.to, m);
    }
    for (const [to, ms] of byTarget) a.set(to, { role: "place", value: bestValueTo(ms) });
    return a;
  }
  // movement
  if (selectedSource === null) {
    const bySource = new Map<number, Move[]>();
    for (const m of moves) {
      if (m.from !== null) pushTo(bySource, m.from, m);
    }
    for (const [from, ms] of bySource) a.set(from, { role: "source", value: bestValueTo(ms) });
  } else {
    a.set(selectedSource, { role: "selected", value: 3 });
    const dests = moves.filter((m) => m.from === selectedSource);
    const byDest = new Map<number, Move[]>();
    for (const m of dests) pushTo(byDest, m.to, m);
    for (const [to, ms] of byDest) a.set(to, { role: "dest", value: bestValueTo(ms) });
    // still allow picking a different man
    for (const m of moves) {
      if (m.from !== null && m.from !== selectedSource && !a.has(m.from)) {
        a.set(m.from, { role: "source", value: 3 });
      }
    }
  }
  return a;
}

function render() {
  const term = terminalValue(current);
  const moves = term === 3 ? legalMoves(current) : [];
  const showHints = hintsBox.checked;
  const aff = affordances(moves);

  // rebuild svg
  while (svg.firstChild) svg.removeChild(svg.firstChild);

  // edges
  for (const [a1, b1] of layout.edges) {
    const [x1, y1] = layout.points[a1];
    const [x2, y2] = layout.points[b1];
    svg.appendChild(el("line", { x1, y1, x2, y2, class: "edge" }));
  }

  // points + stones + affordances
  for (let p = 0; p < layout.points.length; p++) {
    const [x, y] = layout.points[p];
    const g = el("g", { class: "pt", "data-p": p });
    g.appendChild(el("circle", { cx: x, cy: y, r: 0.09, class: "node" }));

    const af = aff.get(p);
    if (af) {
      const cls = `aff ${af.role} ${showHints ? valClass(af.value) : "neutral"}`;
      g.appendChild(el("circle", { cx: x, cy: y, r: 0.4, class: cls }));
    }

    const occ = occupant(current, p);
    if (occ) {
      g.appendChild(el("circle", { cx: x, cy: y, r: 0.33, class: occ === WHITE ? "stone white" : "stone black" }));
    }

    if (af) {
      (g as SVGElement).addEventListener("click", () => onClick(p));
      g.classList.add("clickable");
    }
    svg.appendChild(g);
  }

  // eval + hint
  const wOn = popcount(current.white), bOn = popcount(current.black);
  let evalText: string;
  if (term !== 3) {
    evalText = `Game over — ${sideName(current.turn)} ${term === 1 ? "is lost" : term === 2 ? "is drawn" : "has won"}.`;
  } else {
    const v = positionValue(current);
    const word = ["is winning", "is losing", "draws", "is unknown"][v];
    evalText = `${sideName(current.turn)} to move ${word} with perfect play.`;
  }
  evalEl.innerHTML =
    `<span class="verdict ${term !== 3 ? valClass(term) : valClass(positionValue(current))}">${evalText}</span>` +
    `<span class="counts">White ${wOn} on board, ${current.wHand} in hand · Black ${bOn} on board, ${current.bHand} in hand</span>`;

  hintEl.textContent = hintText(term, moves);
  undoBtn.disabled = history.length === 0;
}

function hintText(term: Val, moves: Move[]): string {
  if (term !== 3) return "Press Reset to play again.";
  if (pendingCapture) return "You formed a mill — click an opponent's man to remove it.";
  if (moverHand(current) > 0) return `${sideName(current.turn)} to place — click an empty point.`;
  if (selectedSource === null) return moves.length ? "Click one of your men to move it." : "No legal move.";
  return "Click a destination (or another of your men).";
}

function onClick(point: number) {
  const moves = legalMoves(current);

  if (pendingCapture) {
    const m = pendingCapture.find((x) => x.captured === point);
    if (m) applyMove(m);
    return;
  }

  if (moverHand(current) > 0) {
    resolve(moves.filter((m) => m.from === null && m.to === point));
    return;
  }

  // movement phase
  const isOwnMovable = moves.some((m) => m.from === point);
  if (selectedSource === null) {
    if (isOwnMovable) {
      selectedSource = point;
      render();
    }
    return;
  }
  if (point === selectedSource) {
    selectedSource = null;
    render();
    return;
  }
  if (isOwnMovable) {
    selectedSource = point; // reselect a different man
    render();
    return;
  }
  resolve(moves.filter((m) => m.from === selectedSource && m.to === point));
}

/** Apply a unique move, or open a capture choice when several share a destination. */
function resolve(candidates: Move[]) {
  if (candidates.length === 0) return;
  if (candidates.length === 1) {
    applyMove(candidates[0]);
  } else {
    pendingCapture = candidates;
    render();
  }
}

function applyMove(m: Move) {
  history.push(current);
  current = m.result;
  selectedSource = null;
  pendingCapture = null;
  render();
}

function popcount(x: number): number {
  let n = 0;
  while (x) {
    x &= x - 1;
    n++;
  }
  return n;
}

undoBtn.addEventListener("click", () => {
  const prev = history.pop();
  if (prev) {
    current = prev;
    selectedSource = null;
    pendingCapture = null;
    render();
  }
});
resetBtn.addEventListener("click", () => {
  current = startPos();
  history = [];
  selectedSource = null;
  pendingCapture = null;
  render();
});
hintsBox.addEventListener("change", render);

async function main() {
  // Gzip payload under an opaque name so the server doesn't set Content-Encoding
  // (which would make the browser auto-decompress and collide with our own).
  const url = `${import.meta.env.BASE_URL}morris6.tb`;
  try {
    await loadEngine(2, 6, url);
    current = startPos();
    render();
  } catch (e) {
    console.error("[explorer] load failed", e);
    evalEl.textContent = `Failed to load the solver: ${(e as Error).message}`;
  }
}

main();
