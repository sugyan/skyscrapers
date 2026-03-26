export interface SamplePuzzle {
  label: string;
  n: number;
  clues: {
    top: (number | null)[];
    bottom: (number | null)[];
    left: (number | null)[];
    right: (number | null)[];
  };
  board: (number | null)[][];
  solution: number[][];
}

export const samplePuzzles: SamplePuzzle[] = [
  {
    label: "4×4",
    n: 4,
    clues: {
      top: [null, null, null, 3],
      bottom: [null, 2, null, null],
      left: [null, 3, null, null],
      right: [4, null, null, null],
    },
    board: [
      [null, null, null, null],
      [null, null, null, null],
      [null, null, null, null],
      [null, null, null, null],
    ],
    solution: [
      [4, 3, 2, 1],
      [1, 2, 4, 3],
      [3, 4, 1, 2],
      [2, 1, 3, 4],
    ],
  },
  {
    label: "5×5",
    n: 5,
    clues: {
      top: [null, null, 3, null, null],
      bottom: [null, 1, null, null, 3],
      left: [null, 3, null, null, null],
      right: [null, null, 3, 4, 2],
    },
    board: [
      [null, null, null, null, null],
      [null, null, null, null, null],
      [null, null, null, null, null],
      [null, null, null, null, null],
      [null, null, null, 2, null],
    ],
    solution: [
      [3, 4, 2, 1, 5],
      [2, 1, 3, 5, 4],
      [1, 3, 5, 4, 2],
      [5, 2, 4, 3, 1],
      [4, 5, 1, 2, 3],
    ],
  },
  {
    label: "6×6",
    n: 6,
    clues: {
      top: [3, 2, 1, null, 2, null],
      bottom: [null, null, 2, null, null, 2],
      left: [null, null, 2, 5, null, null],
      right: [null, null, null, null, 2, 4],
    },
    board: [
      [null, null, null, null, null, null],
      [null, null, null, null, null, null],
      [null, null, null, null, 2, null],
      [null, null, null, null, null, null],
      [null, null, null, 1, null, null],
      [null, null, null, null, null, null],
    ],
    solution: [
      [2, 4, 6, 3, 5, 1],
      [5, 3, 2, 6, 1, 4],
      [3, 6, 1, 4, 2, 5],
      [1, 2, 4, 5, 3, 6],
      [4, 5, 3, 1, 6, 2],
      [6, 1, 5, 2, 4, 3],
    ],
  },
  {
    label: "7×7",
    n: 7,
    clues: {
      top: [4, null, 2, 3, null, 1, 2],
      bottom: [null, 4, null, null, 2, 6, 2],
      left: [null, null, 4, null, null, null, null],
      right: [null, null, null, 4, null, 5, null],
    },
    board: [
      [null, null, null, null, null, null, null],
      [null, null, null, 3, 5, null, null],
      [1, null, null, null, null, null, null],
      [null, null, null, null, null, null, null],
      [null, 2, null, null, null, null, null],
      [null, null, null, null, null, null, null],
      [null, null, null, null, null, null, null],
    ],
    solution: [
      [2, 6, 5, 4, 1, 7, 3],
      [4, 7, 1, 3, 5, 6, 2],
      [1, 5, 4, 6, 3, 2, 7],
      [3, 1, 7, 2, 6, 5, 4],
      [6, 2, 3, 1, 7, 4, 5],
      [7, 4, 6, 5, 2, 3, 1],
      [5, 3, 2, 7, 4, 1, 6],
    ],
  },
];
