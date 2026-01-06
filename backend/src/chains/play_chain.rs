use linera_sdk::linera_base_types::{AccountOwner, ChainId};
use linot::{
    Card, CardSuit, GameEvent, LinotResponse, MatchStatus, Message, Player,
    SpecialEffect, UserStatus, GAME_STREAM_NAME, INITIAL_HAND_SIZE, MAX_PLAYERS, MIN_PLAYERS,
};

use super::super::LinotContract;

impl LinotContract {
    // ================================================================================
    // PLAY_CHAIN OPERATIONS (Mutations executed on PLAY_CHAIN)
    // ================================================================================

    /// Create a new match on PLAY_CHAIN
    pub async fn handle_create_match(&mut self, max_players: u8, nickname: String) -> LinotResponse {
        // Validate
        if max_players < MIN_PLAYERS || max_players > MAX_PLAYERS {
            return LinotResponse::Error(format!(
                "Invalid player count. Must be between {} and {}",
                MIN_PLAYERS, MAX_PLAYERS
            ));
        }

        // Create match data
        let match_data = self.create_match(max_players, nickname.clone());
        
        // Set state
        self.state.match_data.set(match_data);
        self.state.user_status.set(UserStatus::InMatch);
        self.state.player_nickname.set(Some(nickname.clone()));

        // Emit event - get chain_id first to avoid borrow conflict
        let chain_id = self.runtime.chain_id();
        self.emit_event(GameEvent::MatchCreated {
            match_id: chain_id,
            host: nickname,
            max_players,
        });

        log::info!("PLAY_CHAIN: Match created with {} max players", max_players);
        LinotResponse::Ok
    }

    /// Create match on PLAY_CHAIN from cross-chain message
    /// This is called when USER_CHAIN sends RequestCreateMatch message
    pub async fn create_match_on_play_chain(
        &mut self,
        creator_owner: AccountOwner,
        creator_chain: ChainId,
        max_players: u8,
        nickname: String,
    ) -> Result<usize, String> {
        // Validate
        if max_players < MIN_PLAYERS || max_players > MAX_PLAYERS {
            return Err(format!(
                "Invalid player count. Must be between {} and {}",
                MIN_PLAYERS, MAX_PLAYERS
            ));
        }

        // Create match using manual initialization
        let mut match_data = crate::MatchData {
            players: vec![None; max_players as usize],
            deck: Vec::new(),
            deck_size: 0,
            discard_pile: Vec::new(),
            current_player_index: 0,
            status: MatchStatus::Waiting,
            max_players,
            winner_index: None,
            pending_draw_stack: 0,
            pending_draw_type: None,
            active_demand_suit: None,
            turn_start_time: None,
            turn_duration: 0,
        };
        
        // Set creator as first player with their chain info
        match_data.players[0] = Some(Player {
            owner: creator_owner,
            chain_id: creator_chain,
            nickname: nickname.clone(),
            hand: Vec::new(),
            hand_size: 0,
            called_last_card: false,
        });

        // Create and shuffle deck using Fisher-Yates
        let mut deck = Vec::with_capacity(61);
        
        // Regular suits (Circle, Triangle, Cross, Square) - values 1 to 14
        for suit in [CardSuit::Circle, CardSuit::Triangle, CardSuit::Cross, CardSuit::Square] {
            for value in 1..=14 {
                deck.push(Card { suit, value });
            }
        }
        
        // Star suit - values 1 to 7
        for value in 1..=7 {
            deck.push(Card { suit: CardSuit::Star, value });
        }
        
        // Whot cards (wild) - 5 total
        for _ in 0..5 {
            deck.push(Card { suit: CardSuit::Whot, value: 20 });
        }

        // Fisher-Yates shuffle
        let seed = self.runtime.system_time().micros();
        let mut state = seed;
        for i in (1..deck.len()).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (state as usize) % (i + 1);
            deck.swap(i, j);
        }

        match_data.deck = deck;
        match_data.deck_size = 61;
        match_data.status = MatchStatus::Waiting;
        
        // Set state on PLAY_CHAIN
        self.state.match_data.set(match_data);

        // Emit events
        let match_id = self.runtime.chain_id();
        self.emit_event(GameEvent::MatchCreated {
            match_id,
            host: nickname.clone(),
            max_players,
        });

        log::info!("PLAY_CHAIN: Match created via cross-chain message from {:?}", creator_chain);
        Ok(0) // Return match ID
    }

    /// Start the match (deal cards, set first player)
    pub async fn handle_start_match(&mut self) -> LinotResponse {
        let match_data = self.state.match_data.get_mut();

        // Validate enough players
        let player_count = match_data.players.iter()
            .filter(|p| p.is_some()).count();
        
        log::info!("PLAY_CHAIN: handle_start_match called. Player count: {}", player_count);

        if player_count < MIN_PLAYERS as usize {
            let msg = format!(
                "Not enough players. Need at least {}. Current: {}",
                MIN_PLAYERS, player_count
            );
            log::error!("PLAY_CHAIN: StartMatch failed - {}", msg);
            return LinotResponse::Error(msg);
        }

        // Deal cards inline to avoid borrow conflict
        for player in match_data.players.iter_mut().filter_map(|p| p.as_mut()) {
            for _ in 0..INITIAL_HAND_SIZE {
                if let Some(card) = match_data.deck.pop() {
                    player.hand.push(card);
                }
            }
            player.update_hand_size();
        }
        
        // Add first card to discard pile
        if let Some(card) = match_data.deck.pop() {
match_data.discard_pile.push(card);
        }
        
        match_data.deck_size = match_data.deck.len();

        // Set first player
        match_data.current_player_index = 0;
        match_data.status = MatchStatus::InProgress;
        
        // Initialize turn timer
        let start_time = self.runtime.system_time().micros();
        match_data.turn_start_time = Some(start_time);

        // Get first player and top card
        let first_player = match_data.players[0].as_ref()
            .map(|p| p.nickname.clone())
            .unwrap_or_default();
        let top_card = match_data.discard_pile.last().cloned()
            .unwrap_or(Card { suit: CardSuit::Circle, value: 1 });

        // Emit event
        let player_names: Vec<String> = match_data.players.iter()
            .filter_map(|p| p.as_ref().map(|player| player.nickname.clone()))
            .collect();
        
        // Borrow ends here

        self.emit_event(GameEvent::MatchStarted {
            players: player_names,
            first_player: first_player.clone(),
            top_card,
        });
        
        // Emit turn started event
        self.emit_event(GameEvent::TurnStarted {
            player_nickname: first_player,
            duration_micros: linot::TURN_TIMEOUT_MICROS,
        });

        log::info!("PLAY_CHAIN: Match started with {} players", player_count);
        LinotResponse::Ok
    }

    // ================================================================================
    // MESSAGE HANDLERS (Process cross-chain messages from USER_CHAINs)
    // ================================================================================

    /// Handle join request from USER_CHAIN
    pub async fn handle_request_join_message(
        &mut self,
        player_owner: AccountOwner,
        player_chain: ChainId,
        nickname: String
    ) {
        let origin_chain = self.runtime.message_origin_chain_id()
            .expect("Message has no origin");
        let play_chain_id = self.runtime.chain_id();

        let match_data = self.state.match_data.get_mut();

        // Find empty slot
        if let Some(index) = match_data.players.iter().position(|p| p.is_none()) {
            let player = Player::new(player_chain, player_owner, nickname.clone());
            match_data.players[index] = Some(player);

            let player_count = match_data.players.iter()
                .filter(|p| p.is_some()).count();

            // Send confirmation back to USER_CHAIN (triggers subscribe!)
            let confirmation = Message::JoinMatchConfirmed {
                play_chain_id,
                success: true,
            };
            self.runtime.prepare_message(confirmation).send_to(player_chain);

            // Emit event
            self.emit_event(GameEvent::PlayerJoined {
                nickname,
                player_count,
            });

            log::info!("PLAY_CHAIN: Player joined from chain: {:?}, sent confirmation", origin_chain);
        } else {
            // Send failure confirmation
            let confirmation = Message::JoinMatchConfirmed {
                play_chain_id,
                success: false,
            };
            self.runtime.prepare_message(confirmation).send_to(player_chain);

            log::warn!("PLAY_CHAIN: No empty slots for player to join");
        }
    }

    /// Handle play card action from USER_CHAIN
    pub async fn handle_play_card_message(
        &mut self,
        player_owner: AccountOwner,
        card_index: usize,
        chosen_suit: Option<CardSuit>
    ) {
        let match_data = self.state.match_data.get_mut();

        // Check if player has pending penalty cards
        // They must handle the penalty BEFORE playing (unless they're blocking with same card type)
        if match_data.pending_draw_stack > 0 {
            let current_idx = match_data.current_player_index;
            
            if let Some(current_player) = match_data.players[current_idx].as_ref() {
                // Check if the card they're trying to play can block the penalty
                if card_index < current_player.hand.len() {
                    let card = &current_player.hand[card_index];
                    let can_block = match match_data.pending_draw_type {
                        Some(2) => card.value == 2,  // Can block Pick 2 with another Pick 2
                        Some(5) => card.value == 5 && card.suit != CardSuit::Star,  // Can block Pick 3 with another Pick 3 (not Star 5)
                        _ => false,
                    };
                    
                    if !can_block {
                        // Cannot block - must draw penalty cards first
                        let draw_count = match_data.pending_draw_stack;
                        let current_player = match_data.players[current_idx].as_mut().unwrap();
                        
                        log::info!("PLAY_CHAIN: {} must draw {} penalty cards before playing", current_player.nickname, draw_count);
                        
                        // Draw all penalty cards
                        for _ in 0..draw_count {
                            if match_data.deck.is_empty() {
                                log::info!("PLAY_CHAIN: Deck empty during penalty draw");
                                break;
                            }
                            
                            if let Some(card) = match_data.deck.pop() {
                                current_player.hand.push(card);
                            }
                        }
                        
                        current_player.update_hand_size();
                        match_data.deck_size = match_data.deck.len();
                        
                        // Reset penalty
                        match_data.pending_draw_stack = 0;
                        match_data.pending_draw_type = None;
                        
                        log::info!("PLAY_CHAIN: Penalty applied, player can now continue their turn");
                        // DON'T return here - let them play their card
                    }
                }
            }
        }

        // Find player and verify it's their turn
        let current_player_opt = match_data.players[match_data.current_player_index].as_mut();
        
        if let Some(current_player) = current_player_opt {
            if current_player.owner != player_owner {
                log::warn!("PLAY_CHAIN: Not the current player's turn");
                return;
            }

            // Validate card index
            if card_index >= current_player.hand.len() {
                log::warn!("PLAY_CHAIN: Invalid card index");
                return;
            }

            let card = current_player.hand[card_index].clone();
            
            // Validate play - check against active demand suit or top card
            let is_valid = if let Some(demand_suit) = match_data.active_demand_suit {
                // WHOT was played - match against chosen suit
                card.value == 20  // WHOT can always be played
                    || card.suit == demand_suit
            } else if let Some(top) = match_data.discard_pile.last() {
                // Normal validation against top card
                card.value == 20  // Whot card
                    || card.suit == top.suit
                    || card.value == top.value
            } else {
                // First card - always valid
                true
            };
            
            if !is_valid {
                log::warn!("PLAY_CHAIN: Invalid card play");
                return;
            }

            // Remove card from hand
            current_player.hand.remove(card_index);
            current_player.update_hand_size();

            // Add to discard pile
            match_data.discard_pile.push(card.clone());

            // Check win condition
            if current_player.hand.is_empty() {
                match_data.status = MatchStatus::Finished;
                match_data.winner_index = Some(match_data.current_player_index);
                
                let winner_name = current_player.nickname.clone();
                let winner_idx = match_data.current_player_index;
                
                // Borrow ends here

                self.emit_event(GameEvent::MatchEnded {
                    winner: winner_name,
                    winner_index: winner_idx,
                });
                return;
            }

            // Get special effect - inline to avoid borrow conflicts
            let special_effect = match card.value {
                2 => Some(SpecialEffect::PickTwo),
                5 if card.suit != CardSuit::Star => Some(SpecialEffect::PickThree),
                1 => Some(SpecialEffect::HoldOn),
                14 => Some(SpecialEffect::GeneralMarket),
                20 => chosen_suit.map(|suit| SpecialEffect::WhotPlayed { chosen_suit: suit }),
                _ => None,
            };
            
            // Update active demand suit
            if card.value == 20 {
                // WHOT played - set demand suit
                match_data.active_demand_suit = chosen_suit;
                log::info!("PLAY_CHAIN: WHOT played, demand suit set to {:?}", chosen_suit);
            } else {
                // Regular card - clear demand suit
                match_data.active_demand_suit = None;
            }

            // Collect data BEFORE advancing turn
            let current_index = match_data.current_player_index;
            let current_nickname = current_player.nickname.clone();
            
            // Calculate next player manually to avoid borrow conflicts
            let mut next_index = (current_index + 1) % match_data.players.len();
            while match_data.players[next_index].is_none() {
                next_index = (next_index + 1) % match_data.players.len();
            }
            
            let next_player_name = match_data.players[next_index].as_ref()
                .map(|p| p.nickname.clone())
                .unwrap_or_default();
            
            match_data.current_player_index = next_index;
            
            // Reset turn timer
            match_data.turn_start_time = Some(self.runtime.system_time().micros());
            
            // Borrow ends here

            // Emit event
            self.emit_event(GameEvent::CardPlayed {
                player_nickname: current_nickname.clone(),
                card: card.clone(),
                next_player: next_player_name,
                special_effect: special_effect.clone(),
            });

            // NOW EXECUTE THE SPECIAL EFFECT
            // Pass the index of the player who played the card for effects that need it (like General Market)
            self.execute_special_effect(special_effect, current_index);

            log::info!("PLAY_CHAIN: Card played successfully");
        }
    }

    /// Handle draw card action from USER_CHAIN
    pub async fn handle_draw_card_message(&mut self, player_owner: AccountOwner) {
        let match_data = self.state.match_data.get_mut();

        let current_player_opt = match_data.players[match_data.current_player_index].as_mut();
        
        if let Some(current_player) = current_player_opt {
            if current_player.owner != player_owner {
                log::warn!("PLAY_CHAIN: Not the current player's turn");
                return;
            }

            // Check if there's a pending penalty
            let penalty_count = if match_data.pending_draw_stack > 0 {
                let count = match_data.pending_draw_stack;
                log::info!("PLAY_CHAIN: Player drawing with pending penalty of {}", count);
                
                // Reset penalty
                match_data.pending_draw_stack = 0;
                match_data.pending_draw_type = None;
                
                count
            } else {
                0
            };

            // Check if deck is empty - check if player has any valid cards to play
            if match_data.deck.is_empty() {
                log::info!("PLAY_CHAIN: Deck is empty, checking if player can play any card");
                
                // Check if current player has any valid card to play
                let top_card = match_data.discard_pile.last();
                let demand_suit = match_data.active_demand_suit;
                
                let can_play = current_player.hand.iter().any(|card| {
                    if let Some(demand_suit) = demand_suit {
                        // WHOT was played - check against chosen suit
                        card.value == 20 || card.suit == demand_suit
                    } else if let Some(top) = top_card {
                        // Normal validation
                        card.value == 20 || card.suit == top.suit || card.value == top.value
                    } else {
                        // No top card - any card valid
                        true
                    }
                });
                
                if !can_play {
                    log::info!("PLAY_CHAIN: Deck empty and player has no valid cards - ending game");
                    
                    // Count cards for all players
                    let mut card_counts: Vec<(usize, usize)> = Vec::new();
                    for (idx, player_opt) in match_data.players.iter().enumerate() {
                        if let Some(player) = player_opt {
                            card_counts.push((idx, player.hand.len()));
                        }
                    }
                    
                    // Find minimum card count
                    let min_count = card_counts.iter().map(|(_, count)| *count).min().unwrap_or(0);
                    
                    // Find all players with minimum count (for draw detection)
                    let winners: Vec<usize> = card_counts.iter()
                        .filter(|(_, count)| *count == min_count)
                        .map(|(idx, _)| *idx)
                        .collect();
                    
                    if winners.len() > 1 {
                        // DRAW - multiple players with same card count
                        match_data.status = MatchStatus::Finished;
                        match_data.winner_index = None;
                        log::info!("PLAY_CHAIN: Game ended in DRAW - {} players with {} cards each", 
                                   winners.len(), min_count);
                        
                        self.emit_event(GameEvent::MatchEnded {
                            winner: "Draw".to_string(),
                            winner_index: usize::MAX, // Special value for draw
                        });
                    } else if let Some(&winner_idx) = winners.first() {
                        // Single winner - player with fewest cards
                        match_data.status = MatchStatus::Finished;
                        match_data.winner_index = Some(winner_idx);
                        
                        let winner_name = match_data.players[winner_idx].as_ref()
                            .map(|p| p.nickname.clone())
                            .unwrap_or_default();
                        
                        log::info!("PLAY_CHAIN: Game ended - {} wins with {} cards", 
                                   winner_name, min_count);
                        
                        self.emit_event(GameEvent::MatchEnded {
                            winner: winner_name,
                            winner_index: winner_idx,
                        });
                    }
                    
                    return;
                }
                
                log::warn!("PLAY_CHAIN: Deck empty but player has valid cards to play - cannot draw");
                return;
            }
            
            // Draw cards: 1 (manual) + penalty_count
            let total_draw = 1 + penalty_count;
            let mut actually_drawn = 0;
            
            for _ in 0..total_draw {
                if match_data.deck.is_empty() {
                    log::info!("PLAY_CHAIN: Deck empty after drawing {} cards", actually_drawn);
                    break;
                }
                
                if let Some(card) = match_data.deck.pop() {
                    current_player.hand.push(card);
                    actually_drawn += 1;
                }
            }
            
            if actually_drawn == 0 {
                log::warn!("PLAY_CHAIN: Cannot draw - deck empty");
                return;
            }
            
            current_player.update_hand_size();
            current_player.called_last_card = false;  // Reset flag after drawing
            match_data.deck_size = match_data.deck.len();

            log::info!("PLAY_CHAIN: Player drew {} card(s) (1 manual{}) ", 
                      actually_drawn,
                      if penalty_count > 0 { format!(" + {} penalty", penalty_count) } else { String::new() });

            // Collect data before advancing
            let current_index = match_data.current_player_index;
            let current_nickname = current_player.nickname.clone();
            
            // Calculate next player inline to avoid borrow conflicts
            let mut next_index = (current_index + 1) % match_data.players.len();
            while match_data.players[next_index].is_none() {
                next_index = (next_index + 1) % match_data.players.len();
            }
            
            let next_player_name = match_data.players[next_index].as_ref()
                .map(|p| p.nickname.clone())
                .unwrap_or_default();
            
            match_data.current_player_index = next_index;
            
            // Reset turn timer
            match_data.turn_start_time = Some(self.runtime.system_time().micros());
            
            // Borrow ends here

            // Emit event - always report as 1 for frontend consistency (penalty is separate)
            self.emit_event(GameEvent::CardsDrawn {
                player_nickname: current_nickname,
                count: actually_drawn,
                next_player: next_player_name,
            });

        }
    }

    /// Handle call last card action
    pub async fn handle_call_last_card_message(&mut self, player_owner: AccountOwner) {
        let match_data = self.state.match_data.get_mut();

        // Find player by owner and set flag
        let mut player_nickname_opt = None;
        for player_opt in match_data.players.iter_mut() {
            if let Some(player) = player_opt {
                if player.owner == player_owner {
                    player.called_last_card = true;
                    player_nickname_opt = Some(player.nickname.clone());
                    break;
                }
            }
        }
        
        // Borrow ends here
        
        // Emit event
        if let Some(nickname) = player_nickname_opt {
            let log_nickname = nickname.clone();
            self.emit_event(GameEvent::LastCardCalled {
                player_nickname: nickname,
            });
            log::info!("PLAY_CHAIN: {} called last card", log_nickname);
        }
    }

    /// Handle challenge last card action
    pub async fn handle_challenge_last_card_message(
        &mut self,
        _challenger_owner: AccountOwner,
        challenged_player_index: usize
    ) {
        let match_data = self.state.match_data.get_mut();
        
        if challenged_player_index >= match_data.players.len() {
            log::warn!("PLAY_CHAIN: Invalid player index for challenge");
            return;
        }
        
        if let Some(challenged_player) = match_data.players[challenged_player_index].as_mut() {
            // Check if player has exactly 1 card and DIDN'T call last card
            if challenged_player.hand.len() == 1 && !challenged_player.called_last_card {
                let player_nickname = challenged_player.nickname.clone();
                
                // PENALTY: Draw 2 cards (or as many as available)
                for _ in 0..2 {
                    // Check if deck is empty
                    if match_data.deck.is_empty() {
                        log::info!("PLAY_CHAIN: Deck empty during challenge penalty");
                        challenged_player.update_hand_size();
                        match_data.deck_size = 0;
                        break;
                    }
                    
                    if let Some(card) = match_data.deck.pop() {
                        challenged_player.hand.push(card);
                    }
                }
                
                challenged_player.update_hand_size();
                match_data.deck_size = match_data.deck.len();
                
                // Borrow ends here
                
                self.emit_event(GameEvent::ChallengePenalty {
                    challenged_player: player_nickname.clone(),
                    penalty_cards: 2,
                });
                
                log::info!("PLAY_CHAIN: Challenge successful - {} draws 2 cards", player_nickname);
            } else {
                log::info!("PLAY_CHAIN: Challenge failed - player called last card or doesn't have 1 card");
            }
        }
    }

    /// Check if current turn has timed out
    pub async fn handle_check_timeout(&mut self) {
        let match_data = self.state.match_data.get();
        
        if match_data.status != MatchStatus::InProgress {
            return;
        }
        
        if let Some(start_time) = match_data.turn_start_time {
            let current_time = self.runtime.system_time().micros();
            let elapsed = current_time - start_time;
            
            if elapsed > linot::TURN_TIMEOUT_MICROS {
                // Timeout - auto-draw and advance turn
                let _ = match_data;
                self.handle_turn_timeout().await;
            } else if elapsed > linot::TURN_WARNING_MICROS {
                // Warning - time running out
                let player_nickname = match_data.players[match_data.current_player_index]
                    .as_ref()
                    .map(|p| p.nickname.clone())
                    .unwrap_or_default();
                let time_left = linot::TURN_TIMEOUT_MICROS - elapsed;
                let _ = match_data;
                
                self.emit_event(GameEvent::TurnWarning {
                    player_nickname,
                    time_left_micros: time_left,
                });
            }
        }
    }
    
    /// Handle turn timeout - auto-draw and advance turn
    async fn handle_turn_timeout(&mut self) {
        let match_data = self.state.match_data.get_mut();
        let current_idx = match_data.current_player_index;
        
        if let Some(current_player) = match_data.players[current_idx].as_mut() {
            let player_nickname = current_player.nickname.clone();
            
            // Auto-draw one card
            let mut drew = false;
            
            // Check deck and refill if needed
            if match_data.deck.is_empty() && match_data.discard_pile.len() > 1 {
                let top_card = match_data.discard_pile.pop();
                match_data.deck.append(&mut match_data.discard_pile);
                let len = match_data.deck.len();
                for i in (1..len).rev() {
                    let j = (i * 7 + 11) % (i + 1);
                    match_data.deck.swap(i, j);
                }
                match_data.discard_pile.clear();
                if let Some(card) = top_card {
                    match_data.discard_pile.push(card);
                }
            }
            
            if let Some(card) = match_data.deck.pop() {
                current_player.hand.push(card);
                current_player.update_hand_size();
                match_data.deck_size = match_data.deck.len();
                drew = true;
            }
            
            // Advance turn
            let mut next_index = (current_idx + 1) % match_data.players.len();
            while match_data.players[next_index].is_none() {
                next_index = (next_index + 1) % match_data.players.len();
            }
            match_data.current_player_index = next_index;
            
            // Reset turn timer for next player
            match_data.turn_start_time = Some(self.runtime.system_time().micros());
            
            let _ = match_data;
            
            self.emit_event(GameEvent::TurnTimeout {
                player_nickname: player_nickname.clone(),
                auto_drawn: drew,
            });
            
            log::info!("PLAY_CHAIN: Turn timeout - {} auto-drew, turn advanced", player_nickname);
        }
    }

    // ================================================================================
    // HELPER FUNCTIONS
    // ================================================================================

    /// Centralized event emission
    fn emit_event(&mut self, event: GameEvent) {
        self.runtime.emit(GAME_STREAM_NAME.into(), &event);
    }

    /// Execute special card effects (actually modify game state)
    /// `player_who_played_index` is the index of the player who played the special card
    fn execute_special_effect(&mut self, effect: Option<SpecialEffect>, player_who_played_index: usize) {
        if let Some(special) = effect {
            match special {
                SpecialEffect::PickTwo => {
                    let match_data = self.state.match_data.get_mut();
                    
                    // Check if this is a stacking situation
                    if match_data.pending_draw_type == Some(2) {
                        // Another Pick Two played - stack it!
                        match_data.pending_draw_stack += 2;
                        log::info!("PLAY_CHAIN: Pick Two stacked - total pending: {}", match_data.pending_draw_stack);
                        // Next player can also stack or must draw
                        return;
                    }
                    
                    // Set penalty - next player must handle this
                    match_data.pending_draw_stack = 2;
                    match_data.pending_draw_type = Some(2);
                    log::info!("PLAY_CHAIN: Pick Two played - pending draw: 2 (next player can block or must draw)");
                }
                
                SpecialEffect::PickThree => {
                    let match_data = self.state.match_data.get_mut();
                    
                    // Check if this is a stacking situation
                    if match_data.pending_draw_type == Some(5) {
                        // Another Pick Three played - stack it!
                        match_data.pending_draw_stack += 3;
                        log::info!("PLAY_CHAIN: Pick Three stacked - total pending: {}", match_data.pending_draw_stack);
                        return;
                    }
                    
                    // Set penalty - next player must handle this
                    match_data.pending_draw_stack = 3;
                    match_data.pending_draw_type = Some(5);
                    log::info!("PLAY_CHAIN: Pick Three played - pending draw: 3 (next player can block or must draw)");
                }
                
                SpecialEffect::HoldOn => {
                    let match_data = self.state.match_data.get_mut();
                    let current_next = match_data.current_player_index;
                    
                    // Skip next player's turn
                    let mut skip_next = (current_next + 1) % match_data.players.len();
                    while match_data.players[skip_next].is_none() {
                        skip_next = (skip_next + 1) % match_data.players.len();
                    }
                    match_data.current_player_index = skip_next;
                    
                    log::info!("PLAY_CHAIN: Hold On - next player skipped");
                }
                
                SpecialEffect::GeneralMarket => {
                    let match_data = self.state.match_data.get_mut();
                    
                    // Draw for all OTHER players (not the one who played the 14)
                    for (index, player_opt) in match_data.players.iter_mut().enumerate() {
                        if index == player_who_played_index {
                            continue; // Skip the player who played General Market
                        }
                        
                        if let Some(player) = player_opt {
                            if let Some(card) = match_data.deck.pop() {
                                player.hand.push(card);
                                player.update_hand_size();
                            }
                        }
                    }
                    match_data.deck_size = match_data.deck.len();
                    
                    log::info!("PLAY_CHAIN: General Market - all OTHER players (not player {}) draw 1 card", player_who_played_index);
                }
                
                SpecialEffect::WhotPlayed { .. } => {
                    // No additional action - suit change already handled in validation
                    log::info!("PLAY_CHAIN: Whot played - suit changed");
                }
            }
        }
    }

}

