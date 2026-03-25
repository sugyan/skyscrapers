export interface SamplePuzzle {
  label: string;
  encoded: string;
}

export const samplePuzzles: SamplePuzzle[] = [
  {
    label: "5×5 (seed 42)",
    encoded: "5003000100303000003420000000000000000000000020",
  },
  {
    label: "7×7 (seed 100)",
    encoded:
      "703023204052100000000533034000000030000004000000000020000000000040000000000000",
  },
  {
    label: "7×7 (seed 200)",
    encoded:
      "703440000030002030104000543030001000000000030000000000100050030000000000400000",
  },
];
