import React from "react";
import GameButton from "./GameButton";
import { AlertCircle } from "lucide-react";

interface DrawPileProps {
  deckSize: number;
  onDraw: () => void;
  pendingPenalty?: number;
}

export default function DrawPile({
  deckSize,
  onDraw,
  pendingPenalty = 0,
}: DrawPileProps) {
  return (
    <div className="flex flex-col items-center absolute bottom-50 gap-1.5">
      {/* Penalty Badge - Shows cards being removed from deck */}
      {pendingPenalty > 0 && (
        <div className="absolute -top-3 left-1/2 transform -translate-x-1/2 z-10 animate-pulse">
          <div className="bg-gradient-to-r from-red-500 to-orange-500 text-white px-3 py-1 rounded-full shadow-lg border-2 border-white flex items-center gap-1.5">
            <AlertCircle className="w-3 h-3" />
            <span className="font-bold text-xs">-{pendingPenalty}</span>
          </div>
        </div>
      )}
      <div>
        <img src="/drawcard.svg" alt="drawcard" />
      </div>
      <GameButton onClick={onDraw} backgroundColor="#E65150">
        Draw Pile ({deckSize})
      </GameButton>
    </div>
  );
}
