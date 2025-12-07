# Multiplayer Deployment Guide - Linot Whot Card Game

## Quick Start - Local Multiplayer Testing

### Prerequisites
```bash
# Ensure Linera CLI is installed
linera --version

# Navigate to project
cd /home/dinahmaccodes/Documents/codes-rust-linera/linot-card-game
```

### Step 1: Build the Application
```bash
cd backend
cargo build --release --target wasm32-unknown-unknown
cd ..
```

### Step 2: Start Local Network
```bash
# Start a local Linera devnet with multiple validators
linera net up

# This creates multiple chains for testing multiplayer
```

### Step 3: Publish Application
```bash
# Publish and create the application on your default chain
linera project publish-and-create

# Note the APPLICATION_ID returned (e.g., e476187f...)
export APP_ID=<your_application_id>
```

### Step 4: Get Chain IDs
```bash
# List your chains
linera wallet show

# You should see multiple chains, note two of them:
export HOST_CHAIN=<chain_1_id>   # Will be the game host
export PLAYER_CHAIN=<chain_2_id> # Will be player 2
```

## Multiplayer Game Flow

### Scenario: 2 Players on Different Chains

#### As Host (Chain 1)
```bash
# Set active chain to host
linera wallet set-default $HOST_CHAIN

# 1. Join match as Player 1 (host)
linera project run -a $APP_ID -o JoinMatch -p '{
  "nickname": "Alice"
}'

# 2. Wait for Player 2 to join (from other chain)
# You'll see a PlayerJoined event when they join

# 3. Start the match (only host can start)
linera project run -a $APP_ID -o StartMatch

# 4. Check initial state
linera service query-application $APP_ID '{ 
  matchState { 
    status 
    currentPlayerIndex
    players { nickname cardCount isActive }
  } 
}'

# 5. Play your turn when it's your turn
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 0,
  "chosen_suit": null
}'

# If playing a Whot card (20), must specify suit:
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 2,
  "chosen_suit": "Star"
}'
```

#### As Player 2 (Chain 2)
```bash
# Set active chain to player chain
linera wallet set-default $PLAYER_CHAIN

# 1. Join the match on host's chain
linera project run -a $APP_ID -o JoinFromChain -p '{
  "host_chain_id": "'$HOST_CHAIN'",
  "nickname": "Bob"
}'

# 2. Wait for host to start the match
# (Listen to events or poll state)

# 3. Check your hand and game state
linera service query-application $APP_ID '{ 
  matchState { 
    currentPlayerIndex
    players { nickname cardCount }
    topCard { suit value }
  } 
}'

# 4. Play when it's your turn
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 1,
  "chosen_suit": null
}'

# Or draw a card if you can't play
linera project run -a $APP_ID -o DrawCard
```

## GraphQL Queries Reference

### Check for Turn Timeout
```graphql
{
  matchState {
    turnStartedAt      # Timestamp when current turn started
    currentPlayerIndex
    status
  }
}
```

### Enforce Timeout (Anyone Can Call)
```bash
# If a player takes longer than 3 minutes, anyone can call this
linera project run -a $APP_ID -o CheckTimeout

# Backend will:
# 1. Check if 3 minutes elapsed
# 2. Auto-draw one card for the slow player
# 3. Advance to next player's turn
# 4. Emit TurnTimeout event
```

### Check Match Status
```graphql
{
  status
  currentPlayerIndex
  deckSize
}
```

### Get Player Hands (Debug - shows all hands!)
```graphql
{
  matchState {
    players {
      nickname
      cards {
        suit
        value
      }
      cardCount
      isActive
      calledLastCard
    }
  }
}
```

### Get Top Card
```graphql
{
  topCard {
    suit
    value
  }
}
```

### Get Your Cards (Production)
```graphql
{
  playerView(owner: "<your_account_owner>") {
    nickname
    cards {
      suit
      value
    }
    cardCount
  }
}
```

## Special Card Examples

### Playing Hold On (Card 1)
```bash
# Play Hold On card
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 3,
  "chosen_suit": null
}'

# IMMEDIATELY play second card (must match shape of Hold On card)
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 0,
  "chosen_suit": null
}'
```

### Blocking Pick 2
```bash
# If opponent plays Pick 2, you can block with any card of value 2
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 5,  # This should be a "2" card of any suit
  "chosen_suit": null
}'
# Now the penalty (2+2=4 cards) passes to next player
```

### Playing Whot (Card 20)
```bash
# Always specify the shape you want
linera project run -a $APP_ID -o PlayCard -p '{
  "card_index": 4,
  "chosen_suit": "Triangle"
}'
# Next player must play Triangle suit
```

### Stacking Penalties
```bash
# Player 1 plays Pick 2
# Player 2 plays Pick 2 (blocks and stacks)
# Player 3 plays Pick 2 (blocks and stacks)
# Player 4 must draw 6 cards (2+2+2) or block with another Pick 2
```

## Event Monitoring

### Subscribe to Game Events
```bash
# The application emits events on the stream: "linot_game_events"
# Events include:
# - PlayerJoined { nickname, player_count }
# - MatchStarted { first_player, top_card }
# - CardPlayed { player, card, next_player, special_effect }
# - CardsDrawn { player, count, next_player }
# - MatchEnded { winner, winner_index }
# - PlayerLeft { nickname }

# Subscribe (from client code):
# runtime.subscribe_to_events(host_chain_id, app_id, "linot_game_events")
```

## Testing Scenarios

### Test 1: Basic 2-Player Game
1. Host joins and starts
2. Player 2 joins via cross-chain
3. Players alternate turns
4. First to 0 cards wins

### Test 2: Special Cards
1. Test Hold On (1) - must play twice
2. Test Pick 2 - next player draws 2
3. Test Pick 3 - next player draws 3
4. Test Suspension (8) - skip next player
5. Test General Market (14) - all others draw 1
6. Test Whot (20) - choose shape

### Test 3: Penalty Stacking
1. Player 1 plays Pick 2
2. Player 2 plays Pick 2 (stack to 4)
3. Player 3 plays Pick 2 (stack to 6)
4. Player 4 draws 6 cards

### Test 4: Player Forfeit
1. Player leaves via LeaveMatch
2. Remaining player wins automatically

## Deployment to Testnet

### Prerequisites
- Testnet access configured
- Faucet tokens for deployment

### Steps
```bash
# Set testnet network
linera wallet init --with-new-chain --faucet <TESTNET_FAUCET_URL>

# Build application
cd backend
cargo build --release --target wasm32-unknown-unknown

# Publish to testnet
linera project publish-and-create

# Share APP_ID and HOST_CHAIN_ID with other players
```

### For Players Joining Testnet Game
```bash
# Get application ID from host
export APP_ID=<app_id_from_host>
export HOST_CHAIN=<host_chain_id>

# Join the match
linera project run -a $APP_ID -o JoinFromChain -p '{
  "host_chain_id": "'$HOST_CHAIN'",
  "nickname": "YourNickname"
}'
```

## Troubleshooting

### "Match Already Started"
- Match has already begun, cannot join
- Host needs to create a new match

### "Not Your Turn"
- Wait for your turn
- Check `currentPlayerIndex` in match state

### "Invalid Card Play"
- Card doesn't match suit or value
- Check if you need to block a penalty
- Check if Hold On is active (must match shape)

### "Match Full"
- Maximum 2 players for V1
- Host needs to increase `max_players` in config (future)

### Player Can't Join Cross-Chain
- Ensure HOST_CHAIN_ID is correct
- Check application is published on both chains
- Verify network connectivity

### Hold On Not Working
- Must play second card immediately after Hold On
- Second card must match the SHAPE of the Hold On card
- Cannot play different shape (except Whot that matches)

## Performance Tips

### Reduce Latency
- Deploy on chains in same region
- Use local devnet for testing
- Minimize query frequency

### State Management
- Query only when needed
- Cache player state locally
- Subscribe to events for updates

## Security Notes

### What's Protected
- ✅ Turn validation (only current player can play)
- ✅ Card validation (must be valid play)
- ✅ Host-only start (only host can start match)
- ✅ Caller authentication (only owner can play their cards)

### What's Not Protected (Yet)
- ❌ Turn timeout enforcement (3-minute rule)
- ❌ Hand visibility (GraphQL exposes all hands in debug mode)
- ❌ Spectator mode (anyone can query state)

## Advanced Configuration

### Custom Match Config
```bash
# When publishing, pass custom config:
linera project publish-and-create --instantiation-arg '{
  "max_players": 4,
  "host": "<your_account_owner>",
  "is_ranked": false,
  "strict_mode": true
}'
```

### Ranked Matches (Future)
- Set `is_ranked: true`
- Implement ELO/ranking system
- Store match history

### Strict Mode
- Enforces "must draw if no valid move"
- No optional drawing

## Next Steps

1. **Build Frontend**
   - React/Vue app
   - WebSocket event handling
   - Card UI components
   - Turn timer display

2. **Add Turn Timeout**
   - Background timer check
   - Auto-draw after 3 minutes
   - Warning at 2 minutes

3. **Implement Multiple Card Play**
   - Play multiple cards of same number
   - One transaction

4. **Testing**
   - Write integration tests
   - Test all special card combos
   - Load testing with multiple games

5. **Production Deployment**
   - Deploy to Linera mainnet
   - Add monitoring/logging
   - Implement proper error handling

---

## Quick Command Reference

```bash
# Build
cargo build --release --target wasm32-unknown-unknown

# Start network
linera net up

# Publish
linera project publish-and-create

# Join local
linera project run -a $APP_ID -o JoinMatch -p '{"nickname":"Player1"}'

# Join cross-chain
linera project run -a $APP_ID -o JoinFromChain -p '{"host_chain_id":"<CHAIN>","nickname":"Player2"}'

# Start
linera project run -a $APP_ID -o StartMatch

# Play card
linera project run -a $APP_ID -o PlayCard -p '{"card_index":0,"chosen_suit":null}'

# Draw card
linera project run -a $APP_ID -o DrawCard

# Leave
linera project run -a $APP_ID -o LeaveMatch

# Query state
linera service query-application $APP_ID '{ matchState { status players { nickname cardCount } } }'
```

---

**Status**: ✅ Backend ready for local and testnet multiplayer deployment
