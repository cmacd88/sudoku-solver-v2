# Sudoku Solver v2 - TODO

## Tests
- [ ] Add exact-solution regression test (compare solved board to a precomputed correct answer, not just "made progress")
- [ ] Add test that a corrupted/duplicate-filled board fails `is_complete()`
- [ ] Freeze the 30+ level speculation puzzle as a permanent regression test
- [ ] Silence or remove: `create_test_strategy` dead-code warning, unused `strategy` bindings in two test files

## Features
- [ ] Multi-board-size support (6x6, 16x16), parameterized over dimensions
- [ ] Decoupled visualizer communicating via a well-defined data stream (not baked into CLI)
- [ ] Difficulty-level system (restrict strategy complexity + initial clue count)
- [ ] Performance profiling / benchmarking suite
- [ ] Most-constrained-cell prioritization for deterministic strategy selection (currently only speculation uses a cascade heuristic)
- [ ] Real-time solving visualization via event streaming
- [ ] ML-based strategy selection