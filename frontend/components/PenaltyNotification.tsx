import React, { useEffect, useState } from "react";
import { AlertCircle, X } from "lucide-react";

interface PenaltyNotificationProps {
  pendingPenalty: number;
  previousPenalty?: number;
  opponentName?: string;
  currentPlayerName?: string;
  isMyTurn?: boolean; // Whether it's the affected player's turn (the one who has to draw)
}

export default function PenaltyNotification({
  pendingPenalty,
  previousPenalty = 0,
  opponentName = "Opponent",
  currentPlayerName = "You",
  isMyTurn = false,
}: PenaltyNotificationProps) {
  const [showNotification, setShowNotification] = useState(false);
  const [message, setMessage] = useState("");
  const [penaltySentByMe, setPenaltySentByMe] = useState(false); // Track if I sent the penalty
  const [isStackingNotification, setIsStackingNotification] = useState(false); // Track if this is a stacking notification

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
    console.log("[PenaltyNotification] Effect triggered:", {
      pendingPenalty,
      previousPenalty,
      isMyTurn,
      opponentName,
      currentPlayerName,
    });

    // Show warning to the player who SENT or STACKED the penalty card
    if (pendingPenalty > 0 && pendingPenalty !== previousPenalty) {
      console.log("[PenaltyNotification] Penalty increased:", {
        from: previousPenalty,
        to: pendingPenalty,
        isMyTurn,
        message: isMyTurn ? "WILL SHOW WARNING" : "Will NOT show warning",
      });

      // When I play penalty card, it's still my turn (isMyTurn=true)
      // I'm the one who sent the penalty, so show me the warning
      if (isMyTurn) {
        setPenaltySentByMe(true); // Mark that I sent this penalty

        // Check if this is stacking (previous penalty > 0)
        const isStacking = previousPenalty > 0;
        const stackedAmount = isStacking
          ? pendingPenalty - previousPenalty
          : pendingPenalty;

        setIsStackingNotification(isStacking); // Set flag for styling

        if (isStacking) {
          // Stacking scenario - SAME TYPE ONLY
          if (stackedAmount === 2 && previousPenalty === 2) {
            // Pick 2 blocking Pick 2 (2+2=4)
            setMessage(
              `🎴 You blocked with Pick 2! ${opponentName} now faces 4 cards total!`
            );
          } else if (stackedAmount === 3 && previousPenalty === 3) {
            // Pick 3 blocking Pick 3 (3+3=6)
            setMessage(
              `🎴 You blocked with Pick 3! ${opponentName} now faces 6 cards total!`
            );
          } else if (
            stackedAmount === 2 &&
            previousPenalty >= 4 &&
            previousPenalty % 2 === 0
          ) {
            // Multiple Pick 2 stacks (6, 8, 10, etc.)
            setMessage(
              `🎴 You blocked with Pick 2! ${opponentName} now faces ${pendingPenalty} cards total!`
            );
          } else if (
            stackedAmount === 3 &&
            previousPenalty >= 6 &&
            previousPenalty % 3 === 0
          ) {
            // Multiple Pick 3 stacks (9, 12, 15, etc.)
            setMessage(
              `🎴 You blocked with Pick 3! ${opponentName} now faces ${pendingPenalty} cards total!`
            );
          } else {
            // Generic stacking message (should not happen with correct backend logic)
            setMessage(
              `🎴 You stacked +${stackedAmount}! ${opponentName} now faces ${pendingPenalty} cards total!`
            );
          }
        } else {
          // Initial penalty - NOT a stack
          if (pendingPenalty === 2) {
            setMessage(
              `Pick 2 sent! ${opponentName} must draw 2 cards or block with another Pick 2.`
            );
          } else if (pendingPenalty === 3) {
            setMessage(
              `Pick 3 sent! ${opponentName} must draw 3 cards or block with another Pick 3.`
            );
          } else {
            setMessage(
              `Penalty sent! ${opponentName} must draw ${pendingPenalty} cards or block.`
            );
          }
        }

        console.log("[PenaltyNotification] Showing warning notification");
        setShowNotification(true);
      }
    } else if (pendingPenalty === 0 && previousPenalty > 0) {
      console.log("[PenaltyNotification] Penalty cleared:", {
        previousPenalty,
        isMyTurn,
        penaltySentByMe,
        message: !penaltySentByMe
          ? "WILL SHOW CONFIRMATION"
          : "Will NOT show confirmation (I sent it)",
      });

      // Penalty was cleared - show confirmation ONLY to player who RECEIVED the penalty
      // Don't show it to the player who sent it
      if (!penaltySentByMe) {
        setIsStackingNotification(previousPenalty > 3); // Mark as stacking if more than 3 cards drawn

        if (previousPenalty === 2) {
          setMessage(`📥 You drew 2 cards from ${opponentName}'s Pick 2.`);
        } else if (previousPenalty === 3) {
          setMessage(`📥 You drew 3 cards from ${opponentName}'s Pick 3.`);
        } else if (previousPenalty === 4) {
          setMessage(`📥 You drew 4 cards from stacked Pick 2's! (2+2)`);
        } else if (previousPenalty === 6) {
          setMessage(`📥 You drew 6 cards from stacked Pick 3's! (3+3)`);
        } else {
          setMessage(
            `📥 You drew ${previousPenalty} cards from stacked penalties!`
          );
        }

        console.log("[PenaltyNotification] Showing confirmation notification");
        setShowNotification(true);
        setPenaltySentByMe(false); // Reset the flag after showing confirmation
      }
    }
  }, [
    pendingPenalty,
    previousPenalty,
    opponentName,
    currentPlayerName,
    isMyTurn,
  ]);

  if (!showNotification) return null;

  return (
    <div className="fixed top-24 left-1/2 transform -translate-x-1/2 z-50 animate-slideDown">
      <div
        className="flex items-center gap-3 px-6 py-4 rounded-2xl shadow-2xl border-2 backdrop-blur-md max-w-md relative"
        style={{
          background: isStackingNotification
            ? "linear-gradient(135deg, rgba(156, 39, 176, 0.95), rgba(123, 31, 162, 0.95))" // Purple for stacking
            : "linear-gradient(135deg, rgba(234, 70, 61, 0.95), rgba(211, 47, 47, 0.95))", // Red for regular
          borderColor: isStackingNotification
            ? "rgba(156, 39, 176, 0.5)"
            : "rgba(234, 70, 61, 0.3)",
        }}
      >
        <AlertCircle className="w-6 h-6 text-white flex-shrink-0" />
        <p className="text-white font-satoshi font-bold text-base leading-tight flex-1">
          {message}
        </p>
        {/* Close button */}
        <button
          onClick={() => setShowNotification(false)}
          className="ml-2 text-white hover:bg-white/20 rounded-full p-1 transition-colors"
          aria-label="Close notification"
        >
          <X className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
}
