
## WHOT Card Game Logic Specification 

This document defines the core rules and special card mechanics for the digital implementation of the  Whot card game.

### 1. Game Components & Card Types

| Type | Shapes/Numbers | Special Cards |
| :--- | :--- | :--- |
| **Shapes (Suits)** | Circle (0-9), Star (1-7), Triangle (1-9), Cross (1-7), Square (1-5) | N/A |
| **Action Numbers** | **1, 2, 5, 8, 14, 20** (Whot) | Defined Below |
| **General** | Standard deck size (usually 54 cards, containing shape/number combinations). | N/A |

---

### 2. Core Gameplay Rules

| Rule ID | Rule Description | Condition for AI |
| :--- | :--- | :--- |
| **R1** | **Starting Play** | The game starts by placing one random card from the stock/deck face up. |
| **R2** | **Matching Play** | A player must play a card that matches the **shape** OR the **number** of the top card on the pile. |
| **R3** | **Multiple Play (Grouping)** | A player **can play multiple cards** in a single turn **ONLY if** they all share the **same numerical value**. They cannot group by shape. |
| **R4** | **Drawing** | If a player cannot play a valid card, they must draw one card from the deck. The turn passes to the next player. |
| **R5** | **Game End** | The first player to exhaust their hand wins the game. |
| **R6** | **Digital Time Limit** | If a player fails to play a card within **3 minutes**, their turn is skipped, and they must draw one random card from the deck. |

---

### 3. Special Action Cards & Effects

The following cards trigger special actions when played.

| Card Number | Name | Effect Triggered (Single Play) | Blocking Condition |
| :--- | :--- | :--- | :--- |
| **1** | **Hold On** | **Suspends the turn flow.** The player who played the '1' must immediately play a **second card** (matching the '1's shape/number, or a Whot 20) to continue the turn. The next player is skipped. | Cannot be blocked. |
| **2** | **Pick Two** | **Next Player Action:** The next player must draw **2 cards** from the deck, and their turn is skipped. | Can be blocked by playing **ANY** card with the numerical value **2** on top of it. |
| **5** | **Pick Three** | **Next Player Action:** The next player must draw **3 cards** from the deck, and their turn is skipped. | Can be blocked by playing **ANY** card with the numerical value **5** on top of it. |
| **8** | **Suspension** | **Next Player Action:** The next player's turn is **skipped**. Turn passes to the player after them. | Can be blocked by playing **ANY** card with the numerical value **8** on top of it. |
| **14** | **General Market** | **All Other Players Action:** **Every other player** (besides the player who played the '14') must draw **1 card** from the deck. Turn passes to the next player in sequence. | **CANNOT be blocked.** |
| **20** | **WHOT** | **Wild Card & Shape Request:** Can be played on **ANY** card (regardless of shape or number). The player must declare the **shape** they want the next player to follow (e.g., "WHOT! Star!"). | Cannot be blocked. |

---

### 4. Blocking and Stacking Rules

| Action | Condition | Outcome |
| :--- | :--- | :--- |
| **Block (Pick 2)** | Top card is a '2'. The current player plays **any card with the number 2**. | The **pick 2** penalty is passed to the player after the blocker (or nullified if the 2 is the last card played in a series). |
| **Block (Pick 3)** | Top card is a '5'. The current player plays **any card with the number 5**. | The **pick 3** penalty is passed to the player after the blocker (or nullified). |
| **Block (Suspension)** | Top card is an '8'. The current player plays **any card with the number 8**. | The **suspension** penalty is passed to the player after the blocker (or nullified). |
| **No Blocking** | A General Market (**14**) is played. | The effect is immediate and cannot be blocked or chained. |
| **Chain/Stacking** | If an opponent plays a **Pick X** card (2 or 5), and the current player plays their own **Pick X** card of the same number, the penalties are typically **combined** and passed on. | *AI Suggestion:* Implement stacking for Pick 2 and Pick 5 cards. The penalty accumulates. |

---

### 5. Suggested Missing Rules for Consideration

| Rule Suggestion | Rationale | Recommendation |
| :--- | :--- | :--- |
| **Mandatory Shape Follow (After a 20)** | If a player calls a shape with the '20' card, the next player *must* follow that shape, even if they have to draw. | **Strongly Recommended.** |
| **Pick Stacking (Chain Effect)** | When a 'Pick 2' is blocked with another 'Pick 2', the penalty should stack (e.g., total Pick 4). | **Strongly Recommended.** |
| **Announcing Last Card** | A player must verbally or digitally announce "Last Card" when they have one card remaining. Failure to do so incurs a penalty (e.g., drawing two cards). | **Optional for Digital.** |
| **Penalty for Invalid Play** | If a player attempts an illegal play (e.g., doesn't match the card and isn't a '20'), they must draw a card. | **Recommended.** |
