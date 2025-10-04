"use client";

import React, { useState, useRef, useCallback, CSSProperties } from "react";

interface Ripple {
  key: number;
  x: number;
  y: number;
  size: number;
  state: "animating" | "paused" | "destroying";
}

const InteractiveRippleButton = () => {
  const [ripples, setRipples] = useState<Ripple[]>([]);
  const [bounceClass, setBounceClass] = useState("");
  const [isScaledDown, setIsScaledDown] = useState(false);
  const [isHolding, setIsHolding] = useState(false);

  const elementRef = useRef<HTMLDivElement>(null);
  const pressTimer = useRef<NodeJS.Timeout | null>(null);
  const isLongPress = useRef(false);
  const currentRippleKey = useRef<number | null>(null);

  const componentStyles = `
    /* Liquid, glassy ripple with subtle overshoot */
    .ripple {
      position: absolute;
      border-radius: 50%;
      pointer-events: none;
      width: 0;
      height: 0;
      transform: translateZ(0) scale(0);
      will-change: transform, opacity, filter;
      opacity: 0.95;
      transition: opacity 260ms ease-out;

      /* "Liquid glass" look */
      background:
        radial-gradient(65% 65% at 50% 50%,
          rgba(255,255,255,0.55) 0%,
          rgba(255,255,255,0.35) 35%,
          rgba(255,255,255,0.16) 55%,
          rgba(255,255,255,0) 70%
        ),
        radial-gradient(90% 90% at 30% 30%,
          rgba(255,255,255,0.85),
          rgba(255,255,255,0) 38%
        );
      backdrop-filter: blur(14px) saturate(160%);
      -webkit-backdrop-filter: blur(14px) saturate(160%);
      border: 1px solid rgba(255,255,255,0.28);
      box-shadow:
        0 12px 40px rgba(0,0,0,0.18),
        inset 0 1px 8px rgba(255,255,255,0.22);

      animation: ripple-spring 900ms cubic-bezier(.2,.8,.2,1) forwards;
    }

    /* glossy ring edge */
    .ripple::after {
      content: "";
      position: absolute;
      inset: 0;
      border-radius: inherit;
      -webkit-mask: radial-gradient(circle at 50% 50%, rgba(0,0,0,0) 58%, rgba(0,0,0,1) 63%);
      mask: radial-gradient(circle at 50% 50%, rgba(0,0,0,0) 58%, rgba(0,0,0,1) 63%);
      box-shadow: inset 0 0 0 2px rgba(255,255,255,0.65);
      filter: blur(0.3px);
      animation: ring-spring 900ms cubic-bezier(.2,.8,.2,1) forwards;
      opacity: 0.9;
      pointer-events: none;
    }

    /* pause both main and ring during long-press */
    .ripple.is-paused,
    .ripple.is-paused::after {
      animation-play-state: paused;
    }

    /* spring-like expansion with a couple of gentle rebounds */
    @keyframes ripple-spring {
      0%   { transform: scale(0);     opacity: 0.95; filter: saturate(150%) brightness(1.05); }
      48%  { transform: scale(2.75);  opacity: 0.55; }
      70%  { transform: scale(2.35);  opacity: 0.32; }
      85%  { transform: scale(2.55);  opacity: 0.18; }
      100% { transform: scale(2.45);  opacity: 0; }
    }

    @keyframes ring-spring {
      0%   { transform: scale(0.1);   opacity: 0.95; }
      55%  { transform: scale(2.75);  opacity: 0.45; }
      72%  { transform: scale(2.35);  opacity: 0.28; }
      85%  { transform: scale(2.55);  opacity: 0.15; }
      100% { transform: scale(2.45);  opacity: 0; }
    }

    /* smoother, elastic bounce on quick tap */
    .bounce-effect {
      animation: press-bounce 720ms cubic-bezier(.2,.8,.2,1);
    }
    @keyframes press-bounce {
      0%  { transform: scale(1); }
      22% { transform: scale(0.94); }
      55% { transform: scale(1.035); }
      75% { transform: scale(0.985); }
      100%{ transform: scale(1); }
    }

    /* deeper squash + rebound when releasing a long press */
    .long-press-bounce-effect {
      animation: long-press-release 780ms cubic-bezier(.2,.8,.2,1);
    }
    @keyframes long-press-release {
      0%   { transform: scale(0.9); }
      40%  { transform: scale(1.06); }
      70%  { transform: scale(0.985); }
      100% { transform: scale(1); }
    }

    /* gentle breathing while holding */
    .holding {
      animation: hold-pulse 1200ms ease-in-out infinite;
    }
    @keyframes hold-pulse {
      0%   { transform: scale(0.92); }
      50%  { transform: scale(0.95); }
      100% { transform: scale(0.92); }
    }

    /* Merge overlapping ripples with a gooey blend */
    .ripple-layer {
      position: absolute;
      inset: 0;
      overflow: hidden;
      pointer-events: none;
      filter: url(#goo);
    }

    @media (prefers-reduced-motion: reduce) {
      .ripple,
      .ripple::after,
      .bounce-effect,
      .long-press-bounce-effect,
      .holding {
        animation-duration: 0ms !important;
        transition: none !important;
      }
    }
  `;

  const cleanupRipple = useCallback((key: number) => {
    setRipples((prev) => prev.filter((ripple) => ripple.key !== key));
  }, []);

  const handlePressStart = useCallback(
    (e: React.MouseEvent<HTMLDivElement> | React.TouchEvent<HTMLDivElement>) => {
      e.preventDefault();
      const element = elementRef.current;
      if (!element) return;

      isLongPress.current = false;
      if (pressTimer.current) clearTimeout(pressTimer.current);

      const rect = element.getBoundingClientRect();
      const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
      const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;

      // Size big enough to cover container and allow elastic overshoot
      const size = Math.max(rect.width, rect.height) * 2;
      const x = clientX - rect.left - size / 2;
      const y = clientY - rect.top - size / 2;

      const newRipple: Ripple = {
        key: Date.now(),
        x,
        y,
        size,
        state: "animating",
      };

      currentRippleKey.current = newRipple.key;
      setRipples((prev) => [...prev, newRipple]);

      // Long press detection
      pressTimer.current = setTimeout(() => {
        isLongPress.current = true;
        setIsScaledDown(true);
        setIsHolding(true);
        setRipples((prev) =>
          prev.map((r) =>
            r.key === currentRippleKey.current ? { ...r, state: "paused" } : r
          )
        );
      }, 250);
    },
    []
  );

  const endCommon = () => {
    if (pressTimer.current) clearTimeout(pressTimer.current);
    setIsScaledDown(false);
    setIsHolding(false);
    isLongPress.current = false;
    currentRippleKey.current = null;
  };

  const handlePressEnd = useCallback(() => {
    if (isLongPress.current) {
      setBounceClass("long-press-bounce-effect");
      setRipples((prev) =>
        prev.map((r) =>
          r.key === currentRippleKey.current ? { ...r, state: "destroying" } : r
        )
      );
    } else {
      setBounceClass("bounce-effect");
    }
    endCommon();
  }, []);

  const handlePressLeave = useCallback(() => {
    // If the pointer leaves, gracefully fade out any current ripple
    if (currentRippleKey.current !== null) {
      setRipples((prev) =>
        prev.map((r) =>
          r.key === currentRippleKey.current ? { ...r, state: "destroying" } : r
        )
      );
    }
    endCommon();
  }, []);

  const handleBounceAnimationEnd = () => {
    setBounceClass("");
  };

  const getRippleStyle = (ripple: Ripple): CSSProperties => {
    const style: CSSProperties = {
      width: `${ripple.size}px`,
      height: `${ripple.size}px`,
      left: `${ripple.x}px`,
      top: `${ripple.y}px`,
    };

    if (ripple.state === "destroying") {
      style.opacity = 0; // triggers the smooth fade transition
    }

    return style;
  };

  const interactiveElementStyle: CSSProperties = {
    // Inline transform gives quick feedback on tap; during long-press
    // the .holding animation takes over for a subtle breathing effect.
    transform: isScaledDown ? "scale(0.92)" : "scale(1)",
    WebkitTapHighlightColor: "transparent",
  };

  return (
    <>
      {/* Goo filter for "liquid" merging when ripples overlap */}
      <svg width="0" height="0" style={{ position: "absolute" }} aria-hidden="true">
        <defs>
          <filter id="goo">
            <feGaussianBlur in="SourceGraphic" stdDeviation="8" result="blur" />
            <feColorMatrix
              in="blur"
              mode="matrix"
              values="
                1 0 0 0 0
                0 1 0 0 0
                0 0 1 0 0
                0 0 0 24 -8"
              result="goo"
            />
            <feBlend in="SourceGraphic" in2="goo" />
          </filter>
        </defs>
      </svg>

      <style>{componentStyles}</style>

      <div
        ref={elementRef}
        className={`relative w-80 h-56 md:w-96 md:h-64 bg-rose-500 flex items-center justify-center text-white font-sans text-lg select-none overflow-hidden cursor-pointer shadow-2xl shadow-rose-500/40 transition-transform duration-300 ease-in-out isolate transform-gpu ${bounceClass} ${isHolding ? "holding" : ""}`}
        style={interactiveElementStyle}
        onMouseDown={handlePressStart}
        onMouseUp={handlePressEnd}
        onMouseLeave={handlePressLeave}
        onTouchStart={handlePressStart}
        onTouchEnd={handlePressEnd}
        onTouchCancel={handlePressLeave}
        onAnimationEnd={handleBounceAnimationEnd}
      >
        Click or Hold Me

        {/* Ripples live in a goo layer so they merge when overlapping */}
        <div className="ripple-layer">
          {ripples.map((ripple) => (
            <div
              key={ripple.key}
              className={`ripple ${ripple.state === "paused" ? "is-paused" : ""}`}
              style={getRippleStyle(ripple)}
              onAnimationEnd={() => {
                if (ripple.state === "animating") {
                  cleanupRipple(ripple.key);
                }
              }}
              onTransitionEnd={() => {
                if (ripple.state === "destroying") {
                  cleanupRipple(ripple.key);
                }
              }}
            />
          ))}
        </div>
      </div>
    </>
  );
};

export default InteractiveRippleButton;