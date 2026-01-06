"use client";
import React, { useState, useEffect, Suspense } from "react";
import { useSearchParams } from "next/navigation";
import LiveChat from "../../components/LiveChat";
import Timer from "../../components/Timer";
import GamePlayersTab from "../../components/GamePlayersTab";
import DrawPile from "../../components/DrawPile";
import PlayerTwo from "../../components/PlayerTwo";
import PlayerOne from "../../components/PlayerOne";
import WinModal from "../../components/WinModal";
import LoseModal from "../../components/LoseModal";
import Card from "../../components/Card";
import PenaltyNotification from "../../components/PenaltyNotification";
import HoldOnNotification from "../../components/HoldOnNotification";
import GeneralMarketNotification from "../../components/GeneralMarketNotification";
import DrawNotification from "../../components/DrawNotification";
import LastCardNotification from "../../components/LastCardNotification";
import { useWhotGame } from "../../hooks/useWhotGame";
import { OpponentView } from "../../lib/types";
import Navbar from "../../components/Navbar";

function GameClient() {
  const searchParams = useSearchParams();
  const playerNumberParam = searchParams.get("player");
  const playerNumber = (playerNumberParam === "2" ? 2 : 1) as 1 | 2;

  const {
    gameState,
    loading,
    error,
    joinGame,
    startGame,
    playCard,
    drawCard,
    callLastCard,
  } = useWhotGame(playerNumber);

  // Extract active demand suit for visual indicator (null-safe)
  const activeShapeDemand = gameState?.activeShapeDemand;
  const [username, setUsername] = useState<string>("");
  const [maxPlayers, setMaxPlayers] = useState<number>(2);

  // Modal states for testing/editing
  const [showWinModal, setShowWinModal] = useState(false);
  const [showLoseModal, setShowLoseModal] = useState(false);

  // Track previous penalty for notification changes
  const [previousPenalty, setPreviousPenalty] = useState(0);

  // Track previous player index for Hold On detection
  const [previousPlayerIndex, setPreviousPlayerIndex] = useState<number>(-1);

  // Track previous deck size for General Market detection
  const [previousDeckSize, setPreviousDeckSize] = useState<number>(-1);

  // Track previous opponent hand size for Draw notification
  const [previousOpponentHandSize, setPreviousOpponentHandSize] =
    useState<number>(0);

  useEffect(() => {
    const storedProfile = localStorage.getItem(
      `whot_player_profile_${playerNumber}`
    );
    if (storedProfile) {
      try {
        const profile = JSON.parse(storedProfile);
        if (profile.username) {
          setUsername(profile.username);
        }
        if (profile.maxPlayers) {
          setMaxPlayers(profile.maxPlayers);
        }
      } catch (e) {
        console.error("Failed to parse profile", e);
      }
    }
  }, [playerNumber]);

  // Track penalty changes for notifications
  useEffect(() => {
    if (gameState) {
      setPreviousPenalty(gameState.pendingPenalty);
    }
  }, [gameState?.pendingPenalty]);

  // Track player index changes for Hold On notification
  useEffect(() => {
    if (gameState && gameState.currentPlayerIndex !== previousPlayerIndex) {
      setPreviousPlayerIndex(gameState.currentPlayerIndex);
    }
  }, [gameState?.currentPlayerIndex, previousPlayerIndex]);

  // Track deck size changes for General Market notification
  useEffect(() => {
    if (gameState && gameState.deckSize !== previousDeckSize) {
      setPreviousDeckSize(gameState.deckSize);
    }
  }, [gameState?.deckSize, previousDeckSize]);

  // Track opponent hand size changes for Draw notification
  useEffect(() => {
    if (gameState && gameState.opponents.length > 0) {
      const opponentHandSize = gameState.opponents[0].cardCount;
      if (previousOpponentHandSize === 0) {
        // Initialize
        setPreviousOpponentHandSize(opponentHandSize);
      } else if (opponentHandSize !== previousOpponentHandSize) {
        setPreviousOpponentHandSize(opponentHandSize);
      }
    }
  }, [gameState?.opponents, previousOpponentHandSize]);

  // Detect game ending and show win/lose modal
  useEffect(() => {
    if (
      gameState &&
      gameState.status.toUpperCase() === "FINISHED" &&
      gameState.winnerIndex !== null
    ) {
      // Player numbers are 1-indexed, winnerIndex is 0-indexed
      const localPlayerIndex = playerNumber - 1;
      const didIWin = gameState.winnerIndex === localPlayerIndex;

      console.log(
        "[Game] Game finished! Winner index:",
        gameState.winnerIndex,
        "Local player index:",
        localPlayerIndex,
        "Did I win:",
        didIWin
      );

      if (didIWin) {
        setShowWinModal(true);
      } else {
        setShowLoseModal(true);
      }
    }
  }, [gameState?.status, gameState?.winnerIndex, playerNumber]);

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#77F0FC]">
        <p className="text-3xl font-lilitaone text-[#01626F] animate-pulse">
          Loading game...
        </p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center flex-col gap-4 bg-[#77F0FC]">
        <p className="text-2xl text-red-600 font-bold">Connection Error</p>
        <p className="text-[#01626F]">{error.message}</p>
        <button
          onClick={() => window.location.reload()}
          className="px-6 py-2 bg-[#0FB6C6] text-white rounded-lg"
        >
          Retry
        </button>
      </div>
    );
  }

  // 1. Join Screen
  if (!gameState) {
    return (
      <div
        className="min-h-screen w-full relative overflow-hidden flex flex-col items-center justify-center"
        style={{
          background: "linear-gradient(180deg, #77F0FC 0%, #19D3F9 100%)",
        }}
      >
        <img
          src="/water-bubbles.svg"
          className="absolute z-3 top-0 left-[150px] animate-bubbles animation-delay-2000"
          alt=""
        />
        <img
          src="/sea-walls.png"
          className="absolute z-3 top-0 left-0"
          alt=""
        />
        <img
          src="/reflection-lights.svg"
          className="absolute z-3 top-0 left-0 animate-shimmer"
          alt=""
        />
        <div className="z-40 bg-white/30 backdrop-blur-md p-10 rounded-2xl shadow-xl flex flex-col items-center gap-6 border border-white/50">
          <h1 className="text-4xl font-lilitaone text-[#01626F]">
            Welcome {username || `Player ${playerNumber}`}
          </h1>
          {playerNumber === 1 ? (
            <>
              <p className="text-[#01626F] text-lg text-center">
                Create a new game for {maxPlayers} players
              </p>
              <button
                onClick={() =>
                  joinGame(username || `Player ${playerNumber}`, maxPlayers)
                }
                className="px-8 py-4 bg-[#EA463D] hover:bg-[#d63a32] text-white text-2xl font-lilitaone rounded-full shadow-lg transform transition active:scale-95"
              >
                Create Game ({maxPlayers} Players)
              </button>
            </>
          ) : (
            <>
              <p className="text-[#01626F] text-lg text-center">
                Join an existing game created by Player 1.
              </p>
              <button
                onClick={() =>
                  joinGame(username || `Player ${playerNumber}`, maxPlayers)
                }
                className="px-8 py-4 bg-[#EA463D] hover:bg-[#d63a32] text-white text-2xl font-lilitaone rounded-full shadow-lg transform transition active:scale-95"
              >
                Join Game
              </button>
            </>
          )}
        </div>
        <div
          className="absolute bottom-0 w-full h-[150px] z-10 rounded-t-[50%]"
          style={{
            background:
              "linear-gradient(180deg, #D4FEBC -13.12%, #85F0D9 21.68%, #19E2D5 100%)",
            boxShadow: "0px 12px 18px 0px #E8FABC inset",
          }}
        ></div>

        {/* <div className="absolute bottom-4 left-4 z-30 w-[384px] h-[214px] animate-float animate-float-delayed">
          <img
            src="/cards.svg"
            alt="Cards"
            className="w-full h-full object-contain"
          />
        </div> */}

        <div className="absolute bottom-[-55px] left-0 z-20 w-[590px] h-[654px] opacity-90 animate-sway">
          <img
            src="/weed-rock.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>

        <div className="absolute bottom-0 right-[-110px] z-20 w-[530px] h-[800px] opacity-90 animate-sway">
          <img
            src="/sea-weed-1.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>
        <div className="absolute bottom-0 right-0 z-20 w-[530px] h-[800px] opacity-90 animate-sway">
          <img
            src="/sea-weed-2.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>
      </div>
    );
  }

  // 2. Waiting Screen (Status check: Case-insensitive)
  const status = gameState.status.toUpperCase();
  if (status === "WAITING") {
    return (
      <div
        className="min-h-screen w-full relative overflow-hidden flex flex-col items-center justify-center"
        style={{
          background: "linear-gradient(180deg, #77F0FC 0%, #19D3F9 100%)",
        }}
      >
        <img
          src="/water-bubbles.svg"
          className="absolute z-3 top-0 left-[150px] animate-bubbles animation-delay-2000"
          alt=""
        />
        <img
          src="/sea-walls.png"
          className="absolute z-3 top-0 left-0"
          alt=""
        />
        <img
          src="/reflection-lights.svg"
          className="absolute z-3 top-0 left-0 animate-shimmer"
          alt=""
        />
        <div className="z-40 bg-white/30 backdrop-blur-md p-10 rounded-2xl shadow-xl flex flex-col items-center gap-6 w-[500px]">
          <h1 className="text-4xl font-lilitaone text-[#01626F]">Lobby</h1>
          <div className="w-full bg-white/50 rounded-lg p-4">
            <h2 className="text-xl font-bold text-[#01626F] mb-2">
              Players Joined:
            </h2>
            <ul className="space-y-2">
              {gameState.opponents.map((opp: OpponentView, idx: number) => (
                <li
                  key={idx}
                  className="flex justify-between items-center text-[#01626F]"
                >
                  <span>{opp.nickname || "Unknown"}</span>
                  <span className="bg-green-400 text-white text-xs px-2 py-1 rounded-full">
                    Ready
                  </span>
                </li>
              ))}
              <li className="flex justify-between items-center text-[#01626F] font-bold">
                <span>You (Player {playerNumber})</span>
                <span className="bg-green-400 text-white text-xs px-2 py-1 rounded-full">
                  Ready
                </span>
              </li>
            </ul>
          </div>

          {playerNumber === 1 ? (
            <button
              onClick={() => startGame()}
              disabled={gameState.opponents.length < 1}
              className={`px-8 py-4 text-white text-2xl font-lilitaone rounded-full shadow-lg transform transition ${
                gameState.opponents.length < 1
                  ? "bg-gray-400 cursor-not-allowed"
                  : "bg-[#EA463D] hover:bg-[#d63a32] active:scale-95"
              }`}
            >
              Start Match
            </button>
          ) : (
            <div className="text-center">
              <p className="text-lg text-[#01626F] animate-pulse font-bold">
                Waiting for host to start...
              </p>
            </div>
          )}
        </div>
        <div
          className="absolute bottom-0 w-full h-[150px] z-10 rounded-t-[50%]"
          style={{
            background:
              "linear-gradient(180deg, #D4FEBC -13.12%, #85F0D9 21.68%, #19E2D5 100%)",
            boxShadow: "0px 12px 18px 0px #E8FABC inset",
          }}
        ></div>

        {/* <div className="absolute bottom-4 left-4 z-30 w-[384px] h-[214px] animate-float animate-float-delayed">
          <img
            src="/cards.svg"
            alt="Cards"
            className="w-full h-full object-contain"
          />
        </div> */}

        <div className="absolute bottom-[-55px] left-0 z-20 w-[590px] h-[654px] opacity-90 animate-sway">
          <img
            src="/weed-rock.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>

        <div className="absolute bottom-0 right-[-110px] z-20 w-[530px] h-[800px] opacity-90 animate-sway">
          <img
            src="/sea-weed-1.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>
        <div className="absolute bottom-0 right-0 z-20 w-[530px] h-[800px] opacity-90 animate-sway">
          <img
            src="/sea-weed-2.svg"
            alt="Seaweed"
            className="w-full h-full object-contain"
          />
        </div>
      </div>
    );
  }

  // 3. Game Board (In Progress or Finished)
  return (
    <div
      className="min-h-screen w-full relative overflow-y-auto flex flex-col scroll-smooth"
      style={{
        background: "linear-gradient(180deg, #77F0FC 0%, #19D3F9 100%)",
      }}
    >
      <img
        src="/water-bubbles.svg"
        className="fixed z-3 top-0 left-[150px] animate-bubbles animation-delay-2000 pointer-events-none"
        alt=""
      />
      <img
        src="/sea-walls.png"
        className="fixed z-3 top-0 left-0 pointer-events-none"
        alt=""
      />
      <img
        src="/reflection-lights.svg"
        className="fixed z-3 top-0 left-0 animate-shimmer pointer-events-none"
        alt=""
      />
      <Navbar />

      {/* Penalty Card Notification */}
      <PenaltyNotification
        pendingPenalty={gameState.pendingPenalty}
        previousPenalty={previousPenalty}
        opponentName={gameState.opponents[0]?.nickname || "Opponent"}
        currentPlayerName={username || `Player ${playerNumber}`}
        isMyTurn={gameState.currentPlayerIndex !== playerNumber - 1}
      />

      {/* Hold On Notification */}
      <HoldOnNotification
        previousPlayerIndex={previousPlayerIndex}
        currentPlayerIndex={gameState.currentPlayerIndex}
        playerNumber={playerNumber}
        playerName={username || `Player ${playerNumber}`}
        skippedPlayerName={
          gameState.opponents.find(
            (_, idx) => idx + 1 === previousPlayerIndex + 1
          )?.nickname || "Opponent"
        }
      />

      {/* Draw Notification - shows when opponent draws cards */}
      <DrawNotification
        opponentName={gameState.opponents[0]?.nickname || "Opponent"}
        opponentHandSize={gameState.opponents[0]?.cardCount || 0}
        previousOpponentHandSize={previousOpponentHandSize}
        isMyTurn={gameState.currentPlayerIndex === playerNumber - 1}
      />

      {/* Last Card Notification - shows when opponent has 1 card left */}
      <LastCardNotification
        opponentName={gameState.opponents[0]?.nickname || "Opponent"}
        opponentHandSize={gameState.opponents[0]?.cardCount || 0}
        previousOpponentHandSize={previousOpponentHandSize}
      />

      {/* General Market Notification */}
      <GeneralMarketNotification
        previousDeckSize={previousDeckSize}
        currentDeckSize={gameState.deckSize}
        currentPlayerIndex={gameState.currentPlayerIndex}
        previousPlayerIndex={previousPlayerIndex}
        playerNumber={playerNumber}
        opponentName={gameState.opponents[0]?.nickname || "Opponent"}
        topCard={gameState.topCard}
      />

      {/* Active Demand Suit Indicator */}
      {activeShapeDemand && (
        <div
          className="absolute top-24 left-1/2 transform -translate-x-1/2 z-50 
                      bg-gradient-to-r from-yellow-400 via-orange-500 to-red-500 
                      px-8 py-4 rounded-2xl shadow-2xl border-4 border-white
                      animate-pulse"
        >
          <div className="flex items-center gap-4">
            <img
              src={`/suits/${activeShapeDemand.toLowerCase()}.svg`}
              alt={activeShapeDemand}
              className="w-12 h-12 filter drop-shadow-lg"
            />
            <div>
              <p className="text-white font-lilitaone text-2xl drop-shadow-lg">
                SUIT DEMANDED!
              </p>
              <p className="text-white font-bold text-lg">
                Play {activeShapeDemand} cards
              </p>
            </div>
            <img
              src={`/suits/${activeShapeDemand.toLowerCase()}.svg`}
              alt={activeShapeDemand}
              className="w-12 h-12 filter drop-shadow-lg"
            />
          </div>
        </div>
      )}
      <div className="flex-1 w-full relative overflow-visible flex gap-x-12.5 px-5">
        <div className="min-w-max flex gap-x-12.5 w-full">
          <div className="space-y-12 pt-10 z-40">
            <LiveChat />
            <div className="mt-10">
              <Timer />
            </div>
          </div>
          <div className="flex-1 space-y-20 pt-5 z-40 min-w-[600px]">
            <PlayerTwo
              opponent={gameState.opponents[0]}
              topCard={gameState.topCard}
            />

            {/* Discard Pile - Shows the current card on table */}
            <div className="mx-auto flex flex-col items-center gap-4">
              <p className="text-white font-lilitaone text-lg">Card on Table</p>
              {gameState.topCard ? (
                <div className="transform scale-110 shadow-2xl">
                  <Card
                    suit={gameState.topCard.suit as any}
                    value={gameState.topCard.value}
                    isPlayable={false}
                  />
                </div>
              ) : (
                <div className="w-37.5 h-50.5 bg-white/20 rounded-2xl border-4 border-dashed border-white/50 flex items-center justify-center">
                  <p className="text-white/50 font-bold">No cards yet</p>
                </div>
              )}
              <div className="text-white/80 text-sm">
                {gameState.deckSize} cards left in deck
              </div>
            </div>

            <PlayerOne
              myCards={gameState.myCards}
              isMyTurn={
                gameState.status === "IN_PROGRESS" &&
                gameState.currentPlayerIndex === playerNumber - 1
              }
              topCard={gameState.topCard}
              activeDemandSuit={gameState.activeShapeDemand}
              opponentName={gameState.opponents[0]?.nickname}
              onPlayCard={playCard}
              onDrawCard={drawCard}
              onCallLastCard={callLastCard}
            />
          </div>
          <div className="space-y-[46px] z-40 mr-5">
            <GamePlayersTab
              playerNumber={playerNumber as 1 | 2}
              gameState={gameState}
            />
            <DrawPile
              deckSize={gameState.deckSize}
              onDraw={drawCard}
              pendingPenalty={gameState.pendingPenalty}
            />
          </div>
        </div>
      </div>

      {/* TEST BUTTONS - For Modal Design Testing */}
      {/* <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50 flex gap-4">
        <button
          onClick={() => setShowWinModal(true)}
          className="px-6 py-3 bg-green-500 hover:bg-green-600 text-white font-lilitaone rounded-lg shadow-lg transition"
        >
          🏆 Test Win Modal
        </button>
        <button
          onClick={() => setShowLoseModal(true)}
          className="px-6 py-3 bg-red-500 hover:bg-red-600 text-white font-lilitaone rounded-lg shadow-lg transition"
        >
          😢 Test Lose Modal
        </button>
      </div> */}

      {/* Modals */}
      <WinModal
        isOpen={showWinModal}
        onClose={() => setShowWinModal(false)}
        clearTime="4:14"
        stars={3}
        coinsEarned={30}
        beatTheTime={true}
        specialCardsPlayed={2}
        onReplay={() => {
          console.log("Replay clicked - Reloading game");
          window.location.reload();
        }}
        onLobby={() => {
          console.log("Lobby clicked - Going home");
          window.location.href = "/";
        }}
      />

      <LoseModal
        isOpen={showLoseModal}
        onClose={() => setShowLoseModal(false)}
        reason="Your opponent won!"
        onReplay={() => {
          console.log("Replay clicked - Clearing state and reloading");
          // Clear localStorage to force a fresh game
          localStorage.removeItem(`whot_player_profile_${playerNumber}`);
          // Force a full page reload to reset all state
          window.location.href = `/game?player=${playerNumber}`;
        }}
        onLobby={() => {
          console.log("Lobby clicked - Going home");
          // Clear localStorage before going to lobby
          localStorage.removeItem(`whot_player_profile_${playerNumber}`);
          window.location.href = "/";
        }}
      />
    </div>
  );
}

export default function GamePage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center bg-[#77F0FC]">
          <p className="text-3xl font-lilitaone text-[#01626F] animate-pulse">
            Loading...
          </p>
        </div>
      }
    >
      <GameClient />
    </Suspense>
  );
}
