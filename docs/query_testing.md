For Mutations and Queries testing Guide

## Setup Flow (Why This Order)

**Subscribe first** → Enables receiving events from PLAY_CHAIN  
**Create/Join** → Registers players on game chain  
**Start** → Triggers card dealing and game start

---

## Mutation Order

### Player 1 Setup (Port 8081)

```graphql
# 1. Subscribe to PLAY_CHAIN
mutation { subscribe(playChainId: "PLAY_CHAIN_ID") }

# 2. Create match
mutation { createMatch(maxPlayers: 2, nickname: "Alice") }

# Wait for Player 2... to subscribe first!!

# 5. Start match (after P2 joins)
mutation { startMatch }
```

### Player 2 Setup (Port 8082)

```graphql
# 3. Subscribe to PLAY_CHAIN
mutation { subscribe(playChainId: "PLAY_CHAIN_ID") }

# 4. Join match
mutation {
  joinMatch(playChainId: "PLAY_CHAIN_ID", nickname: "Baeline")
}
```

---

## Gameplay Mutations

### Play Card
**Must match top card's suit OR value (or be Whot)**

```graphql
# Regular card (index 0 = first card in hand)
mutation { playCard(cardIndex: 0, chosenSuit: null) }

# Whot card (choose suit)
mutation { playCard(cardIndex: 2, chosenSuit: CIRCLE) }
```

**Suits**: `CIRCLE`, `TRIANGLE`, `CROSS`, `SQUARE`, `STAR`, `WHOT`

### Draw Card
```graphql
mutation { drawCard }
```

### Call Last Card
**Required when down to 1 card**
```graphql
mutation { callLastCard }
```

### Challenge
**Penalize player with 1 card who didn't call**
```graphql
mutation { challengeLastCard(playerIndex: 1) }
```

---

## Card Matching Rules

**Valid if:**
- Same suit as top card
- Same value as top card
- Whot card (always valid)

**Special Cards:**
- `2`: Pick Two
- `5` (not Star): Pick Three  
- `1`: Hold On
- `14`: General Market
- `20` (Whot): Choose suit

---

If card is played successfully, 

```
mutation { playCard(cardIndex: 0, chosenSuit: null) }
```

## Queries

### Full Match State
```graphql
query {
  matchState {
    status
    currentPlayerIndex
    maxPlayers
    deckSize
    discardPile { suit value }
    players {
      nickname
      handSize
      calledLastCard
    }
  }
}
```

How Output might be:

```
{
  "data": {
    "matchState": {
      "status": "IN_PROGRESS",
      "currentPlayerIndex": 0,
      "maxPlayers": 2,
      "deckSize": 55,
      "discardPile": [
        {
          "suit": "STAR",
          "value": 1
        }
      ],
      "players": [
        {
          "nickname": "Alice",
          "handSize": 6,
          "calledLastCard": false
        },
        {
          "nickname": "Baeline",
          "handSize": 6,
          "calledLastCard": false
        }
      ]
    }
  }
}
```

### My Hand
```graphql
query {
  myHand { suit value }
}
```

** How Output might be: **

```
{
  "data": {
    "myHand": [
      {
        "suit": "CIRCLE",
        "value": 4
      },
      {
        "suit": "CROSS",
        "value": 6
      },
      {
        "suit": "TRIANGLE",
        "value": 10
      },
      {
        "suit": "CIRCLE",
        "value": 3
      },
      {
        "suit": "SQUARE",
        "value": 2
      },
      {
        "suit": "STAR",
        "value": 2
      }
    ]
  }
}
```

### Match Info
```graphql
query {
  matchInfo {
    playerCount
    maxPlayers
    status
    deckSize
    topCard { suit value }
  }
}
```

### User Status
```graphql
query {
  userStatus
}
```

---

## Quick Test

```bash
# P1 (8080): subscribe → createMatch → startMatch → play
# P2 (8081): subscribe → joinMatch → play
```

**Endpoints:**
- P1: `http://localhost:8081/chains/{P1_CHAIN}/applications/{APP_ID}`
- P2: `http://localhost:8082/chains/{P2_CHAIN}/applications/{APP_ID}`
- PLAY: `http://localhost:8081/chains/{PLAY_CHAIN}/applications/{APP_ID}`


Okay. Is our query working successfully??

How to add this integration towards the frontend
I want to make the gameplay actually active and sync with the backend queries as ewell as mutations. 
