# Special Cards Verification - Backend vs Frontend

## Issue Report

User reports that special effects for cards with values 1, 2, and 5 are not syncing properly between backend and UI. Some suits trigger effects while others don't.

## Backend Special Card Logic (Confirmed)

From `backend/src/chains/play_chain.rs` lines 372-377:

```rust
let special_effect = match card.value {
    2 => Some(SpecialEffect::PickTwo),
    5 if card.suit != CardSuit::Star => Some(SpecialEffect::PickThree),
    1 => Some(SpecialEffect::HoldOn),
    14 => Some(SpecialEffect::GeneralMarket),
    20 => chosen_suit.map(|suit| SpecialEffect::WhotPlayed { chosen_suit: suit }),
    _ => None,
};
```

### Expected Behavior:

| Card Value | Suits                                       | Special Effect           | Backend Implementation                        |
| ---------- | ------------------------------------------- | ------------------------ | --------------------------------------------- |
| **1**      | ALL (Circle, Triangle, Cross, Square, Star) | Hold On                  | ✅ `1 => Some(SpecialEffect::HoldOn)`         |
| **2**      | ALL (Circle, Triangle, Cross, Square, Star) | Pick Two                 | ✅ `2 => Some(SpecialEffect::PickTwo)`        |
| **5**      | Circle, Triangle, Cross, Square ONLY        | Pick Three               | ✅ `5 if card.suit != CardSuit::Star`         |
| **5**      | Star                                        | NO EFFECT (regular card) | ✅ Excluded by condition                      |
| **14**     | ALL suits                                   | General Market           | ✅ `14 => Some(SpecialEffect::GeneralMarket)` |
| **20**     | ALL suits                                   | Whot/Wild                | ✅ `20 => chosen_suit.map(...)`               |

## Frontend Card Validation (Confirmed Correct)

From `frontend/components/PlayerOne.tsx` lines 41-54:

```tsx
const canPlayCard = (card: CardType, top: CardType | null): boolean => {
  if (!top) return true; // First card of the game

  // WHOT cards can always be played
  if (card.value === "20" || card.suit === "WHOT") return true;

  // If a suit has been demanded by a WHOT card, only that suit can be played
  if (activeDemandSuit) {
    return card.suit === activeDemandSuit;
  }

  // Normal rules: match suit or value
  return card.suit === top.suit || card.value === top.value;
};
```

**This is correct!** The frontend allows ANY card that matches suit or value, just like the backend.

## Visual Feedback Systems

### 1. Penalty Notifications (Pick 2 / Pick 3)

- Component: `PenaltyNotification.tsx`
- Shows warning to sender: "Pick X sent from You"
- Shows confirmation to receiver: "You were given X cards"
- Triggered by `pendingPenalty` state changes

### 2. Draw Pile Badge

- Component: `DrawPile.tsx`
- Shows red badge with penalty count when `pendingPenalty > 0`

### 3. Active Demand Suit (Whot)

- Displayed in `game/page.tsx`
- Large banner showing demanded suit after Whot played

## Potential Issues & Solutions

### Issue 1: Star Suit Value 5 Cards

**Problem:** Users may expect 5 of Stars to have Pick Three effect, but it shouldn't.

**Solution:** Add visual indicator on cards showing which are "special"

### Issue 2: No Visual Feedback for Value 1 (Hold On)

**Problem:** No notification/animation when turn is skipped

**Solution:** Need to add Hold On notification component

### Issue 3: No Visual Feedback for Value 14 (General Market)

**Problem:** No notification when all players draw cards

**Solution:** Need to add General Market notification

### Issue 4: Card Values Are Strings in Frontend

**Problem:** Backend uses numeric values, frontend uses strings

**Check:** Are card values properly converted when sent to backend?

## Testing Checklist

### Test Value 1 (Hold On) - ALL Suits

- [ ] Play 1 of Circle → Verify next player skipped
- [ ] Play 1 of Triangle → Verify next player skipped
- [ ] Play 1 of Cross → Verify next player skipped
- [ ] Play 1 of Square → Verify next player skipped
- [ ] Play 1 of Star → Verify next player skipped

### Test Value 2 (Pick Two) - ALL Suits

- [ ] Play 2 of Circle → Verify opponent draws 2
- [ ] Play 2 of Triangle → Verify opponent draws 2
- [ ] Play 2 of Cross → Verify opponent draws 2
- [ ] Play 2 of Square → Verify opponent draws 2
- [ ] Play 2 of Star → Verify opponent draws 2

### Test Value 5 (Pick Three) - Non-Star Suits Only

- [ ] Play 5 of Circle → Verify opponent draws 3
- [ ] Play 5 of Triangle → Verify opponent draws 3
- [ ] Play 5 of Cross → Verify opponent draws 3
- [ ] Play 5 of Square → Verify opponent draws 3
- [ ] Play 5 of Star → Verify NO special effect (regular card)

## Console Logging for Debugging

Add these logs to verify special effect execution:

```typescript
// In useWhotGame.ts, add logging after playCard mutation
console.log("[Special Card Debug]", {
  cardPlayed: myCards[cardIndex],
  pendingPenaltyBefore: gameState?.pendingPenalty,
  pendingPenaltyAfter: newState?.pendingPenalty,
});
```

```rust
// In play_chain.rs, already has logging:
log::info!("PLAY_CHAIN: Card played successfully");
// After execute_special_effect call
```

## Recommendations

1. **Add console logging** to track which cards are played and what effects trigger
2. **Add visual indicators** on Card component to show which cards are "special"
3. **Implement notifications** for Hold On and General Market effects
4. **Test systematically** with each suit for values 1, 2, and 5
5. **Check GraphQL responses** to verify pendingDrawStack updates correctly

## Quick Fix: Add Special Card Badges

Add visual badge to Card.tsx to show special card types:

```tsx
{
  /* Special Card Badge */
}
{
  (value === "1" ||
    value === "2" ||
    (value === "5" && suit !== "STAR") ||
    value === "14") && (
    <div className="absolute top-1 right-1 bg-yellow-400 text-black px-1 py-0.5 rounded text-xs font-bold">
      {value === "1"
        ? "HOLD"
        : value === "2"
        ? "P2"
        : value === "5"
        ? "P3"
        : "GM"}
    </div>
  );
}
```

This will help users identify which cards should have special effects.
