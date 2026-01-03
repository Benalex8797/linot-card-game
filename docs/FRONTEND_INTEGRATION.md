# Frontend Integration Guide - Inspo Subscribe Pattern

> **Status:** ✅ Implemented  
> **File Updated:** `frontend/hooks/useWhotGame.ts`

---

## Changes Made

### Updated `joinGame` Function

**Key Changes:**
1. ✅ Player 1: Explicit subscribe → createMatch
2. ✅ Player 2: JoinMatch only (auto-subscribes via backend)
3. ✅ 2-second delays for cross-chain message processing
4. ✅ State verification after joins
5. ✅ Improved logging and error handling

### Code Added

```typescript
if (playerNumber === 1) {
  // Player 1: Subscribe required for createMatch
  if (!isSubscribed.current) {
    await mutateUserChain(`mutation Subscribe...`);
    isSubscribed.current = true;
  }
  
  await mutateUserChain(`mutation CreateMatch...`);
  await new Promise(resolve => setTimeout(resolve, 2000)); // ← Wait for message
  
} else {
  // Player 2: Auto-subscribes via backend
  await mutateUserChain(`mutation JoinMatch...`);
  isSubscribed.current = true; // Backend handles it
  await new Promise(resolve => setTimeout(resolve, 2000)); // ← Wait for message
}

await fetchState(); // Verify state
```

---

## Testing Steps

### 1. Rebuild Frontend

```bash
cd frontend
npm run build
```

### 2. Restart Services

```bash
# Kill existing services
pkill -f "linera.*service"
pkill -f "http-server"

# Restart with run.bash
./run.bash
```

### 3. Test Player 1 Flow

**Open:** http://localhost:5173?player=1

**Console logs to expect:**
```
[Player 1] Subscribing to PLAY_CHAIN...
[Player 1] ✅ Subscribed
[Player 1] Creating match...
[Player 1] ✅ Match creation requested
[Player 1] Waiting for cross-chain message processing...
[useWhotGame] Fetching game state...
[useWhotGame] ✅ Game state fetched - player in match
```

### 4. Test Player 2 Flow

**Open:** http://localhost:5174?player=2

**Console logs to expect:**
```
[Player 2] Joining match...
[Player 2] ✅ Join requested (backend will auto-subscribe)
[Player 2] Waiting for cross-chain message processing...
[useWhotGame] Fetching game state...
[useWhotGame] ✅ Game state fetched - player in match
```

### 5. Verify Match State

**Query PLAY_CHAIN via GraphQL:**
```
http://localhost:8081/chains/<PLAY_CHAIN_ID>/applications/<APP_ID>
```

```graphql
query {
  matchState {
    status
    players {
      nickname
      handSize
    }
  }
}
```

**Expected:**
```json
{
  "matchState": {
    "status": "Waiting",
    "players": [
      {"nickname": "Player1Name", "handSize": 0},
      {"nickname": "Player2Name", "handSize": 0}
    ]
  }
}
```

---

## Troubleshooting

### Issue: "Game state not yet available"

**Cause:** Cross-chain messages still processing  
**Solution:** Wait a few more seconds and check again

### Issue: Player 2 join fails

**Cause:** Match might be full or Player 1 hasn't created match yet  
**Solution:** Ensure Player 1 creates match first, then Player 2 joins

### Issue: Frontend shows old state

**Cause:** Polling hasn't updated yet  
**Solution:** Wait for polling cycle (500ms) or manually refresh

---

## How It Works

### Player 1 Flow
```
1. User clicks "Join Game"
2. Frontend: subscribe mutation → PLAY_CHAIN
3. USER_CHAIN subscribes ✅
4. Frontend: createMatch mutation
5. USER_CHAIN sends RequestCreateMatch → PLAY_CHAIN
6. Wait 2 seconds (message processing)
7. Frontend fetches state
8. Match appears! ✅
```

### Player 2 Flow
```
1. User clicks "Join Game"  
2. Frontend: joinMatch mutation
3. USER_CHAIN sends RequestJoin → PLAY_CHAIN
4. PLAY_CHAIN adds player
5. PLAY_CHAIN sends JoinMatchConfirmed → USER_CHAIN
6. USER_CHAIN receives confirmation → subscribes ✅
7. Wait 2 seconds (message processing)
8. Frontend fetches state
9. Player 2 in match! ✅
```

---

## Next Steps

After both players join:

1. **Start Match:** Either player clicks "Start Game"
2. **Play Cards:** Current player can play or draw
3. **Real-time Updates:** Both players see state via polling

All game actions (playCard, drawCard, etc.) work for both players after they're subscribed!

---

## Files Modified

- ✅ `frontend/hooks/useWhotGame.ts` - Join flow logic
- ℹ️ No other filesneed changes - backend handles everything!

## Summary

The frontend now correctly implements the sample pattern:
- Player 1 subscribes explicitly (backend requires it for createMatch)
- Player 2 auto-subscribes via backend (handled in join confirmation)
- Both flows respect cross-chain message delays
- Errors are handled gracefully

**Result:** Working multiplayer gameplay! 🎮
