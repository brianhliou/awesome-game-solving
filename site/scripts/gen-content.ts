// Generate Starlight content from the structured data in ../data/games/*.yaml.
// The data is the single source of truth — these Markdown pages are regenerated
// on every build (predev/prebuild) and are gitignored, never hand-edited.
//
//   node scripts/gen-content.ts
//
// Output:
//   src/content/docs/games/<genre>/<id>.md   one page per game, grouped by genre
//   src/generated/sidebar.json               one sidebar group per genre

import { readFileSync, writeFileSync, readdirSync, mkdirSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { parse } from "yaml";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const GAMES_DIR = join(ROOT, "data", "games");
const OUT_DIR = join(ROOT, "site", "src", "content", "docs", "games");
const SIDEBAR = join(ROOT, "site", "src", "generated", "sidebar.json");

interface Source { title: string; url: string }
interface Game {
  id: string;
  name: string;
  also_known_as?: string[];
  genre: string;
  series?: string;
  board?: string;
  result: string;
  result_detail?: string;
  strength: string;
  approximate?: boolean;
  method?: string[];
  complexity?: { state_space?: number | null; game_tree?: number | null };
  year?: number | null;
  solved_by?: string;
  cite?: string;
  distance_to_result?: string;
  verified?: boolean;
  sources?: Source[];
  resources?: { code?: Source[] };
  notes?: string;
}

const RESULT_LABEL: Record<string, string> = {
  "first-player-win": "First player wins",
  "second-player-win": "Second player wins",
  draw: "Draw",
};

const GENRE_LABEL: Record<string, string> = {
  alignment: "Alignment (n-in-a-row)",
  cgt: "Combinatorial game theory",
  morris: "Morris (mill games)",
  capture: "Capture",
  hunt: "Hunt",
  connection: "Connection",
  chess: "Chess-likes",
  mancala: "Mancala",
  shogi: "Shogi",
  "imperfect-info": "Imperfect information",
  go: "Go",
};

// Cross-links from a game's data card up to its family hub page (where one exists).
const FAMILY_BY_ID: Record<string, { label: string; href: string }> = {
  y: { label: "the Y connection family", href: "/families/y/" },
};
const FAMILY_BY_GENRE: Record<string, { label: string; href: string }> = {
  morris: { label: "the morris (mill) family", href: "/families/morris/" },
};

const num = (n: number | null | undefined) =>
  typeof n === "number" ? n.toLocaleString("en-US") : "—";
const title = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

function page(g: Game): string {
  const result = RESULT_LABEL[g.result] ?? g.result;
  const desc = `${result} — ${g.strength}${g.approximate ? " (approximate)" : ""} solution.`;

  const facts: [string, string][] = [
    ["Genre", GENRE_LABEL[g.genre] ?? g.genre],
    ["Result", result],
    ["Strength", g.strength + (g.approximate ? " (approximate)" : "")],
    ["State space", num(g.complexity?.state_space)],
    ["Year", g.year ? String(g.year) : "—"],
    ["Verified", g.verified ? "yes" : "no"],
  ];

  const out: string[] = [];
  out.push("---");
  out.push(`title: ${JSON.stringify(g.name)}`);
  out.push(`description: ${JSON.stringify(desc)}`);
  out.push("---");
  out.push("");

  if (g.also_known_as?.length) {
    out.push(`*Also known as: ${g.also_known_as.join(", ")}.*`);
    out.push("");
  }

  if (g.result_detail) {
    out.push(g.result_detail);
    out.push("");
  }

  const fam = FAMILY_BY_ID[g.id] ?? FAMILY_BY_GENRE[g.genre];
  if (fam) {
    out.push(`Part of ${`[${fam.label}](${fam.href})`} — see the family page for the scaling table and a playable explorer.`);
    out.push("");
  }

  out.push("## At a glance");
  out.push("");
  out.push("| | |");
  out.push("|---|---|");
  for (const [k, v] of facts) out.push(`| ${k} | ${v} |`);
  out.push("");

  if (g.board) {
    out.push("## Board");
    out.push("");
    out.push(g.board.trim());
    out.push("");
  }

  if (g.method?.length) {
    out.push("## Method");
    out.push("");
    for (const m of g.method) out.push(`- ${m}`);
    out.push("");
  }

  if (g.solved_by) {
    out.push("## Solved by");
    out.push("");
    out.push(g.solved_by.trim());
    out.push("");
    if (g.distance_to_result) {
      out.push(g.distance_to_result.trim());
      out.push("");
    }
  }

  if (g.notes) {
    out.push("## Notes");
    out.push("");
    out.push(g.notes.trim());
    out.push("");
  }

  if (g.sources?.length) {
    out.push("## Sources");
    out.push("");
    for (const s of g.sources) out.push(`- [${s.title}](${s.url})`);
    out.push("");
  }

  if (g.resources?.code?.length) {
    out.push("## Code");
    out.push("");
    for (const s of g.resources.code) out.push(`- [${s.title}](${s.url})`);
    out.push("");
  }

  return out.join("\n");
}

// --- generate ---
rmSync(OUT_DIR, { recursive: true, force: true });
mkdirSync(OUT_DIR, { recursive: true });
mkdirSync(dirname(SIDEBAR), { recursive: true });

const files = readdirSync(GAMES_DIR).filter((f) => f.endsWith(".yaml"));
const byGenre = new Map<string, Game[]>();

for (const f of files) {
  const g = parse(readFileSync(join(GAMES_DIR, f), "utf8")) as Game;
  if (!byGenre.has(g.genre)) byGenre.set(g.genre, []);
  byGenre.get(g.genre)!.push(g);
}

let count = 0;
for (const [genre, games] of byGenre) {
  mkdirSync(join(OUT_DIR, genre), { recursive: true });
  for (const g of games) {
    writeFileSync(join(OUT_DIR, genre, `${g.id}.md`), page(g));
    count++;
  }
}

// Sidebar: one group per genre (most-populated first), each autogenerated.
const sidebar = [...byGenre.entries()]
  .sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0]))
  .map(([genre, games]) => ({
    label: `${GENRE_LABEL[genre] ?? title(genre)} (${games.length})`,
    items: [{ autogenerate: { directory: `games/${genre}` } }],
  }));
writeFileSync(SIDEBAR, JSON.stringify(sidebar, null, 2));

console.log(`[gen-content] ${count} games across ${byGenre.size} genres → ${OUT_DIR}`);
