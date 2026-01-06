import React from "react";

interface CardProps {
  suit: "CIRCLE" | "TRIANGLE" | "CROSS" | "STAR" | "SQUARE" | "WHOT";
  value: string | number;
  onClick?: () => void;
  isPlayable?: boolean;
  className?: string;
}

// Map suits to SVG file paths
const suitImages: Record<string, string> = {
  CIRCLE: "/suits/circle.svg",
  TRIANGLE: "/suits/triangle.svg",
  CROSS: "/suits/cross.svg",
  STAR: "/suits/star.svg",
  SQUARE: "/suits/square.svg",
  WHOT: "/suits/circle.svg", // Whot cards don't show suit (handled separately)
};

// White versions for corner icons
const suitImagesWhite: Record<string, string> = {
  CIRCLE: "/suits/circle-white.svg",
  TRIANGLE: "/suits/triangle-white.svg",
  CROSS: "/suits/cross-white.svg",
  STAR: "/suits/star-white.svg",
  SQUARE: "/suits/square-white.svg",
  WHOT: "/suits/circle-white.svg", // Whot cards don't show suit
};

export default function Card({
  suit,
  value,
  onClick,
  isPlayable = true,
  className = "",
}: CardProps) {
  const redColor = "#E23A2F";
  const isWhotCard = suit === "WHOT"; // Check suit instead of value

  // Determine if this card has a special effect
  const getSpecialEffect = () => {
    const numValue = value.toString();
    if (numValue === "1") return { label: "HOLD", color: "bg-blue-500" };
    if (numValue === "2") return { label: "P2", color: "bg-orange-500" };
    if (numValue === "5" && suit !== "STAR")
      return { label: "P3", color: "bg-red-500" };
    if (numValue === "14") return { label: "GM", color: "bg-purple-500" };
    if (numValue === "20") return { label: "WILD", color: "bg-yellow-500" };
    return null;
  };

  const specialEffect = getSpecialEffect();

  return (
    <div
      onClick={isPlayable ? onClick : undefined}
      className={`
        relative w-37.5 h-50.5 bg-white rounded-2xl overflow-hidden
        transition-all duration-200
        ${
          isPlayable
            ? "cursor-pointer hover:-translate-y-4 hover:shadow-2xl hover:z-50"
            : "opacity-70 cursor-not-allowed"
        }
        ${className}
      `}
      style={{
        border: `1.82px solid ${redColor}`,
        boxShadow: isPlayable
          ? "0 10px 30px rgba(0,0,0,0.3), 0 0 0 1px rgba(0,0,0,0.05)"
          : "0 4px 10px rgba(0,0,0,0.15)",
      }}
    >
      {/* Special Card Badge - Top Right */}
      {specialEffect && (
        <div
          className={`absolute top-1 right-1 ${specialEffect.color} text-white px-1.5 py-0.5 rounded-md text-[10px] font-bold shadow-lg z-10`}
        >
          {specialEffect.label}
        </div>
      )}

      {/* Top-left red corner tab */}
      <div
        className="absolute top-0 left-0 w-8 h-16 flex flex-col items-center justify-start pt-1.5 rounded-br-2xl "
        style={{ backgroundColor: redColor }}
      >
        <span className="text-white text-[16px] font-bold leading-none font-lilitaone mt-1">
          {value}
        </span>
        {!isWhotCard && (
          <img
            src={suitImagesWhite[suit]}
            alt={suit}
            className="w-3.5 h-3.5 mt-2"
          />
        )}
      </div>

      {/* Center large suit symbol or Linot branding for card 20 */}
      <div className="absolute inset-0 flex items-center justify-center">
        {isWhotCard ? (
          <div className="text-center flex flex-col gap-2">
            {/* Outlined Linot text */}
            <span
              className="text-3xl font-lilitaone"
              style={{
                WebkitTextStroke: "1px #E23A2F",
                WebkitTextFillColor: "transparent",
                color: "transparent",
              }}
            >
              Linot
            </span>
            {/* Filled Linot text */}
            <span className="text-3xl font-lilitaone text-[#E23A2F] ml-10">
              Linot
            </span>
          </div>
        ) : (
          <img src={suitImages[suit]} alt={suit} className="w-25 h-25" />
        )}
      </div>

      {/* Bottom-right red corner tab (rotated) */}
      <div
        className="absolute bottom-0 right-0  w-8 h-16 flex flex-col items-center justify-start pt-1.5 rounded-br-2xl rotate-180"
        style={{ backgroundColor: redColor }}
      >
        <span className="text-white text-[16px] font-bold leading-none font-lilitaone mt-1">
          {value}
        </span>
        {!isWhotCard && (
          <img
            src={suitImagesWhite[suit]}
            alt={suit}
            className="w-3.5 h-3.5 mt-2"
          />
        )}
      </div>
    </div>
  );
}
