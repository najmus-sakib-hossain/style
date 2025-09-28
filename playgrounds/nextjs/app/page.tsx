"use client";

import React, { useCallback, useEffect, useRef, useState } from "react";
import LiquidGlass from "@/components/liquid-glass";
import Goo from "@/gooey-react/src/index";

/**
 * Simple spring-ish animator (very lightweight) for a 0..1 progress value.
 * We use it to drive a manual transform so the pill can smoothly
 * emerge and retract while sharing the same goo filter space as the circle.
 */
function useSpringProgress(target: number, stiffness = 280, damping = 22) {
  const [value, setValue] = useState(target);
  const velocity = useRef(0);

  useEffect(() => {
    let frame: number;
    const step = () => {
      const dt = 1 / 60;
      const x = value;
      const to = target;
      const k = stiffness;
      const c = damping;
      const Fspring = -k * (x - to);
      const Fdamper = -c * velocity.current;
      const a = Fspring + Fdamper;
      velocity.current += a * dt;
      let next = x + velocity.current * dt;

      // Clamp when close
      if (Math.abs(velocity.current) < 0.0001 && Math.abs(next - to) < 0.0001) {
        next = to;
        velocity.current = 0;
      } else {
        frame = requestAnimationFrame(step);
      }
      setValue(next);
    };
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [target, stiffness, damping, value]);

  return value;
}

export default function Home() {
  const [open, setOpen] = useState(false);
  const [pillMounted, setPillMounted] = useState(false);

  // Mount pill only after we start opening
  useEffect(() => {
    if (open) {
      setPillMounted(true);
    } else {
      // Delay unmount until animation retract completes
      const t = setTimeout(() => setPillMounted(false), 450);
      return () => clearTimeout(t);
    }
  }, [open]);

  // progress 0 (closed) -> 1 (open)
  const progress = useSpringProgress(open ? 1 : 0);

  // Derived transforms
  // Pill slides out to right & grows from a tiny speck at circle edge.
  const pillTranslateX = 0 + progress * 170; // px to the right
  const pillScale = 0.05 + progress * 1; // grows into full size
  const pillOpacity = Math.min(1, Math.max(0, progress * 1.2));

  // Circle can slightly scale to emphasize expansion
  const circleScale = 1 + progress * 0.08;

  const toggle = useCallback(() => {
    setOpen((o) => !o);
  }, []);

  return (
    <div className="min-h-screen flex items-center justify-center relative overflow-hidden bg-gradient-to-br from-slate-900 via-indigo-950 to-blue-950 text-white">
      {/* Soft grid background */}
      <div
        className="absolute inset-0 pointer-events-none opacity-25"
        style={{
          backgroundImage: `
            radial-gradient(circle at 1px 1px, rgba(255,255,255,0.07) 1px, transparent 0),
            linear-gradient(90deg, rgba(255,255,255,0.04) 1px, transparent 1px),
            linear-gradient(rgba(255,255,255,0.04) 1px, transparent 1px)
          `,
          backgroundSize: "140px 140px, 60px 60px, 60px 60px",
          mixBlendMode: "screen",
        }}
      />

      {/* Goo container – both shapes inside so they merge */}
      <Goo intensity="strong" className="relative">
        <div
          className="relative"
          style={{
            width: 600,
            height: 360,
          }}
        >
          {/* Circle (Bubble) */}
          <div
            style={{
              position: "absolute",
              top: "50%",
              left: "50%",
              transform: `translate(-50%, -50%) scale(${circleScale})`,
              transformOrigin: "center",
              transition: "transform 0.35s cubic-bezier(.25,.8,.25,1)",
              cursor: "pointer",
            }}
            onClick={toggle}
            aria-label="Toggle pill"
            role="button"
          >
            <LiquidGlass mode="bubble" colorTint="blue" className="select-none">
              <div className="flex flex-col items-center justify-center">
                <span className="text-2xl leading-none mb-1">👆</span>
                <span className="text-xs font-semibold tracking-wide uppercase">
                  {open ? "Merge" : "Click Me"}
                </span>
              </div>
            </LiquidGlass>
          </div>

          {/* Pill (conditionally mounted for better performance) */}
          {pillMounted && (
            <div
              style={{
                position: "absolute",
                top: "50%",
                left: "50%",
                // Start at circle edge and move outward
                transform: `translate(-50%, -50%) translateX(${pillTranslateX}px) scale(${pillScale})`,
                opacity: pillOpacity,
                transformOrigin: "center left",
                transition: "opacity 0.35s ease-out",
                pointerEvents: open ? "auto" : "none",
              }}
            >
              <LiquidGlass
                mode="pill"
                colorTint="purple"
                width={220}
                height={84}
                className="px-8 py-4 font-medium tracking-wide text-sm"
              >
                <div className="flex items-center gap-2">
                  <span>💠</span>
                  <span>Gooey Pill Action</span>
                </div>
              </LiquidGlass>
            </div>
          )}
        </div>
      </Goo>

      {/* Instructions */}
      <div className="absolute bottom-6 left-1/2 -translate-x-1/2 w-full max-w-lg px-4">
        <div className="backdrop-blur-md bg-white/5 border border-white/10 rounded-xl p-4 text-center text-sm leading-relaxed">
          <p className="font-semibold mb-1">Gooey Merge Demo</p>
          <p>
            Click the bubble. A pill grows & slides out while sharing the same
            goo filter. Click again and it retracts & fuses back into the bubble
            for a real liquid merge.
          </p>
        </div>
      </div>

      {/* Debug (optional – keep for clarity; comment out if not needed) */}
      <div className="absolute top-4 right-4 text-[11px] font-mono bg-black/40 rounded-md px-3 py-2 border border-white/10">
        <div>open: {open ? "true" : "false"}</div>
        <div>progress: {progress.toFixed(3)}</div>
      </div>
    </div>
  );
}
