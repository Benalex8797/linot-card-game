use crate::state::MatchData;
use linot::{Card, CardSuit, CardValue};

/// Game engine for Whot/Linot card game logic
pub struct GameEngine;

impl GameEngine {
    /// Create a full standard Whot deck (54 cards total)
    /// Distribution per Whot rules:
    /// - Circle: 1-14 (but limited distribution, ~8 cards)
    /// - Cross: 1-7, 10, 13, 14 (~8 cards)
    /// - Triangle: 1-5, 7, 10, 13, 14 (~8 cards)
    /// - Square: 1-5, 7, 10, 13, 14 (~8 cards)
    /// - Star: 1-8 (~8 cards)
    /// - Whot cards: 5 cards (value 20)
    pub fn create_deck() -> Vec<Card> {
        let mut deck = Vec::with_capacity(54);

        // Circle suit - most variety (8 cards: 1, 2, 3, 4, 5, 7, 11, 14)
        let circle_values = [
            CardValue::One,
            CardValue::Two,
            CardValue::Three,
            CardValue::Four,
            CardValue::Five,
            CardValue::Seven,
            CardValue::Eleven,
            CardValue::Fourteen,
        ];
        for &value in &circle_values {
            deck.push(Card {
                suit: CardSuit::Circle,
                value,
            });
        }

        // Cross suit (8 cards: 1, 2, 3, 5, 7, 10, 13, 14)
        let cross_values = [
            CardValue::One,
            CardValue::Two,
            CardValue::Three,
            CardValue::Five,
            CardValue::Seven,
            CardValue::Ten,
            CardValue::Thirteen,
            CardValue::Fourteen,
        ];
        for &value in &cross_values {
            deck.push(Card {
                suit: CardSuit::Cross,
                value,
            });
        }

        // Triangle suit (10 cards: 1, 2, 3, 4, 5, 7, 10, 11, 13, 14)
        let triangle_values = [
            CardValue::One,
            CardValue::Two,
            CardValue::Three,
            CardValue::Four,
            CardValue::Five,
            CardValue::Seven,
            CardValue::Ten,
            CardValue::Eleven,
            CardValue::Thirteen,
            CardValue::Fourteen,
        ];
        for &value in &triangle_values {
            deck.push(Card {
                suit: CardSuit::Triangle,
                value,
            });
        }

        // Square suit (8 cards: 1, 2, 3, 5, 7, 10, 13, 14)
        let square_values = [
            CardValue::One,
            CardValue::Two,
            CardValue::Three,
            CardValue::Five,
            CardValue::Seven,
            CardValue::Ten,
            CardValue::Thirteen,
            CardValue::Fourteen,
        ];
        for &value in &square_values {
            deck.push(Card {
                suit: CardSuit::Square,
                value,
            });
        }

        // Star suit (13 cards: 1, 2, 3, 4, 5, 7, 8, 10, 11, 12, 13, 14, extra 8)
        let star_values = [
            CardValue::One,
            CardValue::Two,
            CardValue::Three,
            CardValue::Four,
            CardValue::Five,
            CardValue::Seven,
            CardValue::Eight,
            CardValue::Eight, // Extra 8 for Star
            CardValue::Ten,
            CardValue::Eleven,
            CardValue::Twelve,
            CardValue::Thirteen,
            CardValue::Fourteen,
        ];
        for &value in &star_values {
            deck.push(Card {
                suit: CardSuit::Star,
                value,
            });
        }

        // Add 5 Whot (wild) cards - displayed as Star 20
        for _ in 0..5 {
            deck.push(Card {
                suit: CardSuit::Star,
                value: CardValue::Whot,
            });
        }

        deck
    }

    /// Shuffle deck using deterministic seed derived from chain_id
    pub fn shuffle_with_seed(deck: &mut Vec<Card>, seed: &[u8]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Create deterministic RNG from seed
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let mut rng_state = hasher.finish();

        // Fisher-Yates shuffle with deterministic RNG
        for i in (1..deck.len()).rev() {
            // Generate random index using LCG
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (rng_state as usize) % (i + 1);
            deck.swap(i, j);
        }
    }

    /// Deal initial hands to players (6 cards each for V1)
    pub fn deal_initial_hands(deck: &mut Vec<Card>, num_players: usize) -> Vec<Vec<Card>> {
        const CARDS_PER_PLAYER: usize = 6;
        let mut hands = vec![Vec::with_capacity(CARDS_PER_PLAYER); num_players];

        for _ in 0..CARDS_PER_PLAYER {
            for player_hand in hands.iter_mut() {
                if let Some(card) = deck.pop() {
                    player_hand.push(card);
                }
            }
        }

        hands
    }

    /// Check if a card can be played on top of another card
    /// Supports: Hold On state, penalty blocking/stacking, shape demands
    pub fn is_valid_play(
        card: &Card,
        top_card: &Card,
        active_demand: Option<CardSuit>,
        pending_penalty: u8,
        hold_on_active: bool,
        hold_on_required_shape: Option<CardSuit>,
    ) -> bool {
        // Whot card (20) can always be played unless Hold On is active
        if card.value == CardValue::Whot {
            if hold_on_active {
                // Can only play Whot during Hold On if it matches the required shape
                if let Some(required_shape) = hold_on_required_shape {
                    return card.suit == required_shape;
                }
                return false;
            }
            return true;
        }

        // Hold On state: must match the required shape
        if hold_on_active {
            if let Some(required_shape) = hold_on_required_shape {
                return card.suit == required_shape;
            }
        }

        // If there's a pending penalty, can only block with same value or accept penalty
        if pending_penalty > 0 {
            return match top_card.value {
                CardValue::Two => card.value == CardValue::Two,
                CardValue::Five => card.value == CardValue::Five,
                _ => false,
            };
        }

        // If there's an active shape demand (from Whot card), must match that suit
        if let Some(demanded_suit) = active_demand {
            return card.suit == demanded_suit || card.value == CardValue::Whot;
        }

        // Normal play: match suit or value
        card.suit == top_card.suit || card.value == top_card.value
    }

    /// Get the special effect of a card
    pub fn get_card_effect(card: &Card) -> SpecialEffect {
        match card.value {
            CardValue::Whot => SpecialEffect::ChooseShape,
            CardValue::One => SpecialEffect::PlayAgain,      // Hold On
            CardValue::Two => SpecialEffect::DrawTwo,        // Pick Two
            CardValue::Five => SpecialEffect::DrawThree,     // Pick Three
            CardValue::Eight => SpecialEffect::SkipNext,     // Suspension
            CardValue::Fourteen => SpecialEffect::AllDrawOne, // General Market
            _ => SpecialEffect::None,
        }
    }

    /// Apply special card effect to match state
    pub fn apply_effect(
        state: &mut MatchData,
        effect: SpecialEffect,
        chosen_suit: Option<CardSuit>,
        played_card_suit: CardSuit,
    ) {
        match effect {
            SpecialEffect::ChooseShape => {
                if let Some(suit) = chosen_suit {
                    state.active_shape_demand = Some(suit);
                }
                state.hold_on_active = false;
                state.hold_on_required_shape = None;
            }
            SpecialEffect::PlayAgain => {
                // Hold On (1): Current player must play again with same shape
                state.hold_on_active = true;
                state.hold_on_required_shape = Some(played_card_suit);
            }
            SpecialEffect::DrawTwo => {
                // Stack penalties if already active
                if state.pending_penalty > 0 {
                    state.penalty_stack += 2;
                    state.pending_penalty += 2;
                } else {
                    state.pending_penalty = 2;
                    state.penalty_stack = 2;
                }
            }
            SpecialEffect::DrawThree => {
                // Stack penalties if already active
                if state.pending_penalty > 0 {
                    state.penalty_stack += 3;
                    state.pending_penalty += 3;
                } else {
                    state.pending_penalty = 3;
                    state.penalty_stack = 3;
                }
            }
            SpecialEffect::SkipNext => {
                // Suspension handled in contract by advancing turn
            }
            SpecialEffect::AllDrawOne => {
                // General Market - handled in contract
            }
            SpecialEffect::None => {
                // Clear active demand and Hold On if no special effect
                if !state.hold_on_active {
                    state.active_shape_demand = None;
                }
            }
        }
    }

    /// Advance to next player's turn
    pub fn advance_turn(state: &mut MatchData) {
        let num_players = state.players.len();
        if num_players > 0 {
            state.current_player_index = (state.current_player_index + 1) % num_players;
        }
    }

    /// Check if the game has ended
    pub fn check_game_end(state: &MatchData) -> Option<GameResult> {
        // Check if any player has won (0 cards)
        for (i, player) in state.players.iter().enumerate() {
            if player.is_active && player.card_count == 0 {
                return Some(GameResult::Winner(i));
            }
        }

        // Check if deck is empty and no valid moves (stalemate)
        if state.deck.is_empty() && state.status == crate::state::MatchStatus::InProgress {
            // Find player with fewest cards
            let min_cards = state
                .players
                .iter()
                .filter(|p| p.is_active)
                .map(|p| p.card_count)
                .min()
                .unwrap_or(0);

            if let Some((idx, _)) = state
                .players
                .iter()
                .enumerate()
                .find(|(_, p)| p.is_active && p.card_count == min_cards)
            {
                return Some(GameResult::Winner(idx));
            }
        }

        None
    }
}

/// Special card effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialEffect {
    /// No special effect
    None,
    /// Choose next suit (Whot card)
    ChooseShape,
    /// Play another card immediately (1 - Hold On)
    PlayAgain,
    /// Next player draws 2 cards (2 - Pick Two)
    DrawTwo,
    /// Next player draws 3 cards (5 - Pick Three)
    DrawThree,
    /// Skip next player's turn (8 - Suspension)
    SkipNext,
    /// All other players draw 1 card (14 - General Market)
    AllDrawOne,
}

/// Game result enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// Player at index won
    Winner(usize),
    /// Game ended in draw (future feature)
    #[allow(dead_code)]
    Draw,
}
