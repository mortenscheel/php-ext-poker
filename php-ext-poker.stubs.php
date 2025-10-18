<?php

// Stubs for php-ext-poker

namespace Poker {
    class EquityCalculator {
        /**
         * Modify number of samples
         */
        public function samples(int $samples): \Poker\EquityCalculator {}

        /**
         * Modify rng seed
         */
        public function seed(int $seed): \Poker\EquityCalculator {}

        /**
         * Calculate equity of the player's hand
         *
         * @param string $player Hero's hand in poker notation
         * @param string[] $opponents Villain hands in poker notation
         * @param string $board Board state in poker notation
         *
         */
        public function calculate(string $player, array $opponents, string $board): \Poker\EquityResult {}

        /**
         * Evaluates the strength of a 5 or 7 card hand.
         * Can be used to compare hands
         */
        public static function rankHand(string $hand): int {}

        /**
         * Construct a calculator with default settings
         */
        public function __construct() {}
    }

    class EquityResult {
        /**
         * @var int number of iterations of the calculation
         */
        public $samples;

        /**
         * @var double the result of an equity calculation
         */
        public $equity;

        /**
         * @var int calculation duration in milliseconds
         */
        public $time;

        public function __toString(): string {}

        public function __construct() {}
    }

    class Deck {
        /**
         * Create a shuffled deck of 52 cards with specific random seed
         */
        public static function fromSeed(int $seed): \Poker\Deck {}

        /**
         * Deal the next card from the deck
         *
         * @return ?string null if the deck is empty
         */
        public function deal(): ?string {}

        /**
         * Reset the deck to its original shuffled state
         *
         * @return void
         */
        public function reset() {}

        /**
         * Remaining cards in deck
         */
        public function count(): int {}

        /**
         * Create a shuffled deck of 52 cards with a random seed
         */
        public function __construct() {}
    }
}
