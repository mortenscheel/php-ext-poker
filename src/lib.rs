#![cfg_attr(windows, feature(abi_vectorcall))]

use aya_poker::base::{Hand, Rank, Suit, CARDS};
use aya_poker::deck::{Deck, FullDeck};
use aya_poker::poker_rank;
use ext_php_rs::types::ZendClassObject;
use ext_php_rs::{exception::PhpResult, prelude::*};
use rand::RngCore;
use std::cmp::Ordering;
use std::time::Instant;

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<EquityCalculator>()
        .class::<EquityResult>()
        .class::<PhpDeck>()
}

#[derive(Copy, Clone)]
#[php_class]
#[php(name = "Poker\\EquityCalculator")]
pub struct EquityCalculator {
    pub samples: usize,
    pub seed: u64,
}

#[php_impl]
impl EquityCalculator {
    /// Construct a calculator with default settings
    pub fn __construct() -> Self {
        Self {
            samples: 100_000,
            seed: 0,
        }
    }

    /// Modify number of samples
    pub fn samples(
        self_: &mut ZendClassObject<EquityCalculator>,
        samples: usize,
    ) -> &mut ZendClassObject<EquityCalculator> {
        self_.samples = samples;
        self_
    }

    /// Modify rng seed
    pub fn seed(
        self_: &mut ZendClassObject<EquityCalculator>,
        seed: u64,
    ) -> &mut ZendClassObject<EquityCalculator> {
        self_.seed = seed;
        self_
    }

    /// Calculate equity of the player's hand
    ///
    /// @param string $player Hero's hand in poker notation
    /// @param string[] $opponents Villain hands in poker notation
    /// @param string $board Board state in poker notation
    ///
    pub fn calculate(
        &self,
        player: &str,
        opponents: Vec<&str>,
        board: &str,
    ) -> PhpResult<EquityResult> {
        let start = Instant::now();
        let player = parse_hand(player, 2)?;
        let opponents = opponents
            .iter()
            .map(|op| parse_hand(op, 2))
            .collect::<Result<Vec<_>, _>>()?;
        let board = parse_hand(board, 5)?;

        let all_opponent_cards = opponents.iter().flat_map(|o| o.iter()).collect::<Hand>();
        // To simulate board run-outs, we begin by preparing a deck
        // that doesn't contain the already dealt-out cards
        let available_cards = CARDS
            .iter()
            .filter(|c| !player.contains(c))
            .filter(|c| !all_opponent_cards.contains(c))
            .filter(|c| !board.contains(c));
        let mut deck = Deck::with_seed(available_cards, self.seed);

        let mut pots_won = 0.0;
        for _ in 0..self.samples {
            // Then, for each run we draw cards to complete the board
            deck.reset();
            let missing = 5 - board.len();
            let complete_board = board
                .iter()
                .chain(deck.deal(missing).unwrap().iter())
                .collect::<Hand>();
            let mut player_hand = player;
            let player_missing = 2 - player_hand.len();
            if player_missing > 0 {
                player_hand = player_hand
                    .iter()
                    .chain(deck.deal(player_missing).unwrap().iter())
                    .collect::<Hand>();
            }
            // Evaluate the player's hand given the completed board
            player_hand.extend(complete_board.iter());
            let player_rank = poker_rank(&player_hand);

            let opponent_rank = opponents
                .iter()
                .map(|o| {
                    let mut opponent = *o;
                    let missing = 2 - opponent.len();
                    if missing > 0 {
                        opponent = opponent
                            .iter()
                            .chain(deck.deal(missing).unwrap().iter())
                            .collect::<Hand>();
                    }
                    opponent.extend(complete_board.iter());
                    poker_rank(&opponent)
                })
                .max()
                .unwrap();

            // And record the player's share of the pot for the run
            match player_rank.cmp(&opponent_rank) {
                Ordering::Greater => pots_won += 1.0,
                Ordering::Less => {}
                Ordering::Equal => pots_won += 0.5,
            };
        }

        let time = start.elapsed().as_millis() as usize;
        let equity = pots_won / self.samples as f64;

        Ok(EquityResult {
            equity,
            samples: self.samples,
            time,
        })
    }

    /// Evaluates the strength of a 5 or 7 card hand.
    /// Can be used to compare hands
    pub fn rank_hand(hand: &str) -> u16 {
        let hand: Hand = hand.parse().unwrap();
        poker_rank(&hand).0
    }
}

#[php_class]
#[php(name = "Poker\\EquityResult")]
pub struct EquityResult {
    /// @var double the result of an equity calculation
    #[php(prop)]
    pub equity: f64,
    /// @var int number of iterations of the calculation
    #[php(prop)]
    pub samples: usize,
    /// @var int calculation duration in milliseconds
    #[php(prop)]
    pub time: usize,
}

#[php_impl]
impl EquityResult {
    #[php(name = "__toString")]
    pub fn stringable(&self) -> String {
        format!(
            "{:.2}% equity [{} samples, {:.2} samples per ms]",
            self.equity * 100.0,
            self.samples,
            self.samples as f64 / self.time as f64
        )
    }
}

#[php_class]
#[php(name = "Poker\\Deck")]
pub struct PhpDeck {
    deck: FullDeck,
}
#[php_impl]
impl PhpDeck {
    /// Create a shuffled deck of 52 cards with a random seed
    pub fn __construct() -> Self {
        Self {
            deck: FullDeck::with_seed(rand::rng().next_u64()),
        }
    }

    /// Create a huffled deck of 52 cards with specific random seed
    pub fn from_seed(seed: u64) -> Self {
        Self {
            deck: FullDeck::with_seed(seed),
        }
    }

    /// Deal the next card from the deck
    ///
    /// @return ?string null if the deck is empty
    pub fn deal(&mut self) -> Option<String> {
        if self.deck.is_empty() {
            return None;
        }
        match self.deck.deal(1) {
            Some(cards) => {
                let rank_str = match cards[0].rank() {
                    Rank::Ace => "A",
                    Rank::King => "K",
                    Rank::Queen => "Q",
                    Rank::Jack => "J",
                    Rank::Ten => "T",
                    Rank::Nine => "9",
                    Rank::Eight => "8",
                    Rank::Seven => "7",
                    Rank::Six => "6",
                    Rank::Five => "5",
                    Rank::Four => "4",
                    Rank::Three => "3",
                    Rank::Two => "2",
                };

                let suit_str = match cards[0].suit() {
                    Suit::Hearts => "h",
                    Suit::Diamonds => "d",
                    Suit::Clubs => "c",
                    Suit::Spades => "s",
                };

                Some(format!("{}{}", rank_str, suit_str))
            }
            None => None,
        }
    }

    /// Reset the deck to its original shuffled state
    ///
    /// @return void
    pub fn reset(&mut self) {
        self.deck.reset();
    }

    /// Remaining cards in deck
    pub fn count(&self) -> usize {
        self.deck.len()
    }
}

fn parse_hand(val: &str, max: usize) -> Result<Hand, String> {
    let hand: Hand = match val.parse::<Hand>() {
        Ok(hand) => {
            if hand.len() > max {
                return Err(format!("Maximum {} cards allowed", max));
            }

            hand
        }
        Err(_) => {
            return Err(format!("Unable to parse {}", val));
        }
    };

    Ok(hand)
}
