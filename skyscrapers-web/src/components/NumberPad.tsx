interface NumberPadProps {
  n: number;
  currentValue: number | null;
  onNumberSelect: (value: number) => void;
  onClear: () => void;
}

export function NumberPad({
  n,
  currentValue,
  onNumberSelect,
  onClear,
}: NumberPadProps) {
  const buttons: React.ReactNode[] = [];

  for (let i = 1; i <= n; i++) {
    buttons.push(
      <button
        key={i}
        className={`numpad-btn${currentValue === i ? " active" : ""}`}
        onClick={() => onNumberSelect(i)}
      >
        {i}
      </button>
    );
  }

  buttons.push(
    <button key="clear" className="numpad-btn clear" onClick={onClear}>
      ×
    </button>
  );

  return <div className="number-pad">{buttons}</div>;
}
