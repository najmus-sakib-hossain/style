"use client";

import React, { useState, useRef } from "react";
import LiquidGlass from "@/components/liquid-glass";
import Goo from "@/gooey-react/src/index";

export default function Home() {
  const [showPill, setShowPill] = useState(false);
  const [pillAnimating, setPillAnimating] = useState(false);
  const circleRef = useRef<HTMLDivElement>(null);
  const pillRef = useRef<HTMLDivElement>(null);

  const handleCircleClick = () => {
    if (pillAnimating) return; // Prevent multiple clicks during animation

    setPillAnimating(true);
    setShowPill(!showPill);

    // Reset animation state after transition
    setTimeout(() => {
      setPillAnimating(false);
    }, 1000); // Match animation duration
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900 flex items-center justify-center p-8">
      {/* Background pattern */}
      <div
        className="fixed inset-0 opacity-20"
        style={{
          backgroundImage: `
            linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px),
            linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px)
          `,
          backgroundSize: "50px 50px",
        }}
      />

      <div className="relative flex items-center gap-16">
        {/* Click Me Circle */}
        <div ref={circleRef} className="relative">
          <LiquidGlass
            mode="bubble"
            colorTint="blue"
            onClick={handleCircleClick}
            className={`transition-all duration-300 ${
              showPill ? "scale-110" : "scale-100"
            }`}
          >
            <div className="text-center">
              <div className="text-2xl mb-2">👆</div>
              <div className="text-white font-semibold">
                {showPill ? "Merge Back" : "Click Me"}
              </div>
            </div>
          </LiquidGlass>
        </div>

        {/* Gooey Pill Animation */}
        {showPill && (
          <div
            ref={pillRef}
            className={`absolute transition-all duration-1000 ease-out ${
              pillAnimating
                ? "animate-bounce-in-right"
                : "animate-bounce-out-left"
            }`}
            style={{
              right: showPill ? "0px" : "auto",
              left: showPill ? "auto" : "0px",
              transform: showPill
                ? "translateX(0) scale(1)"
                : "translateX(-100px) scale(0.8)",
            }}
          >
            <Goo intensity="strong" className="relative">
              <LiquidGlass
                mode="pill"
                colorTint="purple"
                width={200}
                height={80}
                className="shadow-2xl"
              >
                <span className="text-white font-semibold">Gooey Pill!</span>
              </LiquidGlass>
            </Goo>
          </div>
        )}
      </div>

      {/* Instructions */}
      <div className="absolute bottom-8 left-1/2 transform -translate-x-1/2 text-center">
        <div className="bg-black/20 backdrop-blur-sm rounded-xl p-4 border border-white/10 max-w-md">
          <p className="text-gray-300 text-sm">
            Click the circle to spawn a gooey pill with spring animation.
            Click again to merge it back with a gooey effect!
          </p>
        </div>
      </div>
    </div>
  );
}
