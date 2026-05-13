import path from "path";
import { fileURLToPath } from "url";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react-swc";
import tailwindcss from "@tailwindcss/vite";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// https://vite.dev/config/
export default defineConfig({
  base: process.env.BASE_PATH ?? "/",
  plugins: [react(), tailwindcss()],
  optimizeDeps: {
    // skyscrapers-generator: pre-bundling the wasm-pack output trips Vite's
    //   dependency analyzer.
    // skyscrapers-player: source-only package (file: symlink), let Vite
    //   resolve its TSX directly so HMR and source maps point at the real
    //   files in ../skyscrapers-player/src.
    exclude: ["skyscrapers-generator", "skyscrapers-player"],
  },
  server: {
    fs: {
      allow: [path.resolve(__dirname, "..")],
    },
  },
  test: {
    environment: "node",
  },
});
