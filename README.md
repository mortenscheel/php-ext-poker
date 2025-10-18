# php-ext-poker

A high-performance PHP extension for calculating poker equity, written in Rust using [ext-php-rs](https://ext-php.rs/).

## Overview

This extension provides fast poker equity calculations by leveraging Rust's performance and the `aya_poker` library. It allows you to calculate the winning probability of a poker hand against one or more opponent hands on a given board through Monte Carlo simulation.

## Features

- **High Performance**: Written in Rust for maximum speed
- **Monte Carlo Simulation**: Configurable sample sizes for accuracy vs. speed trade-offs
- **Multi-opponent Support**: Calculate equity against multiple opponent hands
- **Flexible Board States**: Works with incomplete boards (flop, turn, or river)
- **Partial Hand Support**: Calculate equity with range representations
- **Deterministic Results**: Optional seeding for reproducible simulations

## Requirements

- PHP 8.0 or higher
- Rust toolchain (for building from source)
- Cargo

## Installation

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/mortenscheel/php-ext-poker
cd php-ext-poker
```

2. Build and install the extension:
Install the cargo-php extension
```bash
cargo install cargo-php
```
Build and install the php extension
```bash
cargo php install --release
```

### IDE Support

The extension includes a stub file (`php-ext-poker.stubs.php`) that provides type hints and documentation for IDEs. To use it:

1. **PHPStorm/IntelliJ IDEA**: Configure the stub file path in Settings → PHP → Include Path
2. **VS Code with Intelephense**: Add to your workspace settings:
   ```json
   {
     "intelephense.stubs": [
       "/path/to/php-ext-poker/php-ext-poker.stubs.php"
     ]
   }
   ```
3. **VS Code with PHP Intelisense**: The extension should automatically detect the stub file if it's in your project

The stub file provides full autocomplete, type checking, and inline documentation for all classes and methods.

## Usage

### Basic Example

```php
<?php

use Poker\EquityCalculator;

$calculator = new EquityCalculator();
$result = $calculator->calculate(
    player: 'As Kh',
    opponents: ['Qd Qc'],
    board: 'Jh 9h 2c'
);

echo $result;  // "45.67% equity [100000 samples, 12345.67 samples per ms]"
echo $result->equity;   // 0.4567
echo $result->samples;  // 100000
echo $result->time;     // 8 (milliseconds)
```

### Advanced Configuration

```php
<?php

use Poker\EquityCalculator;

$calculator = new EquityCalculator();

// Configure number of simulations (default: 100,000)
$calculator->samples(500000);

// Set seed for deterministic results
$calculator->seed(12345);

// Calculate equity against multiple opponents
$result = $calculator->calculate(
    player: 'As Kh',
    opponents: ['Qd Qc', 'Jc Tc', 'Ah 9s'],
    board: '2h 3d 7c'
);

echo "Equity: " . ($result->equity * 100) . "%\n";
```

### Card Notation

Cards are represented using standard poker notation with spaces between cards:
- Ranks: `2`, `3`, `4`, `5`, `6`, `7`, `8`, `9`, `T`, `J`, `Q`, `K`, `A`
- Suits: `h` (hearts), `d` (diamonds), `c` (clubs), `s` (spades)

Examples:
- `As Kh` - Ace of spades, King of hearts
- `2h 3d 7c` - Board with 2 of hearts, 3 of diamonds, 7 of clubs
- `Qd Qc` - Pair of queens

### Empty Boards

You can calculate pre-flop equity by passing an empty board:

```php
<?php

$calculator = new EquityCalculator();
$result = $calculator->calculate(
    player: 'As Kh',
    opponents: ['Qd Qc'],
    board: ''
);
```

### Partial Hands

The calculator supports partial hands (useful for range calculations):

```php
<?php

// Calculate equity with unknown cards (filled randomly in simulation)
$result = $calculator->calculate(
    player: 'As',
    opponents: ['Qd'],
    board: ''
);
```

### Hand Ranking

You can manually evaluate and compare poker hands using the `rankHand()` static method:

```php
<?php

use Poker\EquityCalculator;

// Evaluate 5-card hands
$straightFlush = EquityCalculator::rankHand('Ah Kh Qh Jh Th');
$fullHouse = EquityCalculator::rankHand('Ks Kh Kd 3c 3s');

// Evaluate 7-card hands (best 5 cards used)
$hand1 = EquityCalculator::rankHand('As Kh Qh Jh Th 2c 3d');
$hand2 = EquityCalculator::rankHand('Ks Kh Kd 3c 3s 7h 9d');

// Compare hands (higher rank = better hand)
if ($straightFlush > $fullHouse) {
    echo "Straight flush wins!\n";
}

// Useful for determining winners without full equity calculation
$board = '9h 9d 5c 2h 7s';
$player1 = 'As Ah';
$player2 = 'Kc Kd';

$rank1 = EquityCalculator::rankHand("$player1 $board");
$rank2 = EquityCalculator::rankHand("$player2 $board");

echo $rank1 > $rank2 ? "Player 1 wins" : "Player 2 wins";
```

## API Reference

### `Poker\EquityCalculator`

#### Methods

- `samples(int $samples): EquityCalculator`: Sets the number of Monte Carlo simulations to run (default: 100,000). Returns self for method chaining.
- `seed(int $seed): EquityCalculator`: Sets the random seed for deterministic results. Returns self for method chaining.
- `calculate(string $player, array $opponents, string $board): EquityResult`: Calculates equity for the player's hand against opponent hands on the given board
- `static rankHand(string $hand): int`: Evaluates the strength of a 5 or 7 card hand. Returns an integer rank that can be used to compare hands (higher is better)

### `Poker\EquityResult`

#### Properties

- `equity` (float): Win probability as a decimal (0.0 to 1.0)
- `samples` (int): Number of simulations performed
- `time` (int): Execution time in milliseconds

#### Methods

- `__toString()`: Returns a formatted string representation

### `Poker\Deck`

A class for simulating a deck of cards with shuffling and dealing functionality.

#### Methods

- `__construct()`: Creates a shuffled deck of 52 cards with a random seed
- `static fromSeed(int $seed): Deck`: Creates a shuffled deck with a specific random seed for reproducible results
- `deal(): ?string`: Deals the next card from the deck in poker notation (e.g., "As", "Kh"). Returns null if the deck is empty
- `reset(): void`: Resets the deck to its original shuffled state, putting all dealt cards back
- `count(): int`: Returns the number of remaining cards in the deck

#### Example

```php
<?php

use Poker\Deck;

// Create a shuffled deck
$deck = new Deck();

// Deal some cards
$card1 = $deck->deal();  // e.g., "As"
$card2 = $deck->deal();  // e.g., "Kh"

echo "Remaining cards: " . $deck->count() . "\n";  // 50

// Reset to original state
$deck->reset();
echo "After reset: " . $deck->count() . "\n";  // 52

// Create deterministic deck for testing
$deterministicDeck = Deck::fromSeed(12345);
```

## Performance

The extension is highly optimized for performance. Typical benchmarks on modern hardware:

- 100,000 simulations: ~8-15ms
- 1,000,000 simulations: ~80-150ms
- ~12,000-15,000 samples per millisecond

Performance varies based on:
- Number of opponents
- CPU speed
- Board state complexity

#

## Technical Details

- Built with [ext-php-rs](https://ext-php.rs/) - Rust bindings for PHP extensions
- Uses [aya_poker](https://crates.io/crates/aya_poker) for poker hand evaluation
- Compiled as a cdylib (C dynamic library) for PHP integration
- Implements Monte Carlo simulation for equity calculation

## Credits

- Built with [ext-php-rs](https://ext-php.rs/)
- Poker evaluation by [aya_poker](https://crates.io/crates/aya_poker)