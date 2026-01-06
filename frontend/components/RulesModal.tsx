import React from "react";
import { X, BookOpen } from "lucide-react";

interface RulesModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function RulesModal({ isOpen, onClose }: RulesModalProps) {
  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center"
      onClick={onClose}
    >
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/70 backdrop-blur-sm" />

      {/* Close button */}
      <button
        onClick={onClose}
        className="absolute top-8 right-8 z-[110] bg-white/20 hover:bg-white/30 text-white rounded-full p-3 transition-colors backdrop-blur-sm"
        aria-label="Close rules"
      >
        <X className="w-8 h-8" />
      </button>

      {/* Rules Card */}
      <div
        className="relative z-[105] max-w-3xl max-h-[80vh] overflow-y-auto bg-white rounded-3xl shadow-2xl p-10 mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-4 mb-6 pb-4 border-b-2 border-[#0FB6C6]">
          <BookOpen className="w-10 h-10 text-[#0FB6C6]" />
          <h1 className="text-4xl font-lilitaone text-[#01626F]">Game Rules</h1>
        </div>

        <div className="text-[#01626F] space-y-6 text-lg">
          <section>
            <h2 className="text-2xl font-bold mb-3 text-[#EA463D]">
              How to Play
            </h2>
            <p className="mb-3">
              <strong>Linot is simple!!</strong>
            </p>
            <p>
              Match the shape or number of the top card or draw if you can't.
              You can double play only when the cards share the same number.
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-bold mb-3 text-[#EA463D]">
              Special Cards
            </h2>
            <p className="mb-3">Special cards shake things up:</p>
            <ul className="space-y-2 list-disc list-inside ml-4">
              <li>
                <strong className="text-[#0FB6C6]">1 (Hold On)</strong> - skips
                the next player
              </li>
              <li>
                <strong className="text-[#EA463D]">2 (Pick Two)</strong> -
                forces the next player to draw 2 cards
              </li>
              <li>
                <strong className="text-[#EA463D]">5 (Pick Three)</strong> -
                forces the next player to draw 3 cards
              </li>
              <li>
                <strong className="text-purple-600">14 (General Market)</strong>{" "}
                - makes everyone except you draw 1 card
              </li>
              <li>
                <strong className="text-yellow-600">20 (Linot)</strong> - lets
                you pick a new shape that everyone must follow
              </li>
            </ul>
          </section>

          <section>
            <h2 className="text-2xl font-bold mb-3 text-[#EA463D]">
              Blocking Attacks
            </h2>
            <p>
              You can block any attack card (Pick Two, Pick Three) by matching
              its suit. Play a card of the same suit to defend yourself!
            </p>
          </section>

          <section>
            <h2 className="text-2xl font-bold mb-3 text-[#EA463D]">
              Winning the Game
            </h2>
            <p>
              Win by being the first to empty your hand or, when the deck pile
              runs out, by having the fewest cards.
            </p>
          </section>

          <section className="bg-[#77F0FC]/20 p-4 rounded-lg">
            <h3 className="font-bold text-xl mb-2">💡 Pro Tips</h3>
            <ul className="space-y-1 text-base">
              <li>• Save your special cards for strategic moments</li>
              <li>• Watch your opponent's card count</li>
              <li>• Use Linot (20) wisely to control the game</li>
              <li>
                • Remember: You can stack Pick 2s and Pick 3s to increase the
                penalty!
              </li>
            </ul>
          </section>
        </div>

        <button
          onClick={onClose}
          className="mt-8 w-full py-4 bg-[#0FB6C6] hover:bg-[#0da5b3] text-white text-xl font-bold rounded-xl transition-colors shadow-lg"
        >
          Got it! Let's Play
        </button>
      </div>
    </div>
  );
}
