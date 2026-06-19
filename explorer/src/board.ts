//! Board geometry for the morris explorer. Point indices match the engine's
//! ring ordering (within a ring: corner, mid, corner, mid… clockwise from the
//! top-left; rings outer→inner). Coordinates are in a unit grid; the SVG viewBox
//! scales them.

export interface BoardLayout {
  /** unit (x, y) per point index */
  points: [number, number][];
  /** drawn lines between adjacent points (the playable connections) */
  edges: [number, number][];
}

/** Two-ring, 16-point board: six men's morris (nine men's minus the outer ring). */
export const SIX_MENS: BoardLayout = {
  points: [
    // outer ring 0..7
    [0, 0], [3, 0], [6, 0], [6, 3], [6, 6], [3, 6], [0, 6], [0, 3],
    // inner ring 8..15
    [2, 2], [3, 2], [4, 2], [4, 3], [4, 4], [3, 4], [2, 4], [2, 3],
  ],
  edges: [
    // outer square
    [0, 1], [1, 2], [2, 3], [3, 4], [4, 5], [5, 6], [6, 7], [7, 0],
    // inner square
    [8, 9], [9, 10], [10, 11], [11, 12], [12, 13], [13, 14], [14, 15], [15, 8],
    // spokes joining the side-midpoints
    [1, 9], [3, 11], [5, 13], [7, 15],
  ],
};
