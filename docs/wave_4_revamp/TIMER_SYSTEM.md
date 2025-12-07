# 3-Minute Turn Timer System

## Overview

Each player has **exactly 3 minutes** to make their move once it becomes their turn. If they exceed this time limit:
- ❌ They **cannot play a card** (operation will fail with `TurnTimeout` error)
- ✅ Anyone can call `CheckTimeout` operation to enforce the penalty
- 🎴 The slow player **automatically draws 1 card**
- ⏭️ Turn **advances to the next player**

---

## How It Works

### 1. Turn Timer Starts

When a player's turn begins, the backend sets `turn_started_at`:

```rust
// In contract.rs - StartMatch, PlayCard, DrawCard
match_data.turn_started_at = self.runtime.system_time().micros();
```

This timestamp is stored on-chain and tracks when the current player's turn started.

---

### 2. Automatic Timeout Check (Built-in)

Before allowing **any action** (`PlayCard` or `DrawCard`), the backend checks if 3 minutes have passed:

```rust
// In handle_play_card() and handle_draw_card()
let current_time = self.runtime.system_time().micros();
let elapsed = current_time.saturating_sub(match_data.turn_started_at);

if elapsed >= TURN_TIMEOUT_MICROS {  // 3 * 60 * 1,000,000 microseconds
    return Err(LinotError::TurnTimeout);
}
```

**Result:** Player cannot play after 3 minutes - their operation fails immediately.

---

### 3. Manual Timeout Enforcement (Anyone Can Trigger)

Any player (or even a spectator) can call `CheckTimeout` to enforce the penalty:

```bash
linera project run -a $APP_ID -o CheckTimeout
```

**What happens:**
1. Backend checks if current player exceeded 3 minutes
2. If yes:
   - Draw 1 card for the slow player (as penalty)
   - Clear any active game states (Hold On, Whot demand, etc.)
   - Advance turn to next player
   - Reset timer for next player
   - Emit `TurnTimeout` event

```rust
// In handle_check_timeout()
if elapsed >= TURN_TIMEOUT_MICROS {
    // Auto-draw 1 card
    if let Some(card) = match_data.deck.pop() {
        current_player.cards.push(card);
    }
    
    // Advance turn
    GameEngine::advance_turn(&mut match_data);
    match_data.turn_started_at = current_time;
    
    // Emit event
    emit(TurnTimeout { player, auto_drawn: true });
}
```

---

## Frontend Integration

### Display Timer

```typescript
// Frontend: src/components/TurnTimer.tsx
import { useQuery } from '@apollo/client';

function TurnTimer({ matchState }) {
  const [timeLeft, setTimeLeft] = useState(180); // 3 minutes = 180 seconds
  
  useEffect(() => {
    const turnStartedAt = matchState.turnStartedAt;
    const currentTime = Date.now() * 1000; // Convert to microseconds
    const elapsed = (currentTime - turnStartedAt) / 1_000_000; // Convert to seconds
    const remaining = 180 - elapsed;
    
    setTimeLeft(Math.max(0, Math.floor(remaining)));
    
    const interval = setInterval(() => {
      setTimeLeft(prev => Math.max(0, prev - 1));
    }, 1000);
    
    return () => clearInterval(interval);
  }, [matchState.turnStartedAt]);
  
  const minutes = Math.floor(timeLeft / 60);
  const seconds = timeLeft % 60;
  
  return (
    <div className={timeLeft < 30 ? "timer-warning" : "timer"}>
      ⏱️ {minutes}:{seconds.toString().padStart(2, '0')}
      {timeLeft === 0 && <span>⚠️ TIME'S UP!</span>}
    </div>
  );
}
```

### Auto-Enforce Timeout

```typescript
// Frontend: src/hooks/useAutoTimeout.ts
import { useMutation } from '@apollo/client';

function useAutoTimeout(matchState) {
  const [checkTimeout] = useMutation(CHECK_TIMEOUT);
  
  useEffect(() => {
    const turnStartedAt = matchState.turnStartedAt;
    const currentTime = Date.now() * 1000;
    const elapsed = (currentTime - turnStartedAt) / 1_000_000;
    
    // If already timed out, call CheckTimeout
    if (elapsed >= 180) {
      checkTimeout();
    } else {
      // Set timer to auto-call when timeout occurs
      const timeUntilTimeout = (180 - elapsed) * 1000;
      const timeout = setTimeout(() => {
        checkTimeout();
      }, timeUntilTimeout);
      
      return () => clearTimeout(timeout);
    }
  }, [matchState.turnStartedAt]);
}
```

### Listen to Timeout Events

```typescript
// Frontend: src/hooks/useGameEvents.ts
subscription.on('TurnTimeout', (event) => {
  console.log(`${event.player} took too long!`);
  
  // Show notification
  showNotification({
    title: "Turn Timeout!",
    message: `${event.player} exceeded 3 minutes and drew a card`,
    type: "warning"
  });
  
  // Refetch game state
  refetchGameState();
});
```

---

## Constants

```rust
// In lib.rs
pub const TURN_TIMEOUT_MICROS: u64 = 3 * 60 * 1_000_000; // 180 seconds = 3 minutes
pub const TURN_WARNING_MICROS: u64 = 2 * 60 * 1_000_000; // 120 seconds = 2 minutes
```

---

## GraphQL Queries

### Check Current Turn Timer

```graphql
query GetTurnTimer {
  matchState {
    turnStartedAt      # Unix timestamp in microseconds
    currentPlayerIndex # Whose turn it is
    status             # InProgress/Waiting/Finished
  }
}
```

### Calculate Time Remaining (Frontend)

```javascript
const currentTime = Date.now() * 1000; // microseconds
const elapsed = (currentTime - turnStartedAt) / 1_000_000; // seconds
const remaining = 180 - elapsed; // seconds left
```

---

## Operations

### CheckTimeout (Enforce Penalty)

```graphql
mutation CheckTimeout {
  checkTimeout
}
```

**Returns:** Nothing (void), but emits `TurnTimeout` event if timeout occurred

---

## Events Emitted

### TurnTimeout

```rust
GameEvent::TurnTimeout {
    player: String,      // Nickname of slow player
    auto_drawn: bool,    // true if card was drawn, false if deck empty
}
```

---

## Example Flow

**Scenario:** Player Alice is taking too long

```
T=0:00   Alice's turn starts
         └─> turn_started_at = 1234567890000000

T=2:30   Warning could be shown (2 minutes elapsed)
         └─> Frontend shows yellow timer

T=3:00   Timeout! Alice tries to play
         ├─> PlayCard operation fails with TurnTimeout error
         └─> Frontend shows "Time's up!" message

T=3:01   Bob (opponent) calls CheckTimeout
         ├─> Backend: elapsed = 181 seconds > 180 ✅
         ├─> Alice draws 1 card (penalty)
         ├─> Turn advances to Bob
         ├─> turn_started_at = new timestamp
         └─> Event emitted: TurnTimeout { player: "Alice", auto_drawn: true }

T=3:02   All clients receive event
         ├─> Update UI: "Alice timed out and drew a card"
         ├─> Bob's timer starts
         └─> Game continues
```

---

## Multiplayer Fairness

**Why anyone can call `CheckTimeout`:**

1. **No reliance on slow player** - Opponent doesn't have to wait forever
2. **Automated enforcement** - Frontend can auto-call after 3 minutes
3. **Prevents stalling tactics** - Can't slow down game intentionally
4. **Spectators can help** - Even non-players can keep game moving

**Security:** The backend validates the timeout itself, so malicious calls before 3 minutes do nothing.

---

## Testing

### Local Test

```bash
# Terminal 1: Start as Player 1
linera project run -a $APP_ID -o JoinMatch -p '{"nickname": "Alice"}'

# Terminal 2: Start as Player 2
linera project run -a $APP_ID -o JoinMatch -p '{"nickname": "Bob"}'

# Terminal 1: Start match
linera project run -a $APP_ID -o StartMatch

# Wait 3+ minutes (or manually adjust turn_started_at in state)

# Terminal 2: Try to play (should fail)
linera project run -a $APP_ID -o PlayCard -p '{"card_index": 0, "chosen_suit": null}'
# Error: Turn timeout - player took too long

# Terminal 2: Enforce timeout
linera project run -a $APP_ID -o CheckTimeout
# Success: Alice drew a card, now Bob's turn
```

---

## Future Enhancements

1. **Configurable timeout** - Allow host to set custom timeout (1-5 minutes)
2. **Warning events** - Emit `TurnTimeoutWarning` at 2 minutes
3. **Grace period** - Allow 10 seconds after timeout before enforcement
4. **Pause system** - Allow players to pause (with opponent consent)
5. **Reconnection handling** - Pause timer if player disconnects

---

## Summary

✅ **3-minute timer** enforced on-chain  
✅ **Automatic prevention** - Can't play after timeout  
✅ **Manual enforcement** - Anyone can trigger penalty  
✅ **Fair gameplay** - No stalling tactics  
✅ **Event-driven** - Real-time notifications  
✅ **Production-ready** - Fully implemented and tested

The timer system ensures **fast-paced, fair multiplayer gameplay** - exactly what Linera's low-latency infrastructure enables! ⚡🎮
