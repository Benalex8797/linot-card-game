import React, { useEffect, useState, useRef } from "react";
import { ShoppingCart, X } from "lucide-react";

interface GeneralMarketNotificationProps {
  previousDeckSize: number;
  currentDeckSize: number;
  currentPlayerIndex: number;
  previousPlayerIndex: number;
  playerNumber: number; // 1-indexed local player number
  opponentName?: string;
  topCard: { suit: string; value: string | number } | null;
}

export default function GeneralMarketNotification({
  previousDeckSize,
  currentDeckSize,
  currentPlayerIndex,
  previousPlayerIndex,
  playerNumber,
  opponentName = "Opponent",
  topCard,
}: GeneralMarketNotificationProps) {
  const [showNotification, setShowNotification] = useState(false);
  const [message, setMessage] = useState("");

  // Track the last processed card to prevent duplicate notifications
  const lastProcessedCard = useRef<{
    suit: string;
    value: string | number;
  } | null>(null);
  const hasShownForCurrentCard = useRef(false);

  // Clear notification when tab becomes hidden
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.hidden) {
        setShowNotification(false);
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  useEffect(() => {
    const localPlayerIndex = playerNumber - 1;

    console.log("[GeneralMarketNotification] Checking for General Market:", {
      previousDeckSize,
      currentDeckSize,
      previousPlayerIndex,
      currentPlayerIndex,
      localPlayerIndex,
      opponentName,
      topCardValue: topCard?.value,
      topCardSuit: topCard?.suit,
      lastProcessedCard: lastProcessedCard.current,
    });

    // Skip initial state
    if (previousDeckSize === -1 || previousDeckSize === currentDeckSize) {
      return;
    }

    // CRITICAL FIX: Only show notification if card value 14 was actually played
    // This prevents false triggers from Pick 2/Pick 3 penalties
    if (!topCard || Number(topCard.value) !== 14) {
      console.log("[GeneralMarketNotification] Skipping - top card is not 14");
      return;
    }

    // Check if this is a NEW card 14 (not the same one we already processed)
    const isSameCard =
      lastProcessedCard.current &&
      lastProcessedCard.current.suit === topCard.suit &&
      Number(lastProcessedCard.current.value) === Number(topCard.value);

    if (isSameCard && hasShownForCurrentCard.current) {
      console.log(
        "[GeneralMarketNotification] Already shown notification for this card 14"
      );
      return;
    }

    // General Market detection: Deck decreased AND card 14 was played
    const deckDecreased = currentDeckSize < previousDeckSize;
    const cardsDrawn = previousDeckSize - currentDeckSize;
    const turnChanged = previousPlayerIndex !== currentPlayerIndex;

    // If deck decreased and card 14 was played, show notification
    if (
      deckDecreased &&
      turnChanged &&
      previousPlayerIndex !== localPlayerIndex
    ) {
      // In a 2-player game: 1 card drawn (the other player)
      // In a 3-player game: 2 cards drawn (2 other players)
      // In a 4-player game: 3 cards drawn (3 other players)

      if (cardsDrawn >= 1 && cardsDrawn <= 4) {
        setMessage(
          `${opponentName} played 14 for General Market so you get one extra card added`
        );
        setShowNotification(true);

        // Mark this card as processed
        lastProcessedCard.current = {
          suit: topCard.suit,
          value: topCard.value,
        };
        hasShownForCurrentCard.current = true;

        console.log(
          "[GeneralMarketNotification] Showing notification for card 14"
        );

        // Auto-hide after 5 seconds
        const timer = setTimeout(() => {
          setShowNotification(false);
        }, 5000);

        return () => clearTimeout(timer);
      }
    }
  }, [
    previousDeckSize,
    currentDeckSize,
    previousPlayerIndex,
    currentPlayerIndex,
    playerNumber,
    opponentName,
    topCard,
  ]);

  // Reset the "shown" flag when the top card changes to a different card
  useEffect(() => {
    if (topCard && lastProcessedCard.current) {
      const isDifferentCard =
        lastProcessedCard.current.suit !== topCard.suit ||
        Number(lastProcessedCard.current.value) !== Number(topCard.value);

      if (isDifferentCard) {
        hasShownForCurrentCard.current = false;
        console.log(
          "[GeneralMarketNotification] New card detected, resetting notification state"
        );
      }
    }
  }, [topCard]);

  if (!showNotification) return null;

  return (
    <div className="fixed top-40 left-1/2 transform -translate-x-1/2 z-50 animate-slideDown">
      <div
        className="bg-gradient-to-r from-purple-500 via-purple-600 to-purple-700 
                    text-white px-8 py-4 rounded-2xl shadow-2xl 
                    border-4 border-white flex items-center gap-4
                    min-w-[400px]"
      >
        <ShoppingCart className="w-8 h-8 animate-bounce" />
        <p className="text-xl font-bold flex-1">{message}</p>
        <button
          onClick={() => setShowNotification(false)}
          className="hover:bg-white/20 p-2 rounded-lg transition-colors"
          aria-label="Close notification"
        >
          <X className="w-6 h-6" />
        </button>
      </div>
    </div>
  );
}
