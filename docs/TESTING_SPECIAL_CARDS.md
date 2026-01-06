# Testing Special Cards - Debugging Guide

## Changes Made

### 1. Visual Indicators Added to Card Component

Cards now show colored badges indicating special effects:

- **HOLD** (Blue) - Value 1 - Hold On effect
- **P2** (Orange) - Value 2 - Pick Two effect
- **P3** (Red) - Value 5 (non-Star) - Pick Three effect
- **GM** (Purple) - Value 14 - General Market effect
- **WILD** (Yellow) - Value 20 - Whot/Wild effect

**Important:** Value 5 with Star suit will NOT show a badge because it has no special effect!

### 2. Enhanced Console Logging

When you click a card, the console will now show:

```javascript
{
  index: 2,
  card: { suit: "CIRCLE", value: "5" },
  topCard: { suit: "TRIANGLE", value: "7" },
  isSpecialCard: true,
  isPickThree: true,
  isStar5: false,
  expectedEffect: "Pick Three"
}
```

## What to Test

### Step-by-Step Testing Process

1. **Open Browser Console** (F12 or Cmd+Option+I)
2. **Start a game** with 2 players
3. **Play each special card type** and observe:
   - The badge on the card
   - The console logs
   - The backend effect (penalty notifications, turn skipping, etc.)

### Test Cases

#### Test 1: Value 2 (Pick Two) - ALL Suits Should Work

**Expected Behavior:** ALL suits trigger Pick Two effect

Play these cards one at a time:

- 2 of Circle → Console should show: `expectedEffect: "Pick Two"`
- 2 of Triangle → Console should show: `expectedEffect: "Pick Two"`
- 2 of Cross → Console should show: `expectedEffect: "Pick Two"`
- 2 of Square → Console should show: `expectedEffect: "Pick Two"`
- 2 of Star → Console should show: `expectedEffect: "Pick Two"`

**Verify:**

- ✅ Penalty notification appears: "Pick 2 sent from You"
- ✅ `pendingPenalty` changes from 0 to 2
- ✅ Opponent auto-draws 2 cards
- ✅ Confirmation appears on opponent's screen: "You were given 2 cards"

#### Test 2: Value 5 (Pick Three) - NON-Star Suits Only

**Expected Behavior:** Circle, Triangle, Cross, Square trigger Pick Three. Star does NOT.

Play these cards:

- 5 of Circle → Console: `isPickThree: true`, `expectedEffect: "Pick Three"`
- 5 of Triangle → Console: `isPickThree: true`, `expectedEffect: "Pick Three"`
- 5 of Cross → Console: `isPickThree: true`, `expectedEffect: "Pick Three"`
- 5 of Square → Console: `isPickThree: true`, `expectedEffect: "Pick Three"`
- **5 of Star** → Console: `isStar5: true`, `expectedEffect: "NONE (Star 5 is regular card)"`

**Verify:**

- ✅ First 4 cards (non-Star) show "Pick 3 sent from You" notification
- ✅ `pendingPenalty` changes to 3 for non-Star cards
- ✅ Opponent draws 3 cards for non-Star cards
- ✅ **Star 5 card shows NO badge and has NO special effect** (regular card)

#### Test 3: Value 1 (Hold On) - ALL Suits Should Work

**Expected Behavior:** ALL suits skip next player's turn

Play these cards:

- 1 of Circle → Console: `expectedEffect: "Hold On"`
- 1 of Triangle → Console: `expectedEffect: "Hold On"`
- 1 of Cross → Console: `expectedEffect: "Hold On"`
- 1 of Square → Console: `expectedEffect: "Hold On"`
- 1 of Star → Console: `expectedEffect: "Hold On"`

**Verify:**

- ✅ Next player's turn is skipped
- ✅ Turn advances by 2 positions (skips opponent)
- ⚠️ **Note:** There's no visual notification for Hold On yet (enhancement needed)

#### Test 4: Value 14 (General Market) - ALL Suits Should Work

**Expected Behavior:** All other players draw 1 card

Play these cards:

- 14 of Circle → Console: `expectedEffect: "General Market"`
- 14 of Triangle → Console: `expectedEffect: "General Market"`
- 14 of Cross → Console: `expectedEffect: "General Market"`
- 14 of Square → Console: `expectedEffect: "General Market"`

**Verify:**

- ✅ All players (except current) draw 1 card
- ✅ Deck size decreases by number of other players
- ⚠️ **Note:** There's no visual notification for General Market yet (enhancement needed)

## Common Issues & Solutions

### Issue: "Special card not working for some suits"

**Diagnosis Steps:**

1. Check console logs - does it detect the card as special?
2. Check the badge - is it showing on the card?
3. Check pendingPenalty in GraphQL query - is it updating?

**Possible Causes:**

- **Card values as strings vs numbers:** Backend expects numeric, frontend uses strings
- **Suit name mismatch:** Backend uses `CardSuit::Star`, frontend uses `"STAR"`
- **GraphQL mutation not sending data correctly**

### Issue: "5 of Star showing special effect"

**This is WRONG!** Backend code explicitly excludes Star suit:

```rust
5 if card.suit != CardSuit::Star => Some(SpecialEffect::PickThree)
```

If Star 5 is triggering Pick Three, there's a backend bug.

### Issue: "No visual feedback for Hold On or General Market"

**This is expected!** Currently, only Pick 2/Pick 3 have notifications.

**Enhancements needed:**

- Add HoldOnNotification component
- Add GeneralMarketNotification component

## Debugging Backend

If frontend logs look correct but effects don't trigger, check backend logs:

```bash
# In backend terminal, look for these logs:
[INFO linot_contract::chains::play_chain] PLAY_CHAIN: Card played successfully
[INFO linot_contract::chains::play_chain] PLAY_CHAIN: Pick Two - adding to stack
[INFO linot_contract::chains::play_chain] PLAY_CHAIN: Pick Three - adding to stack
```

## GraphQL Query to Check State

Use this query to verify game state after playing special card:

```graphql
query CheckGameState {
  matchState {
    status
    currentPlayerIndex
    pendingDrawStack
    activeDemandSuit
    discardPile {
      suit
      value
    }
    players {
      nickname
      handSize
    }
  }
}
```

Expected after Pick 2:

- `pendingDrawStack: 2`
- Top card in `discardPile` has `value: 2`

Expected after Pick 3 (non-Star):

- `pendingDrawStack: 3`
- Top card has `value: 5` and `suit` is NOT "Star"

Expected after Pick 3 (Star):

- `pendingDrawStack: 0` (unchanged)
- This is a regular card!

## Summary

**What Should Work:**

- ✅ Value 1 (all suits) → Hold On
- ✅ Value 2 (all suits) → Pick Two
- ✅ Value 5 (Circle, Triangle, Cross, Square) → Pick Three
- ✅ Value 5 (Star) → NO EFFECT (regular card)
- ✅ Value 14 (all suits) → General Market
- ✅ Value 20 (all suits) → Whot/Wild

**Visual Indicators:**

- ✅ Badges show on special cards
- ✅ Star 5 has NO badge
- ✅ Console logs show expected effects

**Next Steps:**

1. Test each card systematically
2. Share console logs if effects don't work
3. Check backend logs for errors
4. Verify GraphQL responses match expectations

If specific cards aren't working, please share:

- Which card (suit and value)
- Console log output
- What happened vs what you expected
- Screenshots of the card (to verify badge display)
