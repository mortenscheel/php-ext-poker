#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(non_snake_case)]

use aya_poker::base::{CARDS, Hand};
use aya_poker::deck::Deck;
use aya_poker::poker_rank;
use ext_php_rs::{exception::PhpResult, prelude::*};
use std::cmp::Ordering;
use std::time::Instant;

#[derive(Copy, Clone)]
#[php_class]
#[php(name = "Poker\\EquityCalculator")]
pub struct EquityCalculator {
    pub samples: usize,
    pub seed: u64,
}

#[php_class]
#[php(name = "Poker\\EquityResult")]
pub struct EquityResult {
    #[php(prop)]
    pub equity: f64,
    #[php(prop)]
    pub samples: usize,
    #[php(prop)]
    pub time: usize,
}

#[php_impl]
impl EquityResult {
    #[php(name = "__toString")]
    pub fn __toString(&self) -> String {
        format!(
            "{:.2}% equity [{} samples, {:.2} samples per ms]",
            self.equity * 100.0,
            self.samples,
            self.samples as f64 / self.time as f64
        )
    }
}

#[php_impl]
impl EquityCalculator {
    pub fn __construct() -> Self {
        Self {
            samples: 100_000,
            seed: 0,
        }
    }

    pub fn samples(&mut self, samples: usize)  {
        self.samples = samples;
    }

    pub fn seed(&mut self, seed: u64) {
        self.seed = seed;
    }

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
}

#[php_module]
pub fn module(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<EquityCalculator>().class::<EquityResult>()
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
