import { defineConfig } from "vite";
import { fileURLToPath } from "node:url";

const page = (name: string) => fileURLToPath(new URL(name, import.meta.url));

// Relative base so the build drops into a subpath on GitHub Pages
// (e.g. /solved-games/explorer/) without rewriting asset URLs.
export default defineConfig({
  base: "./",
  build: {
    target: "es2022",
    rollupOptions: {
      input: {
        main: page("index.html"), // six/nine men's morris explorer
        y: page("y.html"), // Y connection-game explorer
      },
    },
  },
});
