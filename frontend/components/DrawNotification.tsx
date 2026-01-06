"use client";

import React, { useEffect, useState } from "react";

interface DrawNotificationProps {
  opponentName: string;
  opponentHandSize: number;
  previousOpponentHandSize: number;
  isMyTurn: boolean;
}

export default function DrawNotification({
  opponentName,
  opponentHandSize,
  previousOpponentHandSize,
  isMyTurn,
}: DrawNotificationProps) {
  const [showNotification, setShowNotification] = useState(false);
  const [cardsDrawn, setCardsDrawn] = useState(0);

  useEffect(() => {
    // Detect when opponent manually draws cards (hand size increases during their turn)
    if (
      !isMyTurn && // Not my turn
      previousOpponentHandSize > 0 && // Previous hand size was tracked
      opponentHandSize > previousOpponentHandSize // Hand size increased
    ) {
      const drawn = opponentHandSize - previousOpponentHandSize;
      console.log("[DrawNotification] Opponent drew cards:", {
        opponentName,
        drawn,
        previousSize: previousOpponentHandSize,
        currentSize: opponentHandSize,
      });

      setCardsDrawn(drawn);
      setShowNotification(true);
    }
  }, [opponentHandSize, previousOpponentHandSize, isMyTurn, opponentName]);

  useEffect(() => {
    if (showNotification) {
      const timer = setTimeout(() => {
        setShowNotification(false);
      }, 3000); // Hide after 3 seconds

      return () => clearTimeout(timer);
    }
  }, [showNotification]);

  if (!showNotification) return null;

  return (
    <div className="fixed top-24 right-6 z-50 animate-slide-in-right">
      <div className="relative bg-gradient-to-br from-blue-500 via-blue-600 to-blue-700 text-white px-6 py-4 rounded-2xl shadow-2xl border-2 border-blue-400 backdrop-blur-sm">
        {/* Animated border glow */}
        <div className="absolute inset-0 bg-gradient-to-r from-blue-400 to-cyan-400 rounded-2xl opacity-50 blur-md animate-pulse"></div>

        {/* Content */}
        <div className="relative flex items-center gap-3">
          <span className="text-3xl animate-bounce">🎴</span>
          <div>
            <p className="font-bold text-lg">
              {opponentName} drew {cardsDrawn} card{cardsDrawn > 1 ? "s" : ""}{" "}
              from deck
            </p>
            <p className="text-sm text-blue-100 mt-1">
              Current hand: {opponentHandSize} card
              {opponentHandSize !== 1 ? "s" : ""}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
