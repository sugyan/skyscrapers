// Opt-in WASM engine entry. Importing this module pulls in the
// `skyscrapers-generator` WebAssembly bindings — consumers that bring their
// own (e.g. HTTP-backed) SkyscrapersEngine should not import from here.
export { WasmEngine, convertWasmResult } from "./engine/wasm-engine";
export type { WasmPuzzleResult } from "./engine/wasm-engine";
