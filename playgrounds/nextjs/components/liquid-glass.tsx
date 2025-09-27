"use client";

import React, { useRef, useState, useEffect, useCallback, useId } from "react";

// Types
interface LiquidGlassProps {
  children: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
  width?: number;
  height?: number;
  radius?: number;
  mode?: "dock" | "pill" | "bubble" | "free" | "standard";
  displacementScale?: number;
  frost?: number;
  border?: number;
  alpha?: number;
  lightness?: number;
  blur?: number;
  saturation?: number;
  draggable?: boolean;
  colorTint?: "none" | "blue" | "purple" | "green" | "pink" | "orange";
  onClick?: () => void;
  onMouseEnter?: () => void;
  onMouseLeave?: () => void;
}

// Preset configurations matching public script.js
const presets = {
  dock: {
    width: 336,
    height: 96,
    radius: 16,
    scale: -180,
    frost: 0.05,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
    blur: 11,
    saturation: 1.5,
    icons: true,
  },
  pill: {
    width: 200,
    height: 80,
    radius: 40,
    scale: -180,
    frost: 0,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
    blur: 11,
    saturation: 1,
    icons: false,
  },
  bubble: {
    width: 140,
    height: 140,
    radius: 70,
    scale: -180,
    frost: 0,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
    blur: 11,
    saturation: 1,
    icons: false,
  },
  free: {
    width: 140,
    height: 280,
    radius: 80,
    scale: -300,
    frost: 0,
    border: 0.15,
    alpha: 0.74,
    lightness: 60,
    blur: 10,
    saturation: 1,
    icons: false,
  },
  standard: {
    width: 270,
    height: 69,
    radius: 16,
    scale: -180,
    frost: 0.05,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
    blur: 11,
    saturation: 1.2,
    icons: false,
  },
};

// Color tint configurations
const colorTints = {
  none: { hue: 0, saturation: 0 },
  blue: { hue: 220, saturation: 0.3 },
  purple: { hue: 280, saturation: 0.25 },
  green: { hue: 140, saturation: 0.2 },
  pink: { hue: 320, saturation: 0.25 },
  orange: { hue: 30, saturation: 0.2 },
};

// Generate displacement image (matching public script.js)
const buildDisplacementImage = (config: any) => {
  const border = Math.min(config.width, config.height) * (config.border * 0.5);

  const svgString = `
    <svg viewBox="0 0 ${config.width} ${config.height}" xmlns="http://www.w3.org/2000/svg">
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
      <rect x="0" y="0" width="${config.width}" height="${config.height}" fill="black"></rect>
      <rect x="0" y="0" width="${config.width}" height="${config.height}" rx="${config.radius}" fill="url(#red)" />
      <rect x="0" y="0" width="${config.width}" height="${config.height}" rx="${config.radius}" fill="url(#blue)" style="mix-blend-mode: difference" />
      <rect x="${border}" y="${border}" width="${config.width - border * 2}" height="${config.height - border * 2}" rx="${config.radius}" fill="hsl(0 0% ${config.lightness}% / ${config.alpha})" style="filter:blur(${config.blur}px)" />
    </svg>
  `;

  return `data:image/svg+xml,${encodeURIComponent(svgString)}`;
};

// Device tilt detection
const useDeviceOrientation = () => {
  const [tilt, setTilt] = useState({ x: 0, y: 0 });

  useEffect(() => {
    const handleOrientation = (event: DeviceOrientationEvent) => {
      const { beta, gamma } = event;
      setTilt({
        x: (gamma || 0) / 90, // Normalize to -1 to 1
        y: (beta || 0) / 90,
      });
    };

    if (
      window.DeviceOrientationEvent &&
      typeof (DeviceOrientationEvent as any).requestPermission === "function"
    ) {
      (DeviceOrientationEvent as any)
        .requestPermission()
        .then((response: string) => {
          if (response === "granted") {
            window.addEventListener("deviceorientation", handleOrientation);
          }
        });
    } else if (window.DeviceOrientationEvent) {
      window.addEventListener("deviceorientation", handleOrientation);
    }

    return () => {
      window.removeEventListener("deviceorientation", handleOrientation);
    };
  }, []);

  return tilt;
};

// SVG Filter Component (matching public HTML structure)
const LiquidGlassFilter: React.FC<{
  id: string;
  config: any;
  displacementMapUrl: string;
}> = ({ id, config, displacementMapUrl }) => (
  <svg style={{ position: "absolute", width: 0, height: 0 }} aria-hidden="true">
    <defs>
      <filter
        id={id}
        x="-35%"
        y="-35%"
        width="170%"
        height="170%"
        colorInterpolationFilters="sRGB"
      >
        <feImage
          x="0"
          y="0"
          width="100%"
          height="100%"
          result="map"
          href={displacementMapUrl}
          preserveAspectRatio="xMidYMid slice"
        />

        {/* RED channel with strongest displacement */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="map"
          id="redchannel"
          xChannelSelector="R"
          yChannelSelector="G"
          result="dispRed"
          scale={config.scale + 0}
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

        {/* GREEN channel (reference / least displaced) */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="map"
          id="greenchannel"
          xChannelSelector="R"
          yChannelSelector="G"
          result="dispGreen"
          scale={config.scale + 10}
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

        {/* BLUE channel with medium displacement */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="map"
          id="bluechannel"
          xChannelSelector="R"
          yChannelSelector="G"
          result="dispBlue"
          scale={config.scale + 20}
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

        {/* Blend channels back together */}
        <feBlend in="red" in2="green" mode="screen" result="rg" />
        <feBlend in="rg" in2="blue" mode="screen" result="output" />

        {/* Final blur */}
        <feGaussianBlur in="output" stdDeviation="0.7" />
      </filter>
    </defs>
  </svg>
);

// Main LiquidGlass Component
export default function LiquidGlass({
  children,
  className = "",
  style = {},
  width,
  height,
  radius,
  mode = "standard",
  displacementScale,
  frost,
  border,
  alpha,
  lightness,
  blur,
  saturation,
  draggable = false,
  colorTint = "none",
  onClick,
  onMouseEnter,
  onMouseLeave,
}: LiquidGlassProps) {
  const filterId = useId();
  const containerRef = useRef<HTMLDivElement>(null);
  const [isHovered, setIsHovered] = useState(false);
  const [isActive, setIsActive] = useState(false);
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
  const [clickPosition, setClickPosition] = useState<{
    x: number;
    y: number;
  } | null>(null);
  const [clickDuration, setClickDuration] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isMobile, setIsMobile] = useState(false);

  const deviceTilt = useDeviceOrientation();

  // Detect mobile device
  useEffect(() => {
    setIsMobile(
      /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
        navigator.userAgent,
      ),
    );
  }, []);

  // Get preset config
  const preset = presets[mode] || presets.standard;
  const tint = colorTints[colorTint];

  // Merge config with props and preset
  const config = {
    width: width || preset.width,
    height: height || preset.height,
    radius: radius || preset.radius,
    scale: displacementScale || preset.scale,
    frost: frost !== undefined ? frost : preset.frost,
    border: border !== undefined ? border : preset.border,
    alpha: alpha !== undefined ? alpha : preset.alpha,
    lightness: lightness !== undefined ? lightness : preset.lightness,
    blur: blur !== undefined ? blur : preset.blur,
    saturation: saturation || preset.saturation,
    icons: preset.icons,
  };

  // Generate displacement map
  const displacementMapUrl = buildDisplacementImage(config);

  // Mouse tracking with border animation
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!containerRef.current || isDragging) return;

      const rect = containerRef.current.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      setMousePos({
        x: e.clientX - centerX,
        y: e.clientY - centerY,
      });
    },
    [isDragging],
  );

  // Click handling for white overlay expansion
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!containerRef.current) return;

      const rect = containerRef.current.getBoundingClientRect();
      setClickPosition({
        x: ((e.clientX - rect.left) / rect.width) * 100,
        y: ((e.clientY - rect.top) / rect.height) * 100,
      });
      setIsActive(true);
      setClickDuration(0);

      if (draggable) {
        setIsDragging(true);
        const startX = e.clientX - position.x;
        const startY = e.clientY - position.y;

        const handleGlobalMouseMove = (e: MouseEvent) => {
          setPosition({
            x: e.clientX - startX,
            y: e.clientY - startY,
          });
        };

        const handleGlobalMouseUp = () => {
          setIsDragging(false);
          document.removeEventListener("mousemove", handleGlobalMouseMove);
          document.removeEventListener("mouseup", handleGlobalMouseUp);
        };

        document.addEventListener("mousemove", handleGlobalMouseMove);
        document.addEventListener("mouseup", handleGlobalMouseUp);
      }

      // Increment click duration for white overlay expansion
      const interval = setInterval(() => {
        setClickDuration((prev) => {
          if (prev >= 100) {
            clearInterval(interval);
            return 100;
          }
          return prev + 2;
        });
      }, 20);

      const handleMouseUpForWhite = () => {
        clearInterval(interval);
        setClickDuration(0);
        setIsActive(false);
        document.removeEventListener("mouseup", handleMouseUpForWhite);
      };

      document.addEventListener("mouseup", handleMouseUpForWhite);
    },
    [draggable, position],
  );

  // Calculate border rotation based on mouse/tilt
  const calculateBorderRotation = () => {
    if (isMobile) {
      return Math.atan2(deviceTilt.y, deviceTilt.x) * (180 / Math.PI);
    }
    return Math.atan2(mousePos.y, mousePos.x) * (180 / Math.PI);
  };

  // Calculate elastic transformation
  const calculateTransform = () => {
    const elasticX = isMobile ? deviceTilt.x * 5 : mousePos.x * 0.02;
    const elasticY = isMobile ? deviceTilt.y * 5 : mousePos.y * 0.02;
    const scale = isActive ? 0.98 : 1;

    return `translate(${position.x + elasticX}px, ${position.y + elasticY}px) scale(${scale})`;
  };

  // Container styles
  const containerStyles: React.CSSProperties = {
    position: "relative",
    width: config.width,
    height: config.height,
    borderRadius: config.radius,
    transform: calculateTransform(),
    transition: isDragging ? "none" : "all 0.3s cubic-bezier(0.4, 0, 0.2, 1)",
    cursor: draggable
      ? isDragging
        ? "grabbing"
        : "grab"
      : onClick
        ? "pointer"
        : "default",
    ...style,
  };

  // Glass effect styles
  const glassStyles: React.CSSProperties = {
    width: "100%",
    height: "100%",
    borderRadius: "inherit",
    position: "relative",
    overflow: "hidden",
    background: `hsla(${tint.hue}, ${tint.saturation * 100}%, 100%, ${config.frost})`,
    backdropFilter: `url(#${filterId}) brightness(1.1) saturate(${config.saturation})`,
    boxShadow: `
      0 0 2px 1px rgba(255, 255, 255, 0.15) inset,
      0 0 10px 4px rgba(255, 255, 255, 0.1) inset,
      0px 4px 16px rgba(17, 17, 26, 0.05),
      0px 8px 24px rgba(17, 17, 26, 0.05),
      0px 16px 56px rgba(17, 17, 26, 0.05)
    `,
  };

  // Animated border styles
  const borderRotation = calculateBorderRotation();
  const borderIntensity = isHovered ? 0.8 : 0.4;
  const borderStyles: React.CSSProperties = {
    position: "absolute",
    inset: "-2px",
    borderRadius: "inherit",
    padding: "2px",
    background: `conic-gradient(
      from ${borderRotation}deg,
      hsla(${tint.hue}, ${tint.saturation * 100}%, 100%, ${borderIntensity}),
      hsla(${tint.hue + 60}, ${tint.saturation * 100}%, 90%, ${borderIntensity * 0.7}),
      hsla(${tint.hue + 120}, ${tint.saturation * 100}%, 95%, ${borderIntensity}),
      hsla(${tint.hue + 180}, ${tint.saturation * 100}%, 90%, ${borderIntensity * 0.5}),
      hsla(${tint.hue}, ${tint.saturation * 100}%, 100%, ${borderIntensity})
    )`,
    WebkitMask:
      "linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0)",
    WebkitMaskComposite: "xor",
    maskComposite: "exclude",
    opacity: isHovered ? 1 : 0.6,
    transition: "all 0.3s ease",
    pointerEvents: "none",
  };

  // White overlay for click expansion
  const whiteOverlayStyles: React.CSSProperties = {
    position: "absolute",
    borderRadius: "inherit",
    background:
      "radial-gradient(circle, rgba(255,255,255,0.6) 0%, rgba(255,255,255,0) 70%)",
    width: `${clickDuration * 3}%`,
    height: `${clickDuration * 3}%`,
    left: clickPosition ? `${clickPosition.x}%` : "50%",
    top: clickPosition ? `${clickPosition.y}%` : "50%",
    transform: "translate(-50%, -50%)",
    opacity: clickDuration > 0 ? 1 : 0,
    transition: "all 0.1s ease-out",
    pointerEvents: "none",
    mixBlendMode: "overlay",
  };

  // Content styles
  const contentStyles: React.CSSProperties = {
    position: "relative",
    zIndex: 1,
    width: "100%",
    height: "100%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "1rem",
    color: "white",
    textShadow: "0px 2px 12px rgba(0, 0, 0, 0.4)",
  };

  return (
    <>
      <LiquidGlassFilter
        id={filterId}
        config={config}
        displacementMapUrl={displacementMapUrl}
      />

      <div
        ref={containerRef}
        className={className}
        style={containerStyles}
        onMouseMove={handleMouseMove}
        onMouseEnter={() => {
          setIsHovered(true);
          onMouseEnter?.();
        }}
        onMouseLeave={() => {
          setIsHovered(false);
          onMouseLeave?.();
        }}
        onMouseDown={handleMouseDown}
        onClick={onClick}
      >
        {/* Animated border */}
        <div style={borderStyles} />

        {/* Glass effect */}
        <div style={glassStyles}>
          {/* White click expansion overlay */}
          {clickPosition && <div style={whiteOverlayStyles} />}

          {/* Hover enhancement */}
          <div
            style={{
              position: "absolute",
              inset: 0,
              borderRadius: "inherit",
              background: `hsla(${tint.hue}, ${tint.saturation * 50}%, 100%, ${isHovered ? 0.1 : 0})`,
              transition: "all 0.3s ease",
              pointerEvents: "none",
            }}
          />

          {/* Content */}
          <div style={contentStyles}>{children}</div>
        </div>
      </div>
    </>
  );
}

// Export preset configurations for external use
export { presets, colorTints };
export type { LiquidGlassProps };
