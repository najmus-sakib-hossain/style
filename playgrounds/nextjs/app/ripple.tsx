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

  const elementRef = useRef<HTMLDivElement>(null);
  const pressTimer = useRef<NodeJS.Timeout | null>(null);
  const isLongPress = useRef(false);
  const currentRippleKey = useRef<number | null>(null);

  const componentStyles = `
    .ripple {
      position: absolute;
      border-radius: 50%;
      background: rgba(255, 255, 255, 0.4);
      backdrop-filter: blur(5px);
      -webkit-backdrop-filter: blur(5px);
      border: 1px solid rgba(255, 255, 255, 0.3);
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
      transform: scale(0);
      pointer-events: none;
      animation-name: ripple-in;
      animation-duration: 500ms;
      animation-timing-function: cubic-bezier(0.25, 0.8, 0.25, 1);
      animation-fill-mode: forwards;
      will-change: transform, opacity;
      opacity: 1;
      transition: opacity 250ms ease-out;
    }

    @keyframes ripple-in {
      0% {
        transform: scale(0);
        opacity: 0.7;
      }
      80% {
        transform: scale(2.5);
        opacity: 0.3;
      }
      100% {
        transform: scale(2.5);
        opacity: 0;
      }
    }

    .bounce-effect {
      animation: smooth-bounce-effect 800ms ease-out;
    }

    @keyframes smooth-bounce-effect {
      0% { transform: scale(1); }
      20% { transform: scale(0.95); }
      40% { transform: scale(1.02); }
      60% { transform: scale(0.98); }
      80% { transform: scale(1.01); }
      100% { transform: scale(1); }
    }

    .long-press-bounce-effect {
      animation: long-press-bounce-release 700ms ease-out;
    }

    @keyframes long-press-bounce-release {
      0% { transform: scale(0.9); }
      40% { transform: scale(1.04); }
      70% { transform: scale(0.98); }
      100% { transform: scale(1); }
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
      if (pressTimer.current) {
        clearTimeout(pressTimer.current);
      }

      const rect = element.getBoundingClientRect();
      const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
      const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;

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

      pressTimer.current = setTimeout(() => {
        isLongPress.current = true;
        setIsScaledDown(true);
        setRipples((prev) =>
          prev.map((r) =>
            r.key === currentRippleKey.current ? { ...r, state: "paused" } : r
          )
        );
      }, 250);
    },
    []
  );

  const handlePressEnd = useCallback(() => {
    if (pressTimer.current) {
      clearTimeout(pressTimer.current);
    }

    if (isLongPress.current) {
      setBounceClass("long-press-bounce-effect");
      setRipples((prev) =>
        prev.map((r) =>
          r.key === currentRippleKey.current
            ? { ...r, state: "destroying" }
            : r
        )
      );
    } else {
      setBounceClass("bounce-effect");
    }

    setIsScaledDown(false);
    isLongPress.current = false;
    currentRippleKey.current = null;
  }, []);

  const handlePressLeave = useCallback(() => {
    if (pressTimer.current) {
      clearTimeout(pressTimer.current);
    }
    setIsScaledDown(false);

    if (currentRippleKey.current !== null) {
      setRipples((prev) =>
        prev.map((r) =>
          r.key === currentRippleKey.current
            ? { ...r, state: "destroying" }
            : r
        )
      );
    }

    isLongPress.current = false;
    currentRippleKey.current = null;
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

    if (ripple.state === "paused") {
      style.animationPlayState = "paused";
    }

    if (ripple.state === "destroying") {
      style.animation = "none";
      style.opacity = 0;
    }

    return style;
  };

  const interactiveElementStyle: CSSProperties = {
    transform: isScaledDown ? "scale(0.9)" : "scale(1)",
    WebkitTapHighlightColor: "transparent",
  };

  return (
    <>
      <style>{componentStyles}</style>
      <div
        ref={elementRef}
        className={`relative w-80 h-56 md:w-96 md:h-64 bg-rose-500 flex items-center justify-center text-white font-sans text-lg select-none overflow-hidden cursor-pointer shadow-2xl shadow-rose-500/40 transition-transform duration-300 ease-in-out isolate transform-gpu ${bounceClass}`}
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
        {ripples.map((ripple) => (
          <div
            key={ripple.key}
            className="ripple"
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
    </>
  );
};

export default InteractiveRippleButton;