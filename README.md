# Linot - Multiplayer Card Game on Linera
## Linot V0.5

> A fully on-chain, multiplayer implementation of the classic Whot card game, built on the Linera blockchain platform.

---

##  Project Overview

**Linot** is a decentralized multiplayer card game that demonstrates advanced cross-chain communication patterns on Linera. Players can create matches, join games across different chains, and play in real-time with fully synchronized game state.

### What We're Building

This project showcases:
- **True Multiplayer Gameplay**: Two players on separate chains playing synchronously
- **Cross-Chain State Management**: Game state maintained on a dedicated PLAY_CHAIN while players interact from their USER_CHAINs
- **Event-Driven Architecture**: Real-time state synchronization via Linera's event streaming
- **Production-Ready Patterns**: Implementing battle-tested multiplayer patterns for Linera applications

---

##  Wave 5 Submission

**Submission Type:** Multiplayer Implementation  
**Repository:** [github.com/dinahmaccodes/linot-card-game](https://github.com/dinahmaccodes/linot-card-game)

### Linera SDK & Protocol Features Used

Our implementation leverages key Linera platform capabilities:

**1. Cross-Chain Messaging**
- `runtime.prepare_message().send_to()` - Sending operations between chains
- Custom message types (`RequestJoin`, `JoinMatchConfirmed`, etc.)
- Asynchronous message handling

**2. Event Streaming**
- `runtime.subscribe_to_events()` - Real-time state synchronization
- Custom event types (`PlayerJoined`, `CardPlayed`, `MatchStarted`)
- Event-driven UI updates

**3. State Management**
- `linera_views` for persistent state
- Separate state contexts for USER_CHAIN and PLAY_CHAIN
- Optimistic updates with event confirmation

**4. GraphQL API**
- Schema-driven queries and mutations
- Chain-specific endpoints
- Real-time polling pattern

**5. Application Bytecode Propagation**
- Implicit bytecode registration via first message
- Deferred subscription pattern implementation
- Multi-chain application deployment

### Team Contact Information

| Name | Role | Socials(X & Discord) |
|------|------|---------|
| **Dinah Macaulay** | Lead Smart Contract Developer | [dinahmaccodes-on-github](https://github.com/dinahmaccodes) / [dinahmaccodes-on-discord](https://discordapp.com/users/dinahmaccodes)|
| **Divine Macaulay Egbezien** | Project Manager & UI/UX Designer | [divine_macaulay](https://x.com/divine_macaulay) |
| **Alex** | Frontend Developer | [Alex-Benjamin](https://github.com/Benalex8797) |

### Wave 5 Changelog

**New in Wave 5:**
- ✅ Implemented deferred subscription pattern for cross-chain multiplayer
- ✅ Fixed "client is not configured to propose" error
- ✅ Added automatic bytecode propagation handling
- ✅ Created comprehensive multiplayer flow (create/join/start/play)
- ✅ Implemented event-driven state synchronization
- ✅ Added GraphQL mutations for all game actions
- ✅ Built and Improved our dual-player frontend with real-time updates
- ✅ Documented architecture patterns for future reference

**Previous Waves:**
- Wave 4: Basic game logic implementation + Minimalistic UI development
- Wave 3: Smart contract development and Basic UI flow
- Wave 2: Project setup and planning
- Wave 1: Initial proposal

---

## Quick Start (Local Docker Deployment)

### Prerequisites

- Docker & Docker Compose
- Rust 1.86.0
- Node.js 20+

### Run the Game

```
# Clone and enter directory
cd linot-card-game
```

```
# Start services
docker compose up --build
```

**Access Points:**
- Player 1 Frontend: http://localhost:5173?player=1
- Player 2 Frontend: http://localhost:5174?player=2

---

## How to Play

1. **Player 1**: Register → Create Match
2. **Player 2**: Continues the flow by joining the match → Join Match  
3. **Player 1**: Start Match
4. **Gameplay**: Match suits or values(card numbers), first to empty their hand wins the game!

---

## Acknowledgments

### Linera Team

A huge thank you to the **Linera team** for:
- Creating an innovative blockchain platform that makes true decentralized applications possible
- Providing excellent documentation and reference implementations
- Being incredibly responsive and helpful on Discord
- Supporting developers building the future of decentralized applications

Your work is enabling the next generation of on-chain applications, and we're excited to be part of this journey!

---

## 👥 Team

**Linot** is built with ❤️ by:

- **Dinah Macaulay** - Lead Software Developer & Smart Contract Developer  
  [@dinahmaccodes-on-github](https://github.com/dinahmaccodes) | [dinahmaccodes-on-discord](https://discordapp.com/users/dinahmaccodes)
  
- **Divine Macaulay Egbezien** - Project Manager & Product Designer  
  [divine_macaulay](https://x.com/divine_macaulay) 
  
- **Alex** - Frontend Developer  
  [Alex Benjamin](https://github.com/Benalex8797) 

---

## Current Status

**Implemented:**
- Cross-chain multiplayer architecture
- Deferred subscription pattern (working!)
- Game state synchronization
- Full game logic (card dealing, play validation, special cards, win detection)
- Two-player frontend with real-time updates

**In Progress:**
- UI/UX refinements
- Special card effects frontend integration
- Win/lose modal flows

---

## Technical Stack

**Backend:**
- Rust (Linera SDK)
- Custom state management
- Event-driven architecture

**Frontend:**
- Next.js 15
- TypeScript
- GraphQL (queries & mutations)
- Real-time polling (500ms)

**Infrastructure:**
- Docker & Docker Compose
- Linera local network
- Multi-chain deployment

---

##  Game Rules (Whot)

- Each player starts with 6 cards
- Match the **suit** or **value** of the top card
- Special Cards:
  - **2**: Pick Two (next player draws 2)
  - **5**: Pick Three (next player draws 3)
  - **1**: Hold On (skip next player)
  - **14**: General Market (all players draw)
  - **20**: Whot (wild card, choose suit)
- First player to empty their hand wins!

---

##  Contributing

We welcome feedback and contributions! Feel free to:
- Open issues for bugs or suggestions
- Submit PRs for improvements
- Reach out to Linot game on X [linotgame](https://x.com/linotgame)

---

##  More Context

This project is being developed as part of the Linera ecosystem and follows open-source principles.

---

**Built on Linera**  | **Deployed Locally**  | **Multiplayer Ready** 
