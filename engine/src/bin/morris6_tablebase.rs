//! Emit the six men's morris tablebase as a packed 2-bit WLD file addressed by
//! the dense index, plus a JSON sidecar. This is the artifact the explorer loads
//! (probe = read 2 bits at `index(state)`) and that ships as a release asset.
//!
//! Encoding: 2 bits per dense index, little-endian within each byte.
//!   0 = not reachable (absent from the solved set)
//!   1 = win (side to move wins)   2 = loss   3 = draw
//!
//! Values come from the fast forward-only `solve_reachable` (the trusted oracle),
//! so generating the artifact takes ~80s rather than the dense sweep's hours.
//!
//! Run: `cargo run --release --bin morris6_tablebase -- artifacts/morris6`

use game_solver::games::GenericMorris;
use game_solver::{solve_reachable, Game, Outcome, RulesGame};
use std::io::Write;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "artifacts/morris6".to_string());
    let g = GenericMorris::six_mens();
    let total = Game::num_states(&g);

    eprintln!("solving six men's (solve_reachable)…");
    let sol = solve_reachable(&g);
    let (win, loss, draw) = (
        sol.count(Outcome::Win),
        sol.count(Outcome::Loss),
        sol.count(Outcome::Draw),
    );
    let start = RulesGame::start(&g);
    let start_val = sol.get(&start).unwrap();
    eprintln!("  reachable {}  (win {win}, loss {loss}, draw {draw})  start {start_val:?}", sol.len());

    // Pack into a 2-bit array over the full dense index.
    let mut packed = vec![0u8; (total as usize).div_ceil(4)];
    for (s, o) in sol.iter() {
        let code: u8 = match o {
            Outcome::Win => 1,
            Outcome::Loss => 2,
            Outcome::Draw => 3,
        };
        let idx = Game::index(&g, s) as usize;
        packed[idx >> 2] |= code << ((idx & 3) * 2);
    }

    let wld_path = format!("{out}.wld");
    let meta_path = format!("{out}.meta.json");
    if let Some(parent) = std::path::Path::new(&wld_path).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::File::create(&wld_path).unwrap().write_all(&packed).unwrap();

    let meta = format!(
        "{{\n  \"game\": \"six-mens-morris\",\n  \"board\": \"two-ring-16pt-no-flying\",\n  \"men\": 6,\n  \"encoding\": \"2bit-le; 0=unreachable 1=win 2=loss 3=draw; addressed by dense index\",\n  \"num_states\": {total},\n  \"reachable\": {},\n  \"win\": {win},\n  \"loss\": {loss},\n  \"draw\": {draw},\n  \"start_value\": \"{start_val:?}\",\n  \"bytes\": {}\n}}\n",
        sol.len(),
        packed.len()
    );
    std::fs::write(&meta_path, meta).unwrap();

    eprintln!("wrote {wld_path} ({:.1} MiB) and {meta_path}", packed.len() as f64 / (1u64 << 20) as f64);
}
