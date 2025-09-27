"use client";

import React, { useState } from "react";
import LiquidGlass, { presets } from "@/components/liquid-glass";

export default function Home() {
  const [selectedMode, setSelectedMode] =
    useState<keyof typeof presets>("dock");
  const [isDraggable, setIsDraggable] = useState(true);

  return (
    <div className="min-h-screen bg-gradient-to-br from-purple-900 via-blue-900 to-indigo-900 p-8">
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

      <div className="relative z-10">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-6xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
            Liquid Glass
          </h1>
          <p className="text-xl text-gray-300 max-w-2xl mx-auto">
            A beautiful liquid glass effect component with SVG displacement
            mapping, elastic transformations, and customizable presets.
          </p>
        </div>

        {/* Controls */}
        <div className="flex justify-center mb-12">
          <div className="bg-black/20 backdrop-blur-sm rounded-xl p-6 border border-white/10">
            <div className="flex flex-col sm:flex-row gap-4 items-center">
              <div className="flex gap-2">
                <label className="text-white text-sm">Mode:</label>
                <select
                  value={selectedMode}
                  onChange={(e) =>
                    setSelectedMode(e.target.value as keyof typeof presets)
                  }
                  className="bg-white/10 text-white rounded px-3 py-1 text-sm border border-white/20"
                >
                  {Object.keys(presets).map((mode) => (
                    <option key={mode} value={mode} className="bg-gray-800">
                      {mode.charAt(0).toUpperCase() + mode.slice(1)}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="draggable"
                  checked={isDraggable}
                  onChange={(e) => setIsDraggable(e.target.checked)}
                  className="w-4 h-4"
                />
                <label htmlFor="draggable" className="text-white text-sm">
                  Draggable
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Main Demo */}
        <div className="flex justify-center mb-16">
          <LiquidGlass
            mode={selectedMode}
            draggable={isDraggable}
            onClick={() => console.log("Liquid glass clicked!")}
            className="shadow-2xl"
          >
            <div className="flex items-center gap-3">
              {selectedMode === "dock" && (
                <>
                  <div className="w-12 h-12 bg-gradient-to-br from-blue-400 to-blue-600 rounded-lg"></div>
                  <div className="w-12 h-12 bg-gradient-to-br from-green-400 to-green-600 rounded-lg"></div>
                  <div className="w-12 h-12 bg-gradient-to-br from-red-400 to-red-600 rounded-lg"></div>
                  <div className="w-12 h-12 bg-gradient-to-br from-yellow-400 to-yellow-600 rounded-lg"></div>
                </>
              )}
              {selectedMode === "pill" && (
                <span className="text-white font-semibold">Pill Button</span>
              )}
              {selectedMode === "bubble" && (
                <div className="w-16 h-16 bg-gradient-to-br from-purple-400 to-pink-600 rounded-full"></div>
              )}
              {selectedMode === "free" && (
                <div className="text-center">
                  <div className="text-2xl mb-2">🎨</div>
                  <div className="text-sm">Creative</div>
                </div>
              )}
              {selectedMode === "standard" && (
                <span className="text-white font-semibold">Standard Glass</span>
              )}
            </div>
          </LiquidGlass>
        </div>

        {/* Features Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 mb-16">
          {/* Feature 1 */}
          <div className="text-center">
            <LiquidGlass mode="bubble" className="mb-4">
              <div className="text-2xl">✨</div>
            </LiquidGlass>
            <h3 className="text-xl font-semibold text-white mb-2">
              SVG Displacement
            </h3>
            <p className="text-gray-300 text-sm">
              Advanced SVG filters create realistic glass displacement effects
            </p>
          </div>

          {/* Feature 2 */}
          <div className="text-center">
            <LiquidGlass mode="pill" elasticity={0.3} className="mb-4">
              <span className="text-white font-medium">Elastic</span>
            </LiquidGlass>
            <h3 className="text-xl font-semibold text-white mb-2">
              Elastic Motion
            </h3>
            <p className="text-gray-300 text-sm">
              Smooth elastic transformations that respond to mouse movement
            </p>
          </div>

          {/* Feature 3 */}
          <div className="text-center">
            <LiquidGlass mode="standard" draggable={true} className="mb-4">
              <span className="text-white font-medium">Drag Me</span>
            </LiquidGlass>
            <h3 className="text-xl font-semibold text-white mb-2">
              Interactive
            </h3>
            <p className="text-gray-300 text-sm">
              Fully interactive with drag support and hover effects
            </p>
          </div>
        </div>

        {/* Preset Showcase */}
        <div className="text-center mb-8">
          <h2 className="text-3xl font-bold text-white mb-8">
            Available Presets
          </h2>
          <div className="flex flex-wrap justify-center gap-8">
            {Object.keys(presets).map((preset) => (
              <div key={preset} className="text-center">
                <LiquidGlass
                  mode={preset as keyof typeof presets}
                  onClick={() =>
                    setSelectedMode(preset as keyof typeof presets)
                  }
                  className="mb-3 cursor-pointer hover:scale-105 transition-transform"
                >
                  <div className="text-white font-medium text-sm">
                    {preset.charAt(0).toUpperCase() + preset.slice(1)}
                  </div>
                </LiquidGlass>
                <p className="text-gray-400 text-xs">
                  {presets[preset as keyof typeof presets].width} ×{" "}
                  {presets[preset as keyof typeof presets].height}
                </p>
              </div>
            ))}
          </div>
        </div>

        {/* Instructions */}
        <div className="text-center mt-16">
          <div className="bg-black/20 backdrop-blur-sm rounded-xl p-6 border border-white/10 max-w-2xl mx-auto">
            <h3 className="text-xl font-semibold text-white mb-4">
              How to Use
            </h3>
            <div className="text-gray-300 text-sm space-y-2">
              <p>• Hover over components to see elastic effects</p>
              <p>• Enable draggable mode to move components around</p>
              <p>• Click on preset examples to switch modes</p>
              <p>• Watch the liquid glass displacement in real-time</p>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="text-center mt-16 text-gray-400 text-sm">
          <p>Built with React, TypeScript, and SVG displacement mapping</p>
        </div>
      </div>
    </div>
  );
}
