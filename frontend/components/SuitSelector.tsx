"use client";
import React from 'react';

interface SuitSelectorProps {
  onSelect: (suit: string) => void;
  onCancel: () => void;
}

export default function SuitSelector({ onSelect, onCancel }: SuitSelectorProps) {
  const suits = [
    { name: 'CIRCLE', icon: '/suits/circle-white.svg' },
    { name: 'TRIANGLE', icon: '/suits/triangle-white.svg' },
    { name: 'CROSS', icon: '/suits/cross-white.svg' },
    { name: 'SQUARE', icon: '/suits/square-white.svg' },
    { name: 'STAR', icon: '/suits/star-white.svg' },
  ];

  return (
    <div 
      className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 backdrop-blur-sm"
      onClick={onCancel}
    >
      <div 
        className="bg-gradient-to-br from-[#0FB6C6] to-[#01626F] p-8 rounded-3xl shadow-2xl border-4 border-white/30"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-3xl font-lilitaone text-white text-center mb-6">
          Choose a Suit
        </h2>
        <div className="grid grid-cols-3 gap-4">
          {suits.map(suit => (
            <button
              key={suit.name}
              onClick={() => onSelect(suit.name)}
              className="p-6 bg-white/20 hover:bg-white/40 rounded-xl border-2 border-white/50 
                         transform transition hover:scale-110 active:scale-95 hover:shadow-xl"
            >
              <img 
                src={suit.icon} 
                alt={suit.name}
                className="w-16 h-16 mx-auto"
              />
              <p className="text-white text-sm font-bold mt-2">{suit.name}</p>
            </button>
          ))}
        </div>
        <button
          onClick={onCancel}
          className="mt-6 w-full px-4 py-2 bg-red-500 hover:bg-red-600 text-white font-bold rounded-lg
                     transform transition active:scale-95"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
