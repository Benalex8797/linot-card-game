import React, { useEffect, useState } from "react";
import { Clock, X } from "lucide-react";

interface HoldOnNotificationProps {
  previousPlayerIndex: number;
  currentPlayerIndex: number;
  playerNumber: number; // 1-indexed local player number
  playerName?: string;
  skippedPlayerName?: string;
}

export default function HoldOnNotification({
  previousPlayerIndex,
  currentPlayerIndex,
  playerNumber,
  playerName = "You",
  skippedPlayerName = "Opponent",
}: HoldOnNotificationProps) {
  const [showNotification, setShowNotification] = useState(false);
  const [message, setMessage] = useState("");

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
    // Convert 1-indexed player number to 0-indexed
    const localPlayerIndex = playerNumber - 1;

    console.log("[HoldOnNotification] Checking for turn skip:", {
      previousPlayerIndex,
      currentPlayerIndex,
      localPlayerIndex,
      playerName,
      skippedPlayerName,
    });

    // Skip initial state when previousPlayerIndex is -1 (not yet set)
    if (
      previousPlayerIndex === -1 ||
      previousPlayerIndex === currentPlayerIndex
    ) {
      return;
    }

    // Hold On detection: The backend's Hold On skips one player
    // Normal flow: Player 0 -> Player 1 -> Player 2 (in 3-player game)
    // With Hold On: Player 0 (plays Hold On) -> Player 2 (skipping Player 1)

    // In a 2-player game: Player 0 -> Player 1 normally
    // With Hold On from Player 0: Player 0 -> Player 0 again (skipping Player 1)
    // But the backend actually advances by 2, so: Player 0 -> next is (0+1)%2=1, skip to (1+1)%2=0

    // Check if I (localPlayerIndex) would have been next but got skipped
    // For 2 players: expectedNext = (previousPlayerIndex + 1) % 2
    // For 3 players: expectedNext = (previousPlayerIndex + 1) % 3
    // For 4 players: expectedNext = (previousPlayerIndex + 1) % 4

    // We don't know the exact number of players, but we can check for common cases
    const possibleExpectedNext = [2, 3, 4].map(
      (totalPlayers) => (previousPlayerIndex + 1) % totalPlayers
    );

    // If I should have been next but I'm not current, I got skipped
    if (
      possibleExpectedNext.includes(localPlayerIndex) &&
      currentPlayerIndex !== localPlayerIndex
    ) {
      setMessage(`⏸️ Hold On! Your turn was skipped.`);
      setShowNotification(true);
      console.log("[HoldOnNotification] You were skipped!");

      // Auto-hide after 4 seconds
      const timer = setTimeout(() => {
        setShowNotification(false);
      }, 4000);

      return () => clearTimeout(timer);
    }
  }, [
    previousPlayerIndex,
    currentPlayerIndex,
    playerNumber,
    playerName,
    skippedPlayerName,
  ]);

  if (!showNotification) return null;

  return (
    <div className="fixed top-32 left-1/2 transform -translate-x-1/2 z-50 animate-slideDown">
      <div
        className="bg-gradient-to-r from-blue-500 via-blue-600 to-blue-700 
                    text-white px-8 py-4 rounded-2xl shadow-2xl 
                    border-4 border-white flex items-center gap-4
                    min-w-[400px]"
      >
        <Clock className="w-8 h-8 animate-pulse" />
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
