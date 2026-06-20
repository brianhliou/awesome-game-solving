import "./y-style.css";
import { yLayout, viewBox, type YLayout } from "./y-board";
import {
  selectSide,
  solveInfo,
  startPos,
  legalMoves,
  positionValue,
  terminalValue,
  type YPos,
  type YMove,
  type Val,
} from "./y-engine";

const SVGNS = "http://www.w3.org/2000/svg";
const SIZES = [3, 4, 5];
const LABELS = "abcdefghijklmnopqrstu".split("");

const svg = document.getElementById("board") as unknown as SVGSVGElement;
const verdictEl = document.getElementById("verdict")!;
const countsEl = document.getElementById("counts")!;
const lineEl = document.getElementById("line")!;
const movesEl = document.getElementById("moves")!;
const undoBtn = document.getElementById("undo") as HTMLButtonElement;
const resetBtn = document.getElementById("reset") as HTMLButtonElement;
const sizesEl = document.getElementById("sizes")!;
const solvestatEl = document.getElementById("solvestat")!;

let n = 5;
let layout: YLayout = yLayout(n);
let path: YPos[] = [];
let ptr = 0;
let hotMove: YMove | null = null;
let view: { term: Val; moves: YMove[] } = { term: 3, moves: [] };
let solveToken = 0; // bumped per solve so a stale timer/result can bail out

const current = () => path[ptr];
const popcount = (x: number) => { let c = 0; x >>>= 0; while (x) { x &= x - 1; c++; } return c; };
const turnOf = (p: YPos) => (popcount(p.p1) === popcount(p.p2) ? 1 : 2);
const sideName = (t: number) => (t === 1 ? "Player 1" : "Player 2");
const valClass = (v: Val) => ["win", "loss", "draw", "unknown"][v];
const occupant = (p: YPos, c: number): 0 | 1 | 2 =>
  (p.p1 >>> c) & 1 ? 1 : (p.p2 >>> c) & 1 ? 2 : 0;

function el(tag: string, attrs: Record<string, string | number>): SVGElement {
  const e = document.createElementNS(SVGNS, tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, String(v));
  return e;
}

function renderBoard() {
  while (svg.firstChild) svg.removeChild(svg.firstChild);
  svg.setAttribute("viewBox", viewBox(layout));
  const R = 0.34;
  const cur = current();

  // The three sides, tinted — connecting all three with one group wins.
  for (const [cells, cls] of [
    [layout.sideA, "sideA"],
    [layout.sideB, "sideB"],
    [layout.sideC, "sideC"],
  ] as [number[], string][]) {
    svg.appendChild(
      el("polyline", { points: cells.map((c) => layout.points[c].join(",")).join(" "), class: `side ${cls}` }),
    );
  }

  // Grid edges (adjacency), stopping short of the node circles.
  for (const [a, b] of layout.edges) {
    const [ax, ay] = layout.points[a];
    const [bx, by] = layout.points[b];
    const dx = bx - ax, dy = by - ay, len = Math.hypot(dx, dy), ux = dx / len, uy = dy / len;
    svg.appendChild(el("line", { x1: ax + ux * R, y1: ay + uy * R, x2: bx - ux * R, y2: by - uy * R, class: "edge" }));
  }

  // Best-move value per empty cell (Y has no draws, so win or loss).
  const aff = new Map<number, Val>();
  for (const m of view.moves) aff.set(m.to, m.value);

  for (let c = 0; c < layout.cells; c++) {
    const [x, y] = layout.points[c];
    const g = el("g", {});
    const occ = occupant(cur, c);
    let cls = "node";
    if (aff.has(c)) cls += " " + valClass(aff.get(c)!);
    if (hotMove && hotMove.to === c) cls += " hot";
    g.appendChild(el("circle", { cx: x, cy: y, r: R, class: cls }));
    if (occ) {
      g.appendChild(el("circle", { cx: x, cy: y, r: 0.27, class: `stone p${occ}` }));
    } else {
      const t = el("text", { x, y, class: "lbl", "text-anchor": "middle", "dominant-baseline": "central" });
      t.textContent = LABELS[c];
      g.appendChild(t);
    }
    if (aff.has(c)) {
      g.addEventListener("click", () => playByTo(c));
      g.classList.add("clickable");
    }
    svg.appendChild(g);
  }
}

function scoresheet(): string[] {
  const sans: string[] = [];
  for (let i = 1; i < path.length; i++) {
    const placed = ((path[i].p1 | path[i].p2) >>> 0) & ~((path[i - 1].p1 | path[i - 1].p2) >>> 0);
    let b = 0;
    while (((placed >>> b) & 1) === 0) b++;
    sans.push(LABELS[b]);
  }
  return sans;
}

function renderPanel() {
  const cur = current();
  const term = view.term;
  let text: string, cls: string;
  if (term !== 3) {
    cls = valClass(term);
    text = `Game over — ${sideName(turnOf(cur))} loses: the opponent connected all three sides.`;
  } else {
    const v = positionValue(cur);
    cls = valClass(v);
    text = `${sideName(turnOf(cur))} to move ${["is winning", "is losing", "draws", "—"][v]}.`;
  }
  verdictEl.className = `verdict ${cls}`;
  verdictEl.textContent = text;
  countsEl.innerHTML = `<span class="dot p1"></span>${popcount(cur.p1)} &nbsp; <span class="dot p2"></span>${popcount(cur.p2)}`;
  undoBtn.disabled = ptr === 0;

  const sans = scoresheet();
  if (sans.length === 0) {
    lineEl.innerHTML = `<span class="lempty">No moves yet.</span>`;
  } else {
    lineEl.innerHTML = "";
    for (let i = 0; i < sans.length; i += 2) {
      const row = document.createElement("div");
      row.className = "lrow";
      row.innerHTML = `<span class="lnum">${i / 2 + 1}.</span>`;
      for (const j of [i, i + 1]) {
        const span = document.createElement("span");
        if (j < sans.length) {
          span.className = `ply${j === ptr - 1 ? " cur" : ""}`;
          span.textContent = sans[j];
          span.addEventListener("click", () => jumpTo(j + 1));
        }
        row.appendChild(span);
      }
      lineEl.appendChild(row);
    }
  }

  movesEl.innerHTML = "";
  if (term !== 3) {
    movesEl.innerHTML = `<div class="grouphdr">game over</div>`;
    hotMove = null;
    return;
  }
  for (const [label, val, cl] of [["winning", 0, "win"], ["losing", 1, "loss"]] as [string, Val, string][]) {
    const ms = view.moves.filter((m) => m.value === val).sort((a, b) => a.to - b.to);
    if (ms.length === 0) continue;
    const h = document.createElement("div");
    h.className = `grouphdr ${cl}`;
    h.textContent = `${label} — ${ms.length}`;
    movesEl.appendChild(h);
    for (const m of ms) {
      const row = document.createElement("div");
      row.className = "move";
      row.innerHTML = `<span class="mv">${LABELS[m.to]}</span><span class="badge ${cl}">${["W", "L", "D"][val]}</span>`;
      row.addEventListener("click", () => playMove(m));
      row.addEventListener("mouseenter", () => { hotMove = m; renderBoard(); });
      row.addEventListener("mouseleave", () => { hotMove = null; renderBoard(); });
      movesEl.appendChild(row);
    }
  }
}

function render() {
  const cur = current();
  view.term = terminalValue(cur);
  view.moves = view.term === 3 ? legalMoves(cur) : [];
  renderPanel();
  renderBoard();
}

function playByTo(c: number) {
  const m = view.moves.find((x) => x.to === c);
  if (m) playMove(m);
}

function playMove(m: YMove) {
  path = path.slice(0, ptr + 1);
  path.push(m.result);
  ptr++;
  hotMove = null;
  render();
}

function jumpTo(i: number) {
  ptr = i;
  hotMove = null;
  render();
}

undoBtn.addEventListener("click", () => { if (ptr > 0) jumpTo(ptr - 1); });
resetBtn.addEventListener("click", () => { path = [startPos()]; ptr = 0; hotMove = null; render(); });

function buildSizes() {
  sizesEl.innerHTML = "";
  for (const s of SIZES) {
    const b = document.createElement("button");
    b.textContent = String(s);
    b.className = s === n ? "active" : "";
    b.addEventListener("click", () => setSide(s));
    sizesEl.appendChild(b);
  }
}

function setSizesEnabled(on: boolean) {
  for (const b of Array.from(sizesEl.children) as HTMLButtonElement[]) b.disabled = !on;
}

const fmtSecs = (ms: number) => (ms / 1000).toFixed(ms < 1000 ? 2 : 1);

async function setSide(s: number) {
  n = s;
  layout = yLayout(n);
  for (const b of Array.from(sizesEl.children) as HTMLButtonElement[]) {
    b.className = Number(b.textContent) === s ? "active" : "";
  }

  // The whole tablebase is solved live, in a worker. Frame the wait as the demo
  // it is: show the position count and a ticking timer while it computes.
  const token = ++solveToken;
  setSizesEnabled(false);
  undoBtn.disabled = true;
  resetBtn.disabled = true;
  verdictEl.className = "verdict draw";
  solvestatEl.textContent = "";
  const startedAt = performance.now();
  let count = 0;
  let live = true;
  const tick = () => {
    if (!live || token !== solveToken) return;
    const secs = fmtSecs(performance.now() - startedAt);
    verdictEl.textContent = count
      ? `Solving ${count.toLocaleString()} positions live… ${secs}s`
      : `Solving live… ${secs}s`;
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);

  await selectSide(s, (c) => { count = c; });
  live = false; // stop the ticking timer now that the solve is in
  if (token !== solveToken) return; // a newer solve superseded this one

  const info = solveInfo();
  solvestatEl.textContent =
    `Solved live in your browser — ${info.count.toLocaleString()} positions in ${fmtSecs(info.ms)}s.`;
  setSizesEnabled(true);
  resetBtn.disabled = false;
  path = [startPos()];
  ptr = 0;
  hotMove = null;
  render();
}

async function main() {
  buildSizes();
  try {
    await setSide(5);
  } catch (e) {
    console.error("[y-explorer] load failed", e);
    solveToken++; // stop the live timer from overwriting the error
    verdictEl.className = "verdict loss";
    verdictEl.textContent = `Failed to load the solver: ${(e as Error).message}`;
    solvestatEl.textContent = "";
    setSizesEnabled(true);
  }
}

main();
