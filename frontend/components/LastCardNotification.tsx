"use client";

import React, { useEffect, useState } from "react";

interface LastCardNotificationProps {
  opponentName: string;
  opponentHandSize: number;
  previousOpponentHandSize: number;
}

export default function LastCardNotification({
  opponentName,
  opponentHandSize,
  previousOpponentHandSize,
}: LastCardNotificationProps) {
  const [showNotification, setShowNotification] = useState(false);
  const [hasShownForCurrentState, setHasShownForCurrentState] = useState(false);

  useEffect(() => {
    // Show notification when opponent reaches exactly 1 card
    if (
      opponentHandSize === 1 &&
      previousOpponentHandSize > 1 && // Just went from >1 to 1
      !hasShownForCurrentState // Haven't shown notification for this state yet
    ) {
      console.log("[LastCardNotification] Opponent has 1 card left:", {
        opponentName,
        opponentHandSize,
        previousOpponentHandSize,
      });

      setShowNotification(true);
      setHasShownForCurrentState(true);
    }

    // Reset flag when hand size changes from 1
    if (opponentHandSize !== 1) {
      setHasShownForCurrentState(false);
    }
  }, [
    opponentHandSize,
    previousOpponentHandSize,
    hasShownForCurrentState,
    opponentName,
  ]);

  useEffect(() => {
    if (showNotification) {
      const timer = setTimeout(() => {
        setShowNotification(false);
      }, 5000); // Hide after 5 seconds (longer duration for warnings)

      return () => clearTimeout(timer);
    }
  }, [showNotification]);

  if (!showNotification) return null;

  return (
    <div className="fixed top-24 left-1/2 transform -translate-x-1/2 z-50 animate-bounce-in">
      <div className="relative bg-gradient-to-br from-yellow-400 via-orange-400 to-red-500 text-white px-8 py-5 rounded-2xl shadow-2xl border-2 border-yellow-300 backdrop-blur-sm">
        {/* Animated pulsing glow */}
        <div className="absolute inset-0 bg-gradient-to-r from-yellow-300 to-orange-300 rounded-2xl opacity-60 blur-lg animate-pulse"></div>

        {/* Content */}
        <div className="relative flex items-center gap-4">
          <span className="text-4xl animate-pulse">⚠️</span>
          <div>
            <p className="font-extrabold text-xl tracking-wide">
              {opponentName} HAS ONLY 1 CARD LEFT!
            </p>
            <p className="text-sm text-yellow-100 mt-1 font-semibold">
              🚨 Be careful! They're about to win!
            </p>
          </div>
          <span className="text-4xl animate-pulse">⚠️</span>
        </div>
      </div>
    </div>
  );
}
