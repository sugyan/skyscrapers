interface GameControlsProps {
  onReset: () => void;
  onCheck: () => void;
  onNewPuzzle: () => void;
  completed: boolean;
}

export function GameControls({
  onReset,
  onCheck,
  onNewPuzzle,
  completed,
}: GameControlsProps) {
  return (
    <div className="game-controls">
      <button onClick={onReset}>Reset</button>
      <button onClick={onCheck}>Check</button>
      <button onClick={onNewPuzzle}>New Puzzle</button>
      {completed && <span className="completed-message">Completed!</span>}
    </div>
  );
}
