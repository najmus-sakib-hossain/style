"use client";
import gsap from "gsap";
import Image from "next/image";
import React, { useCallback, useEffect, useRef, useState } from "react";
import LiquidGlassControls from "./liquid-glass-controls";
import { presets, type Config } from "./liquid-glass-configs";
import { GlassFilter, updateDisplacementFilter } from "./liquid-glass-filter";
import Gooey from "./gooey";

const HomePage = () => {
  const effectRef = useRef<HTMLButtonElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);

  const [config, setConfig] = useState(presets.bubble);
  const [isActive] = useState(false);
  const [globalMousePos, setGlobalMousePos] = useState({ x: 0, y: 0 });
  const [mouseOffset, setMouseOffset] = useState({ x: 0, y: 0 });
  const [isChromaticEnabled, setIsChromaticEnabled] = useState(true);
  const [chromaticCache, setChromaticCache] = useState({
    r: presets.dock.r,
    g: presets.dock.g,
    b: presets.dock.b,
  });

  const [position, setPosition] = useState({ x: 0, y: 0 });

  const [isHeld, setIsHeld] = useState(false);
  const [pressOffset, setPressOffset] = useState({ x: 0, y: 0 });
  const holdXTween = useRef<any>(null);
  const holdYTween = useRef<any>(null);

  const handleConfigChange = (
    key: keyof Config,
    value: string | number | boolean,
  ) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handlePresetChange = (presetName: string) => {
    const newConfig = presets[presetName as keyof typeof presets];
    setChromaticCache({ r: newConfig.r, g: newConfig.g, b: newConfig.b });
    const targetConfig = { ...newConfig };
    if (!isChromaticEnabled) {
      targetConfig.r = 0;
      targetConfig.g = 0;
      targetConfig.b = 0;
    }
    gsap.to(config, {
      ...targetConfig,
      duration: 0.5,
      onUpdate: () => {
        setConfig({ ...config });
      },
    });
  };

  const toggleChromaticAberration = () => {
    setIsChromaticEnabled((prev) => {
      const isEnabling = !prev;
      if (isEnabling) {
        setConfig((c) => ({
          ...c,
          r: chromaticCache.r,
          g: chromaticCache.g,
          b: chromaticCache.b,
        }));
      } else {
        setChromaticCache({ r: config.r, g: config.g, b: config.b });
        setConfig((c) => ({
          ...c,
          r: 0,
          g: 0,
          b: 0,
        }));
      }
      return isEnabling;
    });
  };

  const calculateFadeInFactor = useCallback(() => {
    if (!globalMousePos.x || !globalMousePos.y || !effectRef.current) {
      return 0;
    }
    const rect = effectRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const edgeDistanceX = Math.max(
      0,
      Math.abs(globalMousePos.x - centerX) - config.width / 2,
    );
    const edgeDistanceY = Math.max(
      0,
      Math.abs(globalMousePos.y - centerY) - config.height / 2,
    );
    const edgeDistance = Math.sqrt(
      edgeDistanceX * edgeDistanceX + edgeDistanceY * edgeDistanceY,
    );
    const activationZone = 200;
    return edgeDistance > activationZone
      ? 0
      : 1 - edgeDistance / activationZone;
  }, [globalMousePos, config.width, config.height]);

  const calculateElasticTranslation = useCallback(() => {
    if (!effectRef.current) return { x: 0, y: 0 };
    const fadeInFactor = calculateFadeInFactor();
    const rect = effectRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    return {
      x: (globalMousePos.x - centerX) * config.elasticity * 0.1 * fadeInFactor,
      y: (globalMousePos.y - centerY) * config.elasticity * 0.1 * fadeInFactor,
    };
  }, [globalMousePos, config.elasticity, calculateFadeInFactor]);

  const calculateDirectionalScale = useCallback(() => {
    if (!globalMousePos.x || !globalMousePos.y || !effectRef.current) {
      return "scale(1)";
    }
    const rect = effectRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    const deltaX = globalMousePos.x - centerX;
    const deltaY = globalMousePos.y - centerY;

    const edgeDistanceX = Math.max(0, Math.abs(deltaX) - config.width / 2);
    const edgeDistanceY = Math.max(0, Math.abs(deltaY) - config.height / 2);
    const edgeDistance = Math.sqrt(
      edgeDistanceX * edgeDistanceX + edgeDistanceY * edgeDistanceY,
    );
    const activationZone = 200;
    if (edgeDistance > activationZone) return "scale(1)";

    const fadeInFactor = 1 - edgeDistance / activationZone;
    const centerDistance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
    if (centerDistance === 0) return "scale(1)";

    const normalizedX = deltaX / centerDistance;
    const normalizedY = deltaY / centerDistance;
    const stretchIntensity =
      Math.min(centerDistance / 300, 1) * config.elasticity * fadeInFactor;
    const scaleX =
      1 +
      Math.abs(normalizedX) * stretchIntensity * 0.3 -
      Math.abs(normalizedY) * stretchIntensity * 0.15;
    const scaleY =
      1 +
      Math.abs(normalizedY) * stretchIntensity * 0.3 -
      Math.abs(normalizedX) * stretchIntensity * 0.15;

    return `scaleX(${Math.max(0.8, scaleX)}) scaleY(${Math.max(0.8, scaleY)})`;
  }, [globalMousePos, config.elasticity, config.width, config.height]);

  useEffect(() => {
    if (!wrapperRef.current) return;
    gsap.set(wrapperRef.current, {
      "--hold-x": "0px",
      "--hold-y": "0px",
      "--press-scale": 1,
      "--jello-x": 1,
      "--jello-y": 1,
    } as any);
  }, []);

  useEffect(() => {
    const handleLocalMouseMove = (e: MouseEvent) => {
      setGlobalMousePos({ x: e.clientX, y: e.clientY });
      if (effectRef.current) {
        const rect = effectRef.current.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        setMouseOffset({
          x: ((e.clientX - centerX) / rect.width) * 100,
          y: ((e.clientY - centerY) / rect.height) * 100,
        });
      }
    };
    window.addEventListener("mousemove", handleLocalMouseMove);
    return () => window.removeEventListener("mousemove", handleLocalMouseMove);
  }, []);

  useEffect(() => {
    setPosition({
      x: Math.round((window.innerWidth - config.width) / 2),
      y: Math.round((window.innerHeight - config.height) / 2),
    });
    gsap.set(".effect, .effect-border", { opacity: 1 });
    const onResize = () => {
      setPosition({
        x: Math.round((window.innerWidth - config.width) / 2),
        y: Math.round((window.innerHeight - config.height) / 2),
      });
    };
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [config.width, config.height]);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const update = () => {
      updateDisplacementFilter(config);
      gsap.set(document.documentElement, {
        "--width": config.width,
        "--height": config.height,
        "--radius": config.radius,
        "--frost": config.frost,
        "--output-blur": config.displace,
        "--saturation": config.saturation,
      } as any);

      (document.documentElement as any).dataset.icons = String(config.icons);
    };

    update();
  }, [config]);

  const handlePressStart = useCallback(
    (e: React.PointerEvent<HTMLButtonElement>) => {
      if (!effectRef.current || !wrapperRef.current) return;
      setIsHeld(true);
      try {
        e.currentTarget.setPointerCapture(e.pointerId);
      } catch { }
      holdXTween.current = gsap.quickTo(wrapperRef.current, "--hold-x", {
        duration: 0.25,
        ease: "expo.out",
      } as any);
      holdYTween.current = gsap.quickTo(wrapperRef.current, "--hold-y", {
        duration: 0.25,
        ease: "expo.out",
      } as any);

      const rect = effectRef.current.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;
      setPressOffset({
        x: ((e.clientX - centerX) / rect.width) * 100,
        y: ((e.clientY - centerY) / rect.height) * 100,
      });

      // MODIFIED: Scale up on press for a juicier feel
      gsap.to(wrapperRef.current, {
        "--press-scale": 1.05,
        duration: 0.12,
        ease: "power2.out",
      } as any);
      gsap.to(document.documentElement, {
        "--output-blur": Math.max(config.displace, 0) + 8,
        duration: 0.18,
        ease: "power2.out",
      } as any);
    },
    [config.displace],
  );

  const handlePressMove = useCallback((e: PointerEvent) => {
    if (!effectRef.current || !isHeld) return;
    const rect = effectRef.current.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;
    const dx = e.clientX - centerX;
    const dy = e.clientY - centerY;

    const max = Math.max(18, Math.min(rect.width, rect.height) * 0.08);
    const dist = Math.hypot(dx, dy);
    const tx = dist > 0 ? (dx / dist) * Math.min(dist, max) : 0;
    const ty = dist > 0 ? (dy / dist) * Math.min(dist, max) : 0;

    holdXTween.current?.(tx);
    holdYTween.current?.(ty);

    setPressOffset({
      x: ((e.clientX - centerX) / rect.width) * 100,
      y: ((e.clientY - centerY) / rect.height) * 100,
    });
  }, [isHeld]);

  useEffect(() => {
    const onMove = (e: PointerEvent) => handlePressMove(e);
    const onUp = () => {
      if (!isHeld || !wrapperRef.current) return;
      setIsHeld(false);
      holdXTween.current?.(0);
      holdYTween.current?.(0);

      const tl = gsap.timeline();
      tl.to(wrapperRef.current, {
        "--jello-x": 1.06,
        "--jello-y": 0.94,
        duration: 0.1,
        ease: "power2.out",
      } as any)
        .to(wrapperRef.current, {
          "--jello-x": 0.96,
          "--jello-y": 1.04,
          duration: 0.12,
          ease: "sine.out",
        } as any)
        .to(wrapperRef.current, {
          "--jello-x": 1.02,
          "--jello-y": 0.98,
          duration: 0.14,
          ease: "sine.out",
        } as any)
        .to(wrapperRef.current, {
          "--jello-x": 1,
          "--jello-y": 1,
          duration: 0.2,
          ease: "sine.out",
        } as any);

      gsap.to(wrapperRef.current, {
        "--press-scale": 1,
        duration: 0.5,
        ease: "elastic.out(1, 0.45)",
      } as any);
      gsap.to(document.documentElement, {
        "--output-blur": config.displace,
        duration: 0.3,
        ease: "power2.out",
      } as any);
    };

    if (isHeld) {
      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
      window.addEventListener("pointercancel", onUp);
    }
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      window.removeEventListener("pointercancel", onUp);
    };
  }, [isHeld, handlePressMove, config.displace]);

  const elasticTranslation = isHeld ? { x: 0, y: 0 } : calculateElasticTranslation();
  const directionalScale = isHeld ? "scale(1)" : calculateDirectionalScale();

  const translateXStr = `calc(${elasticTranslation.x}px + var(--hold-x, 0px))`;
  const translateYStr = `calc(${elasticTranslation.y}px + var(--hold-y, 0px))`;

  const transformStyle = `translate(${translateXStr}, ${translateYStr}) ${isActive ? "scale(0.96)" : directionalScale} scale(var(--press-scale, 1)) scaleX(var(--jello-x, 1)) scaleY(var(--jello-y, 1))`;

  const borderThickness = 2.5;

  const dropShadow = [
    `0 ${Math.max(10, Math.round(config.height * 0.18))}px ${Math.max(28, Math.round(config.height * 0.55))}px ${-Math.max(6, Math.round(config.height * 0.18))}px rgba(0,0,0,0.45)`,
    `0 6px 18px rgba(0,0,0,0.25)`,
    `inset 0 10px 22px rgba(255,255,255,0.28)`,
    `inset 0 -12px 28px rgba(0,0,0,0.22)`,
  ].join(", ");

  const activeOffset = isHeld ? pressOffset : mouseOffset;

  const fluidSplashBackground1 = `
    conic-gradient(
      from ${110 + activeOffset.x * 0.5}deg at ${50 + activeOffset.x * 0.1}% ${50 + activeOffset.y * 0.1}%,
      rgba(255,255,255,0.0) 0deg,
      rgba(255,255,255, ${0.22 + Math.abs(activeOffset.x) * 0.008}) 35deg,
      rgba(255,255,255, ${0.70 + Math.abs(activeOffset.y) * 0.010}) 65deg,
      rgba(255,255,255,0.0) 120deg,
      rgba(255,255,255, ${0.30 + Math.abs(activeOffset.y) * 0.006}) 170deg,
      rgba(255,255,255,0.0) 360deg
    ),
    radial-gradient(120% 70% at ${28 + activeOffset.x * 0.14}% ${18 + activeOffset.y * 0.10}%,
      rgba(255,255,255,0.65) 0%,
      rgba(255,255,255,0.22) 35%,
      rgba(255,255,255,0.0) 60%)
  `;

  const fluidSplashBackground2 = `
    radial-gradient(140% 100% at ${35 + activeOffset.x * 0.10}% ${22 + activeOffset.y * 0.10}%,
      rgba(255,255,255,0.35) 0%,
      rgba(255,255,255,0.12) 45%,
      rgba(255,255,255,0.0) 65%),
    conic-gradient(
      from ${200 + activeOffset.x * 0.3}deg at 50% 50%,
      rgba(255,255,255,0.0) 0deg,
      rgba(255,255,255, ${0.35 + Math.abs(activeOffset.x) * 0.006}) 55deg,
      rgba(255,255,255, ${0.72 + Math.abs(activeOffset.y) * 0.008}) 75deg,
      rgba(255,255,255,0.0) 160deg,
      rgba(255,255,255, ${0.25 + Math.abs(activeOffset.x) * 0.006}) 210deg,
      rgba(255,255,255,0.0) 360deg
    )
  `;

  const baseStyle: React.CSSProperties = {
    background: "transparent",
    border: "none",
    padding: 0,
    margin: 0,
    textAlign: "inherit",
    font: "inherit",
    transform: transformStyle,
    transition: isHeld ? "none" : "transform ease-out 0.2s",
    width: `${config.width}px`,
    height: `${config.height}px`,
    position: "fixed",
    top: `${position.y}px`,
    left: `${position.x}px`,
    cursor: "pointer",
    borderRadius: `${config.radius}px`,
    boxShadow: dropShadow,
    touchAction: "none",
  };

  const borderStyle: React.CSSProperties = {
    ...baseStyle,
    borderRadius: `${config.radius}px`,
    pointerEvents: "none",
  };

  return (
    <>
      <LiquidGlassControls
        config={config}
        handleConfigChange={handleConfigChange}
        handlePresetChange={handlePresetChange}
        toggleChromaticAberration={toggleChromaticAberration}
        isChromaticEnabled={isChromaticEnabled}
      />

      <div ref={wrapperRef}>
        <button
          type="button"
          className="effect"
          ref={effectRef}
          style={baseStyle}
          onPointerDown={handlePressStart}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") e.preventDefault();
          }}
        >
          <div className="nav-wrap">
            <nav>
              <Image
                src="https://assets.codepen.io/605876/finder.png"
                alt="macOS Finder icon"
                width={80}
                height={80}
              />
              <Image
                src="https://assets.codepen.io/605876/launch-control.png"
                alt="macOS Launch Control icon"
                width={80}
                height={80}
              />
              <Image
                src="https://assets.codepen.io/605876/safari.png"
                alt="macOS Safari icon"
                width={80}
                height={80}
              />
              <Image
                src="https://assets.codepen.io/605876/calendar.png"
                alt="macOS Calendar icon"
                width={80}
                height={80}
              />
            </nav>
          </div>

          <div
            aria-hidden
            style={{
              position: "absolute",
              inset: 0,
              borderRadius: `${config.radius}px`,
              pointerEvents: "none",
              mixBlendMode: "screen",
              background: `
                radial-gradient(80% 50% at calc(22% + ${activeOffset.x * 0.15}%) calc(18% + ${activeOffset.y * 0.08}%),
                  rgba(255,255,255,0.75) 0%,
                  rgba(255,255,255,0.35) 25%,
                  rgba(255,255,255,0.10) 48%,
                  rgba(255,255,255,0.0) 60%),
                radial-gradient(40% 30% at calc(58% + ${activeOffset.x * 0.06}%) calc(20% + ${activeOffset.y * 0.05}%),
                  rgba(255,255,255,0.45) 0%,
                  rgba(255,255,255,0.12) 45%,
                  rgba(255,255,255,0.0) 65%)
              `,
              filter: "blur(0.4px) saturate(1.15)",
              opacity: 0.9,
            }}
          />

          <div
            aria-hidden
            style={{
              position: "absolute",
              inset: 0,
              borderRadius: `${config.radius}px`,
              pointerEvents: "none",
              mixBlendMode: "screen",
              opacity: isHeld ? 1 : 0,
              transition:
                "opacity 180ms ease-out, backdrop-filter 250ms ease-out",
              background: `
                radial-gradient(180% 140% at calc(${50 + activeOffset.x * 0.5}%) calc(${50 + activeOffset.y * 0.5}%),
                  rgba(255,255,255,1.0) 0%,
                  rgba(255,255,255,0.65) 25%,
                  rgba(255,255,255,0.2) 50%,
                  rgba(255,255,255,0.0) 70%)
              `,
              filter: "saturate(1.12)",
              backdropFilter: isHeld ? "blur(12px)" : "blur(0px)",
            } as React.CSSProperties}
          />

          <GlassFilter />
        </button>

        <span
          className="effect-border"
          style={{
            ...borderStyle,
            mixBlendMode: "screen",
            opacity: isHeld ? 0.65 : 0.42,
            padding: `${borderThickness}px`,
            WebkitMask:
              "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
            WebkitMaskComposite: "xor",
            maskComposite: "exclude",
            background: fluidSplashBackground1,
          }}
        />
        <span
          className="effect-border"
          style={{
            ...borderStyle,
            mixBlendMode: "overlay",
            opacity: isHeld ? 1 : 0.85,
            padding: `${borderThickness}px`,
            WebkitMask:
              "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
            WebkitMaskComposite: "xor",
            maskComposite: "exclude",
            background: fluidSplashBackground2,
          }}
        />
      </div>
      <Gooey />

    </>
  );
};

export default HomePage;
