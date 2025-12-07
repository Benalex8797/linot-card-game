#![cfg_attr(target_arch = "wasm32", no_main)]

mod game_engine;
mod state;

use linera_sdk::{
    abi::WithContractAbi, linera_base_types::AccountOwner, views::View, Contract, ContractRuntime,
};

use crate::game_engine::{GameEngine, GameResult, SpecialEffect};
use crate::state::{LinotState, MatchData, MatchStatus, Player};
use linot::{CardSuit, GameEvent, LinotAbi, LinotError, MatchConfig, Message, Operation, GAME_EVENTS_STREAM, TURN_TIMEOUT_MICROS};

use linera_sdk::linera_base_types::StreamName;

pub struct LinotContract {
    state: LinotState,
    runtime: ContractRuntime<Self>,
}

linera_sdk::contract!(LinotContract);

impl WithContractAbi for LinotContract {
    type Abi = LinotAbi;
}

impl Contract for LinotContract {
    type Message = Message;
    type Parameters = ();
    type InstantiationArgument = MatchConfig;
    type EventValue = GameEvent;

    async fn load(runtime: ContractRuntime<Self>) -> Self {
        let state = LinotState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        LinotContract { state, runtime }
    }

    async fn instantiate(&mut self, mut config: Self::InstantiationArgument) {
        // Auto-detect host from authenticated signer if not provided
        if config.host.is_none() {
            config.host = self.runtime.authenticated_signer();
        }
        
        // Store match configuration
        self.state.config.set(config);

        // Initialize empty match data
        let mut match_data = MatchData::default();
        match_data.created_at = self.runtime.system_time().micros();
        match_data.status = MatchStatus::Waiting;

        self.state.match_data.set(match_data);

        // Initialize betting pool as None
        self.state.betting_pool.set(None);
    }

    async fn execute_operation(&mut self, operation: Self::Operation) -> Self::Response {
        let caller = self
            .runtime
            .authenticated_signer()
            .ok_or(LinotError::CallerRequired)
            .expect("Caller required");

        let result = match operation {
            Operation::JoinMatch { nickname } => {
                self.handle_join_match(caller, nickname).await
            }
            Operation::JoinFromChain { host_chain_id, nickname } => {
                self.handle_join_from_chain(host_chain_id, nickname).await
            }
            Operation::StartMatch => {
                self.handle_start_match(caller).await
            }
            Operation::PlayCard {
                card_index,
                chosen_suit,
            } => {
                self.handle_play_card(caller, card_index, chosen_suit).await
            }
            Operation::DrawCard => {
                self.handle_draw_card(caller).await
            }
            Operation::CallLastCard => {
                self.handle_call_last_card(caller).await
            }
            Operation::ChallengeLastCard { player_index } => {
                self.handle_challenge_last_card(caller, player_index).await
            }
            Operation::LeaveMatch => {
                self.handle_leave_match(caller).await
            }
            Operation::CheckTimeout => {
                self.handle_check_timeout().await
            }
            Operation::PlaceBet {
                player_index: _,
                amount: _,
            } => {
                Err(LinotError::BettingNotImplemented)
            }
        };

        // Panic on error to maintain existing behavior
        // TODO: Return Result from execute_operation in future
        if let Err(e) = result {
            panic!("Operation failed: {}", e);
        }
    }

    async fn execute_message(&mut self, message: Self::Message) {
        let result = match message {
            Message::JoinRequest { player, nickname } => {
                // Handle cross-chain join request
                let join_result = self.handle_join_match(player, nickname.clone()).await;
                
                if join_result.is_ok() {
                    // Emit event that player joined
                    let match_data = self.state.match_data.get();
                    let event = GameEvent::PlayerJoined {
                        nickname: nickname.clone(),
                        player_count: match_data.players.len(),
                    };
                    self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
                    
                    // Send initial state sync back to the joining player
                    // Note: In Linera, messages are sent to chains, not directly to AccountOwners
                    // For now, we'll skip sending the sync message since the player can query state via GraphQL
                    // In a full implementation, we'd track player chain IDs separately
                }
                
                join_result
            }
            Message::InitialStateSync { config: _, players: _, status: _ } => {
                // Player receives initial state from host
                // In V1, we don't need to process this on contract side
                // The frontend will handle this via GraphQL queries
                Ok(())
            }
            Message::GameEvent { event } => {
                // Broadcast game event to subscribers
                self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
                Ok(())
            }
        };

        // Silently handle message errors (don't crash the chain)
        if let Err(_e) = result {
            // In production, could emit error event
        }
    }

    async fn store(self) {
        // State changes are automatically persisted by the framework
        // No manual save needed when using Views
    }
}

impl LinotContract {
    /// Handle player joining the match
    async fn handle_join_match(&mut self, caller: AccountOwner, nickname: String) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();
        let config = self.state.config.get().clone();

        // Validate: match must be waiting
        if match_data.status != MatchStatus::Waiting {
            return Err(LinotError::MatchAlreadyStarted);
        }

        // Validate: not at max players
        if match_data.players.len() >= config.max_players as usize {
            return Err(LinotError::MatchFull(config.max_players));
        }

        // Validate: player not already joined
        if match_data.players.iter().any(|p| p.owner == caller) {
            return Err(LinotError::PlayerAlreadyJoined);
        }

        // Add player
        match_data.players.push(Player::new(caller, nickname.clone()));
        self.state.match_data.set(match_data.clone());
        
        // Emit event
        let event = GameEvent::PlayerJoined {
            nickname,
            player_count: match_data.players.len(),
        };
        self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        
        Ok(())
    }

    /// Handle cross-chain join request
    async fn handle_join_from_chain(&mut self, host_chain_id: String, nickname: String) -> Result<(), LinotError> {
        let caller = self.runtime.authenticated_signer().ok_or(LinotError::CallerRequired)?;
        
        // Parse host chain ID
        let host_chain = host_chain_id.parse()
            .map_err(|_| LinotError::CallerRequired)?; // Reuse error for simplicity
        
        // Send cross-chain join request message
        let message = Message::JoinRequest {
            player: caller,
            nickname,
        };
        
        self.runtime.send_message(host_chain, message);
        
        Ok(())
    }

    /// Handle starting the match
    async fn handle_start_match(&mut self, caller: AccountOwner) -> Result<(), LinotError> {
        let config = self.state.config.get().clone();
        let mut match_data = self.state.match_data.get().clone();

        // Validate: only host can start
        if let Some(host) = &config.host {
            if caller != *host {
                return Err(LinotError::OnlyHostCanStart);
            }
        } else {
            return Err(LinotError::OnlyHostCanStart);
        }

        // Validate: enough players (2 for V1)
        if match_data.players.len() < 2 {
            return Err(LinotError::NotEnoughPlayers(2));
        }

        // Validate: match is waiting
        if match_data.status != MatchStatus::Waiting {
            return Err(LinotError::MatchAlreadyStarted);
        }

        // Create and shuffle deck
        let mut deck = GameEngine::create_deck();
        let chain_id = self.runtime.chain_id();
        let seed = chain_id.to_string();
        GameEngine::shuffle_with_seed(&mut deck, seed.as_bytes());

        // Deal initial hands (6 cards each)
        let hands = GameEngine::deal_initial_hands(&mut deck, match_data.players.len());
        for (i, player) in match_data.players.iter_mut().enumerate() {
            player.cards = hands[i].clone();
            player.update_card_count();
        }

        // Place first card in discard pile
        if let Some(first_card) = deck.pop() {
            match_data.discard_pile.push(first_card);
        }

        // Update match state
        match_data.deck = deck;
        match_data.status = MatchStatus::InProgress;
        match_data.current_player_index = 0;
        match_data.turn_started_at = self.runtime.system_time().micros();

        let first_player = match_data.players[0].nickname.clone();
        let top_card = match_data.discard_pile.last().unwrap().clone();

        self.state.match_data.set(match_data);
        
        // Emit event
        let event = GameEvent::MatchStarted {
            first_player,
            top_card,
        };
        self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        
        Ok(())
    }

    /// Handle playing a card
    async fn handle_play_card(
        &mut self,
        caller: AccountOwner,
        card_index: usize,
        chosen_suit: Option<CardSuit>,
    ) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        // Validate: match is in progress
        if match_data.status != MatchStatus::InProgress {
            return Err(LinotError::MatchNotInProgress);
        }

        // Check if turn has timed out (3 minutes)
        let current_time = self.runtime.system_time().micros();
        let elapsed = current_time.saturating_sub(match_data.turn_started_at);
        if elapsed >= TURN_TIMEOUT_MICROS {
            return Err(LinotError::TurnTimeout);
        }

        // Validate: it's caller's turn
        let current_player = &mut match_data.players[match_data.current_player_index];
        if current_player.owner != caller {
            return Err(LinotError::NotYourTurn);
        }

        // Validate: card index is valid
        if card_index >= current_player.cards.len() {
            return Err(LinotError::InvalidCardIndex(card_index));
        }

        // Get the card
        let card = current_player.cards[card_index].clone();

        // Get top card from discard pile
        let top_card = match_data
            .discard_pile
            .last()
            .ok_or(LinotError::NoCardInDiscardPile)?;

        // Validate: card can be played
        if !GameEngine::is_valid_play(
            &card,
            top_card,
            match_data.active_shape_demand,
            match_data.pending_penalty,
            match_data.hold_on_active,
            match_data.hold_on_required_shape,
        ) {
            return Err(LinotError::InvalidCardPlay);
        }

        // Remove card from hand
        current_player.cards.remove(card_index);
        current_player.update_card_count();

        // Add to discard pile
        match_data.discard_pile.push(card.clone());

        // Check if player should call last card
        if current_player.card_count == 1 && !current_player.called_last_card {
            // Automatic last card call in V1
            current_player.called_last_card = true;
        }

        // Apply special card effect
        let effect = GameEngine::get_card_effect(&card);
        let card_suit = card.suit;
        GameEngine::apply_effect(&mut match_data, effect, chosen_suit, card_suit);

        // Check if game ended
        if let Some(result) = GameEngine::check_game_end(&match_data) {
            match result {
                GameResult::Winner(idx) => {
                    match_data.winner_index = Some(idx);
                    match_data.status = MatchStatus::Finished;
                    
                    // Emit game ended event
                    let winner = match_data.players[idx].nickname.clone();
                    let event = GameEvent::MatchEnded {
                        winner: winner.clone(),
                        winner_index: idx,
                    };
                    self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
                    
                    self.state.match_data.set(match_data);
                    return Ok(());
                }
                GameResult::Draw => {
                    match_data.status = MatchStatus::Finished;
                }
            }
        }

        // Handle General Market effect if needed
        if let SpecialEffect::AllDrawOne = effect {
            Self::apply_general_market(&mut match_data);
        }

        let current_player_name = match_data.players[match_data.current_player_index].nickname.clone();
        
        // Advance turn based on effect
        let skip_turn_advance = if effect == SpecialEffect::PlayAgain {
            // Hold On (1): Current player must play again, don't advance
            true
        } else {
            // Clear hold on state when playing second card
            if match_data.hold_on_active {
                match_data.hold_on_active = false;
                match_data.hold_on_required_shape = None;
            }
            
            if effect == SpecialEffect::SkipNext {
                // Suspension (8): Skip next player by advancing twice
                GameEngine::advance_turn(&mut match_data);
                GameEngine::advance_turn(&mut match_data);
            } else {
                // Normal: Advance to next player
                GameEngine::advance_turn(&mut match_data);
            }
            
            // Update turn timestamp
            match_data.turn_started_at = self.runtime.system_time().micros();
            
            false
        };

        let next_player = match_data.players[match_data.current_player_index].nickname.clone();

        self.state.match_data.set(match_data);
        
        // Emit card played event
        if !skip_turn_advance {
            let special_desc = match effect {
                SpecialEffect::ChooseShape => {
                    let suit_name = format!("{:?}", chosen_suit.unwrap_or(CardSuit::Circle));
                    Some(format!("Whot! Called {}", suit_name))
                },
                SpecialEffect::DrawTwo => Some("Pick Two".to_string()),
                SpecialEffect::DrawThree => Some("Pick Three".to_string()),
                SpecialEffect::SkipNext => Some("Suspension".to_string()),
                SpecialEffect::AllDrawOne => Some("General Market".to_string()),
                SpecialEffect::PlayAgain => Some("Hold On - Play Again".to_string()),
                SpecialEffect::None => None,
            };
            
            let event = GameEvent::CardPlayed {
                player: current_player_name,
                card,
                next_player,
                special_effect: special_desc,
            };
            self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        }
        
        Ok(())
    }

    /// Handle drawing a card
    async fn handle_draw_card(&mut self, caller: AccountOwner) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        // Check if turn has timed out (3 minutes)
        let current_time = self.runtime.system_time().micros();
        let elapsed = current_time.saturating_sub(match_data.turn_started_at);
        if elapsed >= TURN_TIMEOUT_MICROS {
            return Err(LinotError::TurnTimeout);
        }

        // Validate: it's caller's turn
        let current_player_idx = match_data.current_player_index;
        let current_player = &mut match_data.players[current_player_idx];
        if current_player.owner != caller {
            return Err(LinotError::NotYourTurn);
        }

        // Determine how many cards to draw
        let cards_to_draw = if match_data.pending_penalty > 0 {
            let count = match_data.pending_penalty;
            match_data.pending_penalty = 0;
            count
        } else {
            1
        };

        // Draw cards
        for _ in 0..cards_to_draw {
            if match_data.deck.is_empty() {
                // Reshuffle discard pile (except top card)
                if match_data.discard_pile.len() > 1 {
                    let top_card = match_data.discard_pile.pop().unwrap();
                    match_data.deck = match_data.discard_pile.clone();
                    match_data.discard_pile.clear();
                    match_data.discard_pile.push(top_card);

                    // Reshuffle with new seed
                    match_data.round_number += 1;
                    let seed = format!("{}{}", self.runtime.chain_id(), match_data.round_number);
                    GameEngine::shuffle_with_seed(&mut match_data.deck, seed.as_bytes());
                } else {
                    break; // No more cards available
                }
            }

            if let Some(card) = match_data.deck.pop() {
                current_player.cards.push(card);
            }
        }

        current_player.update_card_count();

        // Clear active shape demand after drawing
        match_data.active_shape_demand = None;
        
        let player_name = current_player.nickname.clone();

        // Advance turn
        GameEngine::advance_turn(&mut match_data);
        
        // Update turn timestamp
        match_data.turn_started_at = self.runtime.system_time().micros();
        
        let next_player = match_data.players[match_data.current_player_index].nickname.clone();

        self.state.match_data.set(match_data);
        
        // Emit event
        let event = GameEvent::CardsDrawn {
            player: player_name,
            count: cards_to_draw,
            next_player,
        };
        self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        
        Ok(())
    }

    /// Handle calling last card
    async fn handle_call_last_card(&mut self, caller: AccountOwner) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        if let Some(player) = match_data.players.iter_mut().find(|p| p.owner == caller) {
            player.called_last_card = true;
        }

        self.state.match_data.set(match_data);
        
        Ok(())
    }

    /// Handle challenging a player who didn't call last card
    async fn handle_challenge_last_card(&mut self, _caller: AccountOwner, player_index: usize) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        // Validate player index
        if player_index >= match_data.players.len() {
            return Err(LinotError::InvalidPlayerIndex(player_index));
        }

        let player = &mut match_data.players[player_index];

        // If player has 1 card and didn't call last card, penalty
        if player.card_count == 1 && !player.called_last_card {
            // Draw 2 cards as penalty
            for _ in 0..2 {
                if let Some(card) = match_data.deck.pop() {
                    player.cards.push(card);
                }
            }
            player.update_card_count();
        }

        self.state.match_data.set(match_data);
        
        Ok(())
    }

    /// Handle player leaving/forfeiting
    async fn handle_leave_match(&mut self, caller: AccountOwner) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        // Find player and mark as inactive
        let player_nickname = match_data.players.iter()
            .find(|p| p.owner == caller)
            .map(|p| p.nickname.clone());
        
        if let Some(player) = match_data.players.iter_mut().find(|p| p.owner == caller) {
            player.is_active = false;
        }

        // Check if only one active player left
        let active_players: Vec<_> = match_data.players.iter().filter(|p| p.is_active).collect();
        if active_players.len() == 1 {
            // Remaining player wins
            let winner_idx = match_data.players.iter().position(|p| p.is_active).unwrap();
            match_data.winner_index = Some(winner_idx);
            match_data.status = MatchStatus::Finished;
            
            // Emit game ended event
            let winner = match_data.players[winner_idx].nickname.clone();
            let event = GameEvent::MatchEnded {
                winner: winner.clone(),
                winner_index: winner_idx,
            };
            self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        }

        self.state.match_data.set(match_data);
        
        // Emit player left event
        if let Some(nickname) = player_nickname {
            let event = GameEvent::PlayerLeft { nickname };
            self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);
        }
        
        Ok(())
    }

    /// Check if current player has exceeded 3-minute timeout
    /// If yes, automatically draw a card for them and advance turn
    /// Anyone can call this to enforce fair play
    async fn handle_check_timeout(&mut self) -> Result<(), LinotError> {
        let mut match_data = self.state.match_data.get().clone();

        // Only check timeout during active match
        if match_data.status != MatchStatus::InProgress {
            return Ok(()); // Silently ignore if not in progress
        }

        // Get current time
        let current_time = self.runtime.system_time().micros();
        let elapsed = current_time.saturating_sub(match_data.turn_started_at);

        // Check if 3 minutes have passed
        if elapsed < linot::TURN_TIMEOUT_MICROS {
            return Ok(()); // No timeout yet
        }

        // Timeout occurred! Auto-draw card for current player
        let current_player_idx = match_data.current_player_index;
        let current_player = &mut match_data.players[current_player_idx];
        let player_name = current_player.nickname.clone();

        // Draw one card as penalty
        let mut drawn = false;
        if !match_data.deck.is_empty() {
            if let Some(card) = match_data.deck.pop() {
                current_player.cards.push(card);
                current_player.update_card_count();
                drawn = true;
            }
        } else {
            // Deck empty - reshuffle if possible
            if match_data.discard_pile.len() > 1 {
                let top_card = match_data.discard_pile.pop().unwrap();
                match_data.deck = match_data.discard_pile.clone();
                match_data.discard_pile.clear();
                match_data.discard_pile.push(top_card);

                match_data.round_number += 1;
                let seed = format!("{}{}", self.runtime.chain_id(), match_data.round_number);
                GameEngine::shuffle_with_seed(&mut match_data.deck, seed.as_bytes());

                if let Some(card) = match_data.deck.pop() {
                    current_player.cards.push(card);
                    current_player.update_card_count();
                    drawn = true;
                }
            }
        }

        // Clear any active states
        match_data.active_shape_demand = None;
        match_data.hold_on_active = false;
        match_data.hold_on_required_shape = None;

        // Advance to next player
        GameEngine::advance_turn(&mut match_data);
        match_data.turn_started_at = current_time; // Reset timer for next player

        self.state.match_data.set(match_data);

        // Emit timeout event
        let event = GameEvent::TurnTimeout {
            player: player_name,
            auto_drawn: drawn,
        };
        self.runtime.emit(StreamName::from(GAME_EVENTS_STREAM), &event);

        Ok(())
    }

    /// Apply General Market effect (all other players draw 1)
    fn apply_general_market(match_data: &mut MatchData) {
        let current_idx = match_data.current_player_index;
        for (i, player) in match_data.players.iter_mut().enumerate() {
            if i != current_idx && !match_data.deck.is_empty() {
                if let Some(card) = match_data.deck.pop() {
                    player.cards.push(card);
                    player.update_card_count();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added in future iteration
}
