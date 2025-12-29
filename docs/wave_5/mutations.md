Perfect! Here's Your Frontend Mutation Map 

I've created a complete analysis showing exactly which components in your 
frontend/
 folder make mutations to your backend.

## The 3 Components That Make Mutations:
- app/game/page.tsx ← Main controller
- joinGame() → Creates/joins matches (subscribe, createMatch, joinMatch)
- startGame() → Starts the match (startMatch)
- components/PlayerOne.tsx

###  ← In-game actions
Card clicks → playCard() mutation
"LAST CARD!" button → callLastCard() mutation
components/DrawPile.tsx
###  ← Deck interaction
Deck click → drawCard() mutation

#### Central Hub:
All these components call functions from 
`hooks/useWhotGame.ts`, which is your single source of truth for all GraphQL mutations. 

This hook uses mutateUserChain()
 from 
lib/graphql.ts
 to send mutations to your Linera backend.

7 Total Mutations to Backend:
✅ subscribe - Subscribe to PLAY_CHAIN
✅ createMatch - Player 1 creates game
✅ joinMatch - Player 2 joins game
✅ startMatch - Player 1 starts match
✅ playCard - Play a card from hand
✅ drawCard - Draw from deck
✅ callLastCard - Call last card warning

All mutations go through the centralized 
`useWhotGame` hook, making your code clean and maintainable! 


```typescript
// lib/graphql.ts provides three key functions:

1. queryPlayChain()   // Query PLAY_CHAIN for match state
2. mutateUserChain()  // Send mutations to USER_CHAIN
3. loadConfig()       // Load deployment configuration
```

### Configuration Flow
1. [run.bash](file:///home/dinahmaccodes/Documents/codes-rust-linera/linot-card-game/run.bash) generates [config.json](file:///home/dinahmaccodes/Documents/codes-rust-linera/linot-card-game/frontend/tsconfig.json) during deployment
2. Frontend loads [config.json](file:///home/dinahmaccodes/Documents/codes-rust-linera/linot-card-game/frontend/tsconfig.json) via [loadConfig()](file:///home/dinahmaccodes/Documents/codes-rust-linera/linot-card-game/frontend/lib/graphql.ts#25-45)
3. GraphQL functions route to correct chains:
   - **Queries** → PLAY_CHAIN (authoritative game state)
   - **Mutations** → USER_CHAIN (player actions)

---

## File Structure Summary

```
linot-card-game/
├── frontend/              ← ✅ YOUR WORKING FRONTEND
│   ├── app/
│   │   ├── game/         ← Game UI components
│   │   ├── register/     ← Player registration
│   │   └── page.tsx      ← Landing page
│   ├── lib/
│   │   └── graphql.ts    ← ✅ Linera GraphQL integration
│   ├── components/       ← Shared UI components  
│   ├── public/           ← Static assets
│   └── .env.local        ← Environment config
│