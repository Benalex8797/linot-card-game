import React from "react";
import { X } from "lucide-react";

interface CardModalProps {
  isOpen: boolean;
  onClose: () => void;
  suit: "CIRCLE" | "TRIANGLE" | "CROSS" | "STAR" | "SQUARE" | "WHOT";
  value: string | number;
}

// Map suits to SVG file paths
const suitImages: Record<string, string> = {
  CIRCLE: "/suits/circle.svg",
  TRIANGLE: "/suits/triangle.svg",
  CROSS: "/suits/cross.svg",
  STAR: "/suits/star.svg",
  SQUARE: "/suits/square.svg",
  WHOT: "/suits/circle.svg",
};

// White versions for corner icons
const suitImagesWhite: Record<string, string> = {
  CIRCLE: "/suits/circle-white.svg",
  TRIANGLE: "/suits/triangle-white.svg",
  CROSS: "/suits/cross-white.svg",
  STAR: "/suits/star-white.svg",
  SQUARE: "/suits/square-white.svg",
  WHOT: "/suits/circle-white.svg",
};

export default function CardModal({
  isOpen,
  onClose,
  suit,
  value,
}: CardModalProps) {
  if (!isOpen) return null;

  const redColor = "#E23A2F";
  const isWhotCard = suit === "WHOT";

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
        aria-label="Close card view"
      >
        <X className="w-8 h-8" />
      </button>

      {/* Large Card */}
      <div
        className="relative z-[105] transform scale-150"
        onClick={(e) => e.stopPropagation()}
      >
        <div
          className="relative w-[150px] h-[202px] bg-white rounded-2xl overflow-hidden shadow-2xl"
          style={{
            border: `3px solid ${redColor}`,
          }}
        >
          {/* Top-left red corner tab */}
          <div
            className="absolute top-0 left-0 w-12 h-24 flex flex-col items-center justify-start pt-2 rounded-br-3xl"
            style={{ backgroundColor: redColor }}
          >
            <span className="text-white text-[24px] font-bold leading-none font-lilitaone mt-1.5">
              {value}
            </span>
            {!isWhotCard && (
              <img
                src={suitImagesWhite[suit]}
                alt={suit}
                className="w-5 h-5 mt-3"
              />
            )}
          </div>

          {/* Center large suit symbol or Linot branding */}
          <div className="absolute inset-0 flex items-center justify-center">
            {isWhotCard ? (
              <div className="text-center flex flex-col gap-3">
                {/* Outlined Linot text */}
                <span
                  className="text-5xl font-lilitaone"
                  style={{
                    WebkitTextStroke: "2px #E23A2F",
                    WebkitTextFillColor: "transparent",
                    color: "transparent",
                  }}
                >
                  Linot
                </span>
                {/* Filled Linot text */}
                <span className="text-5xl font-lilitaone text-[#E23A2F] ml-16">
                  Linot
                </span>
              </div>
            ) : (
              <img src={suitImages[suit]} alt={suit} className="w-32 h-32" />
            )}
          </div>

          {/* Bottom-right red corner tab (rotated) */}
          <div
            className="absolute bottom-0 right-0 w-12 h-24 flex flex-col items-center justify-start pt-2 rounded-br-3xl rotate-180"
            style={{ backgroundColor: redColor }}
          >
            <span className="text-white text-[24px] font-bold leading-none font-lilitaone mt-1.5">
              {value}
            </span>
            {!isWhotCard && (
              <img
                src={suitImagesWhite[suit]}
                alt={suit}
                className="w-5 h-5 mt-3"
              />
            )}
          </div>
        </div>

        {/* Info text */}
        <div className="text-center mt-6">
          <p className="text-white font-lilitaone text-2xl drop-shadow-lg">
            {isWhotCard ? "WHOT Card" : `${suit} ${value}`}
          </p>
          <p className="text-white/80 text-sm mt-1">Click anywhere to close</p>
        </div>
      </div>
    </div>
  );
}
