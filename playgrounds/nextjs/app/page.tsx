"use client";

import gsap from "gsap";
import Image from "next/image";
import React, { useCallback, useEffect, useRef, useState } from "react";
import LiquidGlassControls from "./liquid-glass-controls";
import { base, presets, type Config } from "./liquid-glass-configs";

const HomePage = () => {
  const effectRef = useRef<HTMLButtonElement>(null);
  const dockPlaceholderRef = useRef<HTMLDivElement>(null);

  const [config, setConfig] = useState(presets.dock);
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
  const [isDragging, setIsDragging] = useState(false);
  const dragOffset = useRef({ x: 0, y: 0 });
  const [isExpanded, setIsExpanded] = useState(false);
  const [isMoving, setIsMoving] = useState(false); // keep visible during motion

  // Asymmetric easing (open: slow start, close: slow end)
  const OPEN_EASE = "cubic-bezier(0.55, 0.085, 0.68, 0.53)";
  const CLOSE_EASE = "cubic-bezier(0.25, 0.46, 0.45, 0.94)";
  const ease = isExpanded ? OPEN_EASE : CLOSE_EASE;

  // Faster movement
  const OPEN_MS = 1200;
  const CLOSE_MS = 900;
  const transformMs = isExpanded ? OPEN_MS : CLOSE_MS;

  const y = isExpanded ? -160 : -100;
  const move = `translateX(-50%) translateY(${y}px)`;

  // Skeleton panel: full, fixed height; stays visible while moving so close is seen
  const skeletonStyle: React.CSSProperties = {
    transform: move,
    opacity: isExpanded || isMoving ? 1 : 0,
    transformOrigin: "bottom center",
    willChange: "transform, opacity",
    transition: [`transform ${transformMs}ms ${ease}`, "opacity 150ms ease-out"].join(", "),
  };

  // Text container: always visible during motion; clicks only when expanded
  const contentStyle: React.CSSProperties = {
    transform: move,
    opacity: isExpanded || isMoving ? 1 : 0,
    pointerEvents: isExpanded ? "auto" : "none",
    transformOrigin: "bottom center",
    willChange: "transform, opacity",
    transition: `transform ${transformMs}ms ${ease}, opacity 150ms ease`,
  };

  // When movement finishes, allow panel to hide if we're closing
  const onMoveEnd = (e: React.TransitionEvent<HTMLDivElement>) => {
    if (e.propertyName === "transform") {
      setIsMoving(false);
    }
  };

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

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      if (!effectRef.current) return;
      setIsDragging(true);
      const rect = effectRef.current.getBoundingClientRect();
      dragOffset.current = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };
      e.preventDefault();
    },
    [],
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  const handleMouseMoveGlobal = useCallback(
    (e: MouseEvent) => {
      if (!isDragging) return;
      setPosition({
        x: e.clientX - dragOffset.current.x,
        y: e.clientY - dragOffset.current.y,
      });
    },
    [isDragging],
  );

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
    if (isDragging) {
      window.addEventListener("mousemove", handleMouseMoveGlobal);
      window.addEventListener("mouseup", handleMouseUp);
    }

    return () => {
      window.removeEventListener("mousemove", handleMouseMoveGlobal);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDragging, handleMouseMoveGlobal, handleMouseUp]);

  useEffect(() => {
    if (dockPlaceholderRef.current) {
      const rect = dockPlaceholderRef.current.getBoundingClientRect();
      setPosition({
        x: rect.left,
        y: rect.top > window.innerHeight ? window.innerHeight * 0.5 : rect.top,
      });
      gsap.set(".effect, .effect-border", { opacity: 1 });
    }
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;

    const buildDisplacementImage = () => {
      const border =
        Math.min(config.width, config.height) * (config.border * 0.5);
      const svgString = `
        <svg class="displacement-image" viewBox="0 0 ${config.width} ${config.height
        }" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="red" x1="100%" y1="0%" x2="0%" y2="0%">
              <stop offset="0%" stop-color="#000"/>
              <stop offset="100%" stop-color="red"/>
            </linearGradient>
            <linearGradient id="blue" x1="0%" y1="0%" x2="0%" y2="100%">
              <stop offset="0%" stop-color="#000"/>
              <stop offset="100%" stop-color="blue"/>
            </linearGradient>
          </defs>
          <rect x="0" y="0" width="${config.width}" height="${config.height
        }" fill="black"></rect>
          <rect x="0" y="0" width="${config.width}" height="${config.height
        }" rx="${config.radius}" fill="url(#red)" />
          <rect x="0" y="0" width="${config.width}" height="${config.height
        }" rx="${config.radius
        }" fill="url(#blue)" style="mix-blend-mode: ${config.blend}" />
          <rect x="${border}" y="${border}" width="${config.width - border * 2
        }" height="${config.height - border * 2}" rx="${config.radius
        }" fill="hsl(0 0% ${config.lightness}% / ${config.alpha
        })" style="filter:blur(${config.blur}px)" />
        </svg>
      `;
      const encoded = encodeURIComponent(svgString);
      const dataUri = `data:image/svg+xml,${encoded}`;
      gsap.set("feImage", { attr: { href: dataUri } });
      gsap.set("feDisplacementMap", {
        attr: { xChannelSelector: config.x, yChannelSelector: config.y },
      });
    };

    const update = () => {
      buildDisplacementImage();
      gsap.set(document.documentElement, {
        "--width": config.width,
        "--height": config.height,
        "--radius": config.radius,
        "--frost": config.frost,
        "--output-blur": config.displace,
        "--saturation": config.saturation,
      });
      gsap.set("feDisplacementMap", { attr: { scale: config.scale } });
      gsap.set("#redchannel", { attr: { scale: config.scale + config.r } });
      gsap.set("#greenchannel", {
        attr: { scale: config.scale + config.g },
      });
      gsap.set("#bluechannel", { attr: { scale: config.scale + config.b } });
      gsap.set("feGaussianBlur", {
        attr: { stdDeviation: config.displace },
      });

      document.documentElement.dataset.icons = String(config.icons);
    };

    update();
  }, [config]);

  const elasticTranslation = isDragging
    ? { x: 0, y: 0 }
    : calculateElasticTranslation();
  const directionalScale = isDragging ? "scale(1)" : calculateDirectionalScale();

  const transformStyle = `translate(${elasticTranslation.x}px, ${elasticTranslation.y
    }px) ${isActive ? "scale(0.96)" : directionalScale}`;

  // Thicker ring + juicy water-drop shadow + fluid splash highlights
  const borderThickness = 2.5;

  const dropShadow = [
    `0 ${Math.max(10, Math.round(config.height * 0.18))}px ${Math.max(28, Math.round(config.height * 0.55))}px ${-Math.max(6, Math.round(config.height * 0.18))}px rgba(0,0,0,0.45)`,
    `0 6px 18px rgba(0,0,0,0.25)`,
    `inset 0 10px 22px rgba(255,255,255,0.28)`,
    `inset 0 -12px 28px rgba(0,0,0,0.22)`,
  ].join(", ");

  const fluidSplashBackground1 = `
    conic-gradient(
      from ${110 + mouseOffset.x * 0.5}deg at ${50 + mouseOffset.x * 0.1}% ${50 + mouseOffset.y * 0.1}%,
      rgba(255,255,255,0.0) 0deg,
      rgba(255,255,255, ${0.22 + Math.abs(mouseOffset.x) * 0.008}) 35deg,
      rgba(255,255,255, ${0.70 + Math.abs(mouseOffset.y) * 0.010}) 65deg,
      rgba(255,255,255,0.0) 120deg,
      rgba(255,255,255, ${0.30 + Math.abs(mouseOffset.y) * 0.006}) 170deg,
      rgba(255,255,255,0.0) 360deg
    ),
    radial-gradient(120% 70% at ${28 + mouseOffset.x * 0.14}% ${18 + mouseOffset.y * 0.10}%,
      rgba(255,255,255,0.65) 0%,
      rgba(255,255,255,0.22) 35%,
      rgba(255,255,255,0.0) 60%)
  `;

  const fluidSplashBackground2 = `
    radial-gradient(140% 100% at ${35 + mouseOffset.x * 0.10}% ${22 + mouseOffset.y * 0.10}%,
      rgba(255,255,255,0.35) 0%,
      rgba(255,255,255,0.12) 45%,
      rgba(255,255,255,0.0) 65%),
    conic-gradient(
      from ${200 + mouseOffset.x * 0.3}deg at 50% 50%,
      rgba(255,255,255,0.0) 0deg,
      rgba(255,255,255, ${0.35 + Math.abs(mouseOffset.x) * 0.006}) 55deg,
      rgba(255,255,255, ${0.72 + Math.abs(mouseOffset.y) * 0.008}) 75deg,
      rgba(255,255,255,0.0) 160deg,
      rgba(255,255,255, ${0.25 + Math.abs(mouseOffset.x) * 0.006}) 210deg,
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
    transition: isDragging ? "none" : "transform ease-out 0.2s",
    width: `${config.width}px`,
    height: `${config.height}px`,
    position: "fixed",
    top: `${position.y}px`,
    left: `${position.x}px`,
    cursor: isDragging ? "grabbing" : "grab",
    borderRadius: `${config.radius}px`,
    boxShadow: dropShadow,
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

      <button
        type="button"
        className="effect"
        ref={effectRef}
        style={baseStyle}
        onMouseDown={handleMouseDown}
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

        {/* Internal specular splash for liquid feel */}
        <div
          aria-hidden
          style={{
            position: "absolute",
            inset: 0,
            borderRadius: `${config.radius}px`,
            pointerEvents: "none",
            mixBlendMode: "screen",
            background: `
              radial-gradient(80% 50% at calc(22% + ${mouseOffset.x * 0.15}%) calc(18% + ${mouseOffset.y * 0.08}%),
                rgba(255,255,255,0.75) 0%,
                rgba(255,255,255,0.35) 25%,
                rgba(255,255,255,0.10) 48%,
                rgba(255,255,255,0.0) 60%),
              radial-gradient(40% 30% at calc(58% + ${mouseOffset.x * 0.06}%) calc(20% + ${mouseOffset.y * 0.05}%),
                rgba(255,255,255,0.45) 0%,
                rgba(255,255,255,0.12) 45%,
                rgba(255,255,255,0.0) 65%)
            `,
            filter: "blur(0.4px) saturate(1.15)",
            opacity: 0.9,
          }}
        />

        <svg className="filter" xmlns="http://www.w3.org/2000/svg">
          <title>Glass effect SVG filter</title>
          <defs>
            <filter id="filter" colorInterpolationFilters="sRGB">
              <feImage x="0" y="0" width="100%" height="100%" result="map" />
              <feDisplacementMap
                in="SourceGraphic"
                in2="map"
                id="redchannel"
                xChannelSelector="R"
                yChannelSelector="G"
                result="dispRed"
              />
              <feColorMatrix
                in="dispRed"
                type="matrix"
                values="1 0 0 0 0
                                0 0 0 0 0
                                0 0 0 0 0
                                0 0 0 1 0"
                result="red"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="map"
                id="greenchannel"
                xChannelSelector="R"
                yChannelSelector="G"
                result="dispGreen"
              />
              <feColorMatrix
                in="dispGreen"
                type="matrix"
                values="0 0 0 0 0
                                0 1 0 0 0
                                0 0 0 0 0
                                0 0 0 1 0"
                result="green"
              />
              <feDisplacementMap
                in="SourceGraphic"
                in2="map"
                id="bluechannel"
                xChannelSelector="R"
                yChannelSelector="G"
                result="dispBlue"
              />
              <feColorMatrix
                in="dispBlue"
                type="matrix"
                values="0 0 0 0 0
                                0 0 0 0 0
                                0 0 1 0 0
                                0 0 0 1 0"
                result="blue"
              />
              <feBlend in="red" in2="green" mode="screen" result="rg" />
              <feBlend in="rg" in2="blue" mode="screen" result="output" />
              <feGaussianBlur in="output" stdDeviation="0.7" />
            </filter>
          </defs>
        </svg>
      </button>

      {/* Thick, fluid splash ring around the glass (not a straight line) */}
      <span
        className="effect-border"
        style={{
          ...borderStyle,
          mixBlendMode: "screen",
          opacity: 0.42,
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
          opacity: 0.85,
          padding: `${borderThickness}px`,
          WebkitMask:
            "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
          WebkitMaskComposite: "xor",
          maskComposite: "exclude",
          background: fluidSplashBackground2,
        }}
      />

      <main>
        <section className="placeholder">
          <div className="dock-placeholder" ref={dockPlaceholderRef} />
        </section>
      </main>

    </>
  );
};

export default HomePage;
