use async_graphql::{Request, Response};
use linera_sdk::{
    abi::{ContractAbi, ServiceAbi},
    graphql::GraphQLMutationRoot,
    linera_base_types::AccountOwner,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============ Constants ============

/// Stream name for game events (multiplayer synchronization)
pub const GAME_EVENTS_STREAM: &[u8] = b"linot_game_events";

/// Turn timeout in microseconds (3 minutes)
pub const TURN_TIMEOUT_MICROS: u64 = 3 * 60 * 1_000_000; // 180 seconds

/// Turn warning in microseconds (2 minutes)
pub const TURN_WARNING_MICROS: u64 = 2 * 60 * 1_000_000; // 120 seconds

// ============ Error Types ============

#[derive(Debug, Error)]
pub enum LinotError {
    #[error("Match already started")]
    MatchAlreadyStarted,
    
    #[error("Match not started")]
    MatchNotStarted,
    
    #[error("Match is full (max {0} players)")]
    MatchFull(u8),
    
    #[error("Player already joined")]
    PlayerAlreadyJoined,
    
    #[error("Only host can start match")]
    OnlyHostCanStart,
    
    #[error("Need at least {0} players to start")]
    NotEnoughPlayers(usize),
    
    #[error("Not your turn")]
    NotYourTurn,
    
    #[error("Invalid card index: {0}")]
    InvalidCardIndex(usize),
    
    #[error("Invalid card play: card doesn't match suit, value, or special requirements")]
    InvalidCardPlay,
    
    #[error("Invalid player index: {0}")]
    InvalidPlayerIndex(usize),
    
    #[error("Match not in progress")]
    MatchNotInProgress,
    
    #[error("No card in discard pile")]
    NoCardInDiscardPile,
    
    #[error("Betting not implemented yet")]
    BettingNotImplemented,
    
    #[error("Caller authentication required")]
    CallerRequired,
    
    #[error("Cannot play Hold On (1) without a second card")]
    HoldOnRequiresSecondCard,
    
    #[error("Must play same shape after Hold On")]
    HoldOnShapeMismatch,
    
    #[error("Cannot stack penalties - different card types")]
    CannotStackPenalties,
    
    #[error("Turn timeout - player took too long")]
    TurnTimeout,
}

// ============ ABI Definition ============

pub struct LinotAbi;

impl ContractAbi for LinotAbi {
    type Operation = Operation;
    type Response = ();
}

impl ServiceAbi for LinotAbi {
    type Query = Request;
    type QueryResponse = Response;
}

// ============ Data Types ============

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct Card {
    pub suit: CardSuit,
    pub value: CardValue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, async_graphql::Enum, PartialEq, Eq, Hash)]
pub enum CardSuit {
    Circle,
    Cross,
    Triangle,
    Square,
    Star,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, async_graphql::Enum, PartialEq, Eq)]
pub enum CardValue {
    One,      // 1 - Hold On (special)
    Two,      // 2 - Pick Two (special)
    Three,    // 3
    Four,     // 4
    Five,     // 5 - Pick Three (special)
    Six,      // 6
    Seven,    // 7
    Eight,    // 8 - Suspension (special)
    Nine,     // 9
    Ten,      // 10
    Eleven,   // 11
    Twelve,   // 12
    Thirteen, // 13
    Fourteen, // 14 - General Market (special)
    Whot,     // 20 - Wild card (special)
}

impl CardValue {
    /// Get the numeric value for display/logic
    pub fn to_number(&self) -> u8 {
        match self {
            CardValue::One => 1,
            CardValue::Two => 2,
            CardValue::Three => 3,
            CardValue::Four => 4,
            CardValue::Five => 5,
            CardValue::Six => 6,
            CardValue::Seven => 7,
            CardValue::Eight => 8,
            CardValue::Nine => 9,
            CardValue::Ten => 10,
            CardValue::Eleven => 11,
            CardValue::Twelve => 12,
            CardValue::Thirteen => 13,
            CardValue::Fourteen => 14,
            CardValue::Whot => 20,
        }
    }

    /// Check if this is a special action card
    pub fn is_special(&self) -> bool {
        matches!(self, 
            CardValue::One | CardValue::Two | CardValue::Five | 
            CardValue::Eight | CardValue::Fourteen | CardValue::Whot
        )
    }
}

// ============ Match Configuration ============

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::SimpleObject)]
pub struct MatchConfig {
    pub max_players: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<AccountOwner>,
    pub is_ranked: bool,
    pub strict_mode: bool, // Enforce must draw if no valid move
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            max_players: 2,
            host: None,
            is_ranked: false,
            strict_mode: false,
        }
    }
}

// ============ Operations (GraphQL Mutations) ============

#[derive(Debug, Deserialize, Serialize, GraphQLMutationRoot)]
pub enum Operation {
    /// Join this match instance (local chain)
    JoinMatch {
        nickname: String,
    },

    /// Join a match on another chain (cross-chain)
    JoinFromChain {
        host_chain_id: String,
        nickname: String,
    },

    /// Start the match (host only)
    StartMatch,

    /// Play a card from your hand
    PlayCard {
        card_index: usize,
        chosen_suit: Option<CardSuit>, // Required for Whot card
    },

    /// Draw a card from the deck (when stuck or choosing to draw)
    DrawCard,

    /// Call Last Card when you have exactly 1 card
    CallLastCard,

    /// Challenge someone who forgot to call Last Card
    ChallengeLastCard {
        player_index: usize,
    },

    /// Leave the match (forfeit)
    LeaveMatch,

    /// Check if current player has timed out (anyone can call)
    CheckTimeout,

    // Wave 4-5: Betting (placeholder)
    PlaceBet {
        player_index: usize,
        amount: u64,
    },
}

// ============ Messages (Cross-Chain Communication) ============

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    /// Join request from player on another chain
    JoinRequest {
        player: AccountOwner,
        nickname: String,
    },

    /// Sync initial state to newly joined player
    InitialStateSync {
        config: MatchConfig,
        players: Vec<String>, // player nicknames
        status: u8, // MatchStatus as u8
    },

    /// Broadcast game event to all players
    GameEvent {
        event: GameEvent,
    },
}

// ============ Events (For Stream Emission) ============

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GameEvent {
    /// Player joined the match
    PlayerJoined {
        nickname: String,
        player_count: usize,
    },

    /// Match has started
    MatchStarted {
        first_player: String,
        top_card: Card,
    },

    /// Card was played
    CardPlayed {
        player: String,
        card: Card,
        next_player: String,
        special_effect: Option<String>,
    },

    /// Cards were drawn
    CardsDrawn {
        player: String,
        count: u8,
        next_player: String,
    },

    /// Match ended
    MatchEnded {
        winner: String,
        winner_index: usize,
    },

    /// Player left/forfeited
    PlayerLeft {
        nickname: String,
    },

    /// Turn timeout warning (2 minutes elapsed)
    TurnTimeoutWarning {
        player: String,
    },

    /// Turn timeout (3 minutes elapsed, auto-draw)
    TurnTimeout {
        player: String,
        auto_drawn: bool,
    },
}
