// Regenerate the README's generated regions from the structured data in
// data/games/*.yaml. The data files are the source of truth; this renders the
// human-facing table from them.
//
//   npm run build     # rewrite README.md in place
//   npm run check     # exit non-zero if README.md is out of date (for CI)
//
// Runs on Node's native TypeScript support (Node >= 23). Only dependency: `yaml`.

import { readFileSync, writeFileSync, readdirSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { parse } from 'yaml'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const GAMES_DIR = join(ROOT, 'data', 'games')
const README = join(ROOT, 'README.md')

interface Source { title: string; url: string; primary?: boolean }
interface Game {
  id: string
  name: string
  family: string
  genre: string
  result: 'first-player-win' | 'second-player-win' | 'draw'
  strength: 'ultra-weak' | 'weak' | 'strong'
  approximate?: boolean
  method: string[]
  year: number | null
  solved_by: string
  cite?: string // optional short citation label for the table (overrides derivation)
  verified: boolean
  sources: Source[]
}

const RESULT_PHRASE: Record<Game['result'], string> = {
  'first-player-win': 'First player wins',
  'second-player-win': 'Second player wins',
  draw: 'Draw',
}

// Display order for the genre sections (familiar → frontier). Slugs match the
// `genre` field; titles are the rendered `###` headings. Keep in sync with
// data/schema.md and scripts/build-site.ts.
const GENRE_ORDER: { slug: string; title: string }[] = [
  { slug: 'alignment', title: 'Alignment & *m,n,k* games' },
  { slug: 'morris', title: 'Morris / mill family' },
  { slug: 'connection', title: 'Connection games' },
  { slug: 'mancala', title: 'Mancala' },
  { slug: 'capture', title: 'Capture & board control' },
  { slug: 'chess', title: 'Chess & chess variants' },
  { slug: 'hunt', title: 'Hunt & unequal forces' },
  { slug: 'cgt', title: 'Combinatorial game theory' },
  { slug: 'shogi', title: 'Shogi variants' },
  { slug: 'go', title: 'Go' },
  { slug: 'imperfect-info', title: 'Imperfect information' },
]

// Human labels for the primary solving method (the first entry in `method`).
const METHOD_LABEL: Record<string, string> = {
  'retrograde-analysis': 'Retrograde analysis',
  'alpha-beta+db': 'Alpha-beta + DB',
  'brute-force-enumeration': 'Brute-force enumeration',
  'mathematical-proof': 'Mathematical proof',
  'proof-number-search': 'Proof-number search',
  'depth-first-pns': 'Depth-first PNS',
  'conspiracy-number-search': 'Conspiracy-number search',
  'threat-space-search': 'Threat-space search',
  'knowledge-based': 'Knowledge-based',
  'strategy-stealing/pairing-strategy': 'Strategy-stealing',
  'monte-carlo-cfr': 'Monte-Carlo CFR',
}
function methodLabel(g: Game): string {
  const m = g.method?.[0]
  if (!m) return '—'
  return METHOD_LABEL[m] ?? m.replace(/[-+/]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
}

function loadGames(): Game[] {
  return readdirSync(GAMES_DIR)
    .filter((f) => f.endsWith('.yaml'))
    .map((f) => parse(readFileSync(join(GAMES_DIR, f), 'utf8')) as Game)
}

// Derive a short citation label from the free-text `solved_by`:
// "Tetsurō Tanaka (田中 哲朗), University of Tokyo" -> "Tanaka 2009";
// "Mehdi Mhalla and Frédéric Prost" -> "Mhalla et al. 2013" (lead author);
// "Erik C. D. van der Werf (program MIGOS)" -> "van der Werf 2002".
const PARTICLES = new Set(['van', 'von', 'der', 'den', 'de', 'da', 'di', 'du', 'la', 'le', 'ten'])
function surnameOf(name: string): string {
  const tokens = name.trim().split(/\s+/)
  let i = tokens.length - 1
  while (i > 0 && PARTICLES.has(tokens[i - 1].toLowerCase())) i--
  return tokens.slice(i).join(' ').replace(/\.$/, '')
}
function cite(g: Game): string {
  // Author list = text before the first parenthesis or semicolon (drops
  // affiliations and later clauses, which otherwise trip " and " false positives).
  const authors = g.solved_by.split(/[(;]/)[0]
  const head = authors.split(/,|&| and /)[0].trim()
  const isFolklore = /folklore|classical hand analysis/i.test(head)
  const multi = /&| and |,|et al/i.test(authors)
  const label =
    g.cite ??
    (isFolklore
      ? 'folklore'
      : `${surnameOf(head)}${multi ? ' et al.' : ''}${g.year ? ` ${g.year}` : ''}`)
  const url = (g.sources.find((s) => s.primary) ?? g.sources[0])?.url
  const link = url ? `[${label}](${url})` : label
  // Dōbutsu shōgi is the flagship worked example — cross-link it.
  return g.id === 'dobutsu-shogi'
    ? `${link} · [worked example below](#worked-example-dōbutsu-shōgi)`
    : link
}

function row(g: Game): string {
  const name = g.id === 'dobutsu-shogi' ? `[${g.name}](#worked-example-dōbutsu-shōgi)` : g.name
  let result = RESULT_PHRASE[g.result]
  if (g.approximate) result += ' (ε-Nash)'
  const strength = g.strength.charAt(0).toUpperCase() + g.strength.slice(1) + (g.approximate ? ', approx.' : '')
  const year = g.year ?? '—'
  return `| ${name} | ${result} | ${strength} | ${methodLabel(g)} | ${year} | ${cite(g)} |`
}

const TABLE_HEADER =
  '| Game | Result under perfect play | Strength | Method | Year | Source |\n|---|---|---|---|---|---|'
const byYearThenName = (a: Game, b: Game) =>
  (a.year ?? 0) - (b.year ?? 0) || a.name.localeCompare(b.name)

function solvedGamesTable(games: Game[]): string {
  // Only citation-verified entries appear in the public tables (citation-gated).
  const verified = games.filter((g) => g.verified)
  const placed = new Set<string>()
  const sections: string[] = []

  for (const { slug, title } of GENRE_ORDER) {
    const inGenre = verified.filter((g) => g.genre === slug).sort(byYearThenName)
    inGenre.forEach((g) => placed.add(g.id))
    if (inGenre.length === 0) continue // skip genres with no verified entries yet
    sections.push(`### ${title}\n\n${[TABLE_HEADER, ...inGenre.map(row)].join('\n')}`)
  }

  // Any verified game whose genre isn't in GENRE_ORDER lands in a trailing
  // "Other" section so it can't silently vanish; warn so we add the slug.
  const orphans = verified.filter((g) => !placed.has(g.id)).sort(byYearThenName)
  if (orphans.length) {
    console.warn(
      `build-readme: ${orphans.length} verified game(s) with unrecognized genre → "Other": ${orphans.map((g) => g.id).join(', ')}`,
    )
    sections.push(`### Other\n\n${[TABLE_HEADER, ...orphans.map(row)].join('\n')}`)
  }

  // Leading newline guarantees a blank line between the BEGIN marker and the
  // first heading so the heading renders on every Markdown engine.
  return `\n${sections.join('\n\n')}`
}

function spliceRegion(md: string, region: string, body: string): string {
  const begin = new RegExp(`(<!-- BEGIN GENERATED:${region}[^>]*-->\\n)[\\s\\S]*?(\\n<!-- END GENERATED:${region} -->)`)
  if (!begin.test(md)) throw new Error(`marker region "${region}" not found in README.md`)
  return md.replace(begin, `$1${body}$2`)
}

const games = loadGames()
const current = readFileSync(README, 'utf8')
const next = spliceRegion(current, 'solved-games', solvedGamesTable(games))

const verifiedCount = games.filter((g) => g.verified).length
const checkMode = process.argv.includes('--check')

if (checkMode) {
  if (next !== current) {
    console.error('README.md is out of date — run `npm run build`.')
    process.exit(1)
  }
  console.log(`README.md up to date (${verifiedCount} verified games).`)
} else if (next === current) {
  console.log(`No change. README.md already reflects ${verifiedCount} verified games.`)
} else {
  writeFileSync(README, next)
  console.log(`Regenerated README.md solved-games table: ${verifiedCount} verified of ${games.length} records.`)
}
