import { useMemo } from "react";
import { PuzzleApp } from "skyscrapers-player";
import "skyscrapers-player/styles.css";
import { TauriEngine } from "./engine/tauri-engine";

/**
 * Desktop host for the shared `<PuzzleApp>`: supplies the Tauri engine and
 * nothing else. The web app's URL query-string handling is deliberately absent
 * — there is no address bar to share here.
 */
function App() {
  const engine = useMemo(() => new TauriEngine(), []);

  return <PuzzleApp engine={engine} />;
}

export default App;
