# Sudoku Strategy Definitions

This directory contains JSON-based strategy definitions for the Sudoku solver. Strategies are organized by difficulty level.

## Directory Structure

```
strategies/
├── basic/              # Basic strategies (difficulty 1-2)
│   ├── naked_single.json
│   └── hidden_single.json
├── intermediate/       # Intermediate strategies (difficulty 3-5)
│   ├── naked_pair.json
│   └── pointing_pair.json
└── advanced/          # Advanced strategies (difficulty 6+)
    └── (future strategies)
```

## Strategy JSON Format

Each strategy is defined in a JSON file with the following structure:

```json
{
  "name": "strategy_name",
  "difficulty": 1,
  "description": "Human-readable description",
  "applicable_dimensions": ["9x9", "6x6", "16x16"],
  "pattern": {
    "type": "pattern_type",
    "conditions": [...]
  },
  "action": {
    "type": "action_type",
    "target": "target_cells",
    "candidates": "candidate_source"
  },
  "priority": 100
}
```

### Fields

- **name**: Unique identifier for the strategy
- **difficulty**: Numeric difficulty level (1 = easiest, higher = harder)
- **description**: Human-readable explanation of what the strategy does
- **applicable_dimensions**: List of board sizes this strategy works with
- **pattern**: Defines what pattern to look for on the board
- **action**: Defines what to do when the pattern is found
- **priority**: Selection priority (higher = selected first)

### Pattern Types

#### `single_cell`
Matches individual cells with specific properties.

```json
{
  "type": "single_cell",
  "conditions": [
    {"type": "single_candidate"}
  ]
}
```

#### `cell_group`
Matches groups of cells within a unit (row, column, or box).

```json
{
  "type": "cell_group",
  "unit_type": ["row", "column", "box"],
  "conditions": [
    {"type": "cell_count", "count": 2},
    {"type": "candidate_count", "count": 2},
    {"type": "same_candidates", "value": true}
  ]
}
```

#### `pointing_candidates`
Matches candidates that point from one unit to another.

```json
{
  "type": "pointing_candidates",
  "source_unit": "box",
  "target_unit": "row",
  "conditions": [
    {"type": "restricted_to_line", "value": true}
  ]
}
```

### Action Types

#### `set_cell_value`
Sets a cell to a specific value.

```json
{
  "type": "set_cell_value",
  "target": "matched_cells",
  "value": "single_candidate"
}
```

#### `eliminate_candidates`
Removes candidates from cells.

```json
{
  "type": "eliminate_candidates",
  "target": "other_cells_in_unit",
  "candidates": "matched_candidates"
}
```

## Implemented Strategies

### Basic Strategies

#### Naked Single (Priority: 100, Difficulty: 1)
A cell with only one candidate must contain that value.

**Example**: If a cell can only be 5, place 5 in that cell.

#### Hidden Single (Priority: 90, Difficulty: 2)
A value that can only go in one cell within a unit must be placed there.

**Example**: If 7 can only go in one cell in a row, place 7 there.

### Intermediate Strategies

#### Naked Pair (Priority: 70, Difficulty: 3)
Two cells in a unit with the same two candidates eliminate those candidates from other cells.

**Example**: If cells A and B both have candidates {3, 7}, remove 3 and 7 from all other cells in that unit.

#### Pointing Pair (Priority: 60, Difficulty: 4)
Candidates restricted to a line within a box eliminate candidates in that line outside the box.

**Example**: If 5 can only appear in the top row of a box, eliminate 5 from the rest of that row.

## Adding New Strategies

To add a new strategy:

1. Create a new JSON file in the appropriate difficulty directory
2. Define the strategy using the JSON format above
3. Implement a corresponding matcher in `src/strategy/matcher.rs` if needed
4. Add the matcher to the `create_matcher` function
5. Test the strategy with various puzzles

## Strategy Selection

The solver can use different policies to select strategies:

- **Priority**: Select by priority value (highest first)
- **Difficulty**: Select by difficulty (easiest first)
- **FirstMatch**: Select the first strategy that finds a match

## Usage

Strategies are automatically loaded from this directory when the solver starts. The strategy bank validates each JSON file and makes strategies available to the solver.

```rust
use sudoku_solver_v2::strategy::StrategyBank;

// Load all strategies
let bank = StrategyBank::load_from_directory("strategies")?;

// Get a specific strategy
let naked_single = bank.get_strategy("naked_single");

// Get strategies by difficulty
let easy_strategies = bank.get_strategies_up_to_difficulty(2);
