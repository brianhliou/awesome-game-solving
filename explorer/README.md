# Morris explorer

A static, client-side perfect-play explorer for the solved morris boards. The
validated Rust engine (`../engine`) is compiled to WASM and runs in the browser;
a packed WLD tablebase is loaded as data and probed through the engine's own
dense index, so the browser never re-implements the rules.

## Build

The WASM package and the tablebase are generated, not committed.

```sh
# 1. Compile the engine bindings to WASM (writes ../wasm/pkg)
wasm-pack build ../wasm --target web --release

# 2. Generate the tablebase and drop it in public/ under an opaque name
#    (a .gz name would make servers set Content-Encoding: gzip and fight our
#    own DecompressionStream).
cargo run --release --manifest-path ../engine/Cargo.toml --bin morris6_tablebase -- ../engine/artifacts/morris6
gzip -9 -c ../engine/artifacts/morris6.wld > public/morris6.tb

# 3. Install and run / build
npm install
npm run dev        # local dev server
npm run build      # static bundle in dist/
```

`dist/` deploys anywhere static (GitHub Pages). The whole payload is ~3.6 MB
(~200 KB wasm + ~13 KB app + ~3.4 MB tablebase, fetched once).
