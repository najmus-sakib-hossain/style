'use client'

import React, { useRef, useState, useEffect, useCallback, useId, type CSSProperties } from 'react'
import { displacementMap, polarDisplacementMap, prominentDisplacementMap } from './liquid-glass-utils'

// Types
interface LiquidGlassProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
  width?: number
  height?: number
  radius?: number
  mode?: 'dock' | 'pill' | 'bubble' | 'free' | 'standard'
  displacementScale?: number
  blurAmount?: number
  saturation?: number
  aberrationIntensity?: number
  elasticity?: number
  frost?: number
  border?: number
  alpha?: number
  lightness?: number
  blend?: string
  draggable?: boolean
  onClick?: () => void
  onMouseEnter?: () => void
  onMouseLeave?: () => void
}

// Preset configurations
const presets = {
  dock: {
    width: 336,
    height: 96,
    radius: 16,
    displacementScale: -180,
    frost: 0.05,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
  },
  pill: {
    width: 200,
    height: 80,
    radius: 40,
    displacementScale: -180,
    frost: 0,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
  },
  bubble: {
    width: 140,
    height: 140,
    radius: 70,
    displacementScale: -180,
    frost: 0,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
  },
  free: {
    width: 140,
    height: 280,
    radius: 80,
    displacementScale: -300,
    frost: 0,
    border: 0.15,
    alpha: 0.74,
    lightness: 60,
  },
  standard: {
    width: 270,
    height: 69,
    radius: 16,
    displacementScale: 70,
    frost: 0.05,
    border: 0.07,
    alpha: 0.93,
    lightness: 50,
  },
}

// Get displacement map based on mode
const getDisplacementMap = (mode: string) => {
  switch (mode) {
    case 'standard':
    case 'dock':
    case 'pill':
    case 'bubble':
    case 'free':
      return displacementMap
    default:
      return displacementMap
  }
}

// Generate dynamic displacement image
const generateDisplacementImage = (config: any) => {
  const border = Math.min(config.width, config.height) * (config.border * 0.5)

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
      <rect x="0" y="0" width="${config.width}" height="${config.height}" rx="${config.radius}" fill="url(#blue)" style="mix-blend-mode: ${config.blend || 'difference'}" />
      <rect x="${border}" y="${border}" width="${config.width - border * 2}" height="${config.height - border * 2}" rx="${config.radius}" fill="hsl(0 0% ${config.lightness}% / ${config.alpha})" style="filter:blur(${config.blur || 11}px)" />
    </svg>
  `

  const encoded = encodeURIComponent(svgString)
  return `data:image/svg+xml,${encoded}`
}

// SVG Filter Component
const LiquidGlassFilter: React.FC<{
  id: string
  config: any
  displacementMapUrl: string
}> = ({ id, config, displacementMapUrl }) => (
  <svg style={{ position: 'absolute', width: 0, height: 0 }} aria-hidden="true">
    <defs>
      <filter id={id} x="-35%" y="-35%" width="170%" height="170%" colorInterpolationFilters="sRGB">
        <feImage
          x="0"
          y="0"
          width="100%"
          height="100%"
          result="DISPLACEMENT_MAP"
          href={displacementMapUrl}
          preserveAspectRatio="xMidYMid slice"
        />

        {/* Red channel displacement */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="DISPLACEMENT_MAP"
          scale={config.displacementScale + (config.r || 0)}
          xChannelSelector="R"
          yChannelSelector="G"
          result="RED_DISPLACED"
        />
        <feColorMatrix
          in="RED_DISPLACED"
          type="matrix"
          values="1 0 0 0 0
                 0 0 0 0 0
                 0 0 0 0 0
                 0 0 0 1 0"
          result="RED_CHANNEL"
        />

        {/* Green channel displacement */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="DISPLACEMENT_MAP"
          scale={config.displacementScale + (config.g || 10)}
          xChannelSelector="R"
          yChannelSelector="G"
          result="GREEN_DISPLACED"
        />
        <feColorMatrix
          in="GREEN_DISPLACED"
          type="matrix"
          values="0 0 0 0 0
                 0 1 0 0 0
                 0 0 0 0 0
                 0 0 0 1 0"
          result="GREEN_CHANNEL"
        />

        {/* Blue channel displacement */}
        <feDisplacementMap
          in="SourceGraphic"
          in2="DISPLACEMENT_MAP"
          scale={config.displacementScale + (config.b || 20)}
          xChannelSelector="R"
          yChannelSelector="G"
          result="BLUE_DISPLACED"
        />
        <feColorMatrix
          in="BLUE_DISPLACED"
          type="matrix"
          values="0 0 0 0 0
                 0 0 0 0 0
                 0 0 1 0 0
                 0 0 0 1 0"
          result="BLUE_CHANNEL"
        />

        {/* Combine channels */}
        <feBlend in="RED_CHANNEL" in2="GREEN_CHANNEL" mode="screen" result="RG_COMBINED" />
        <feBlend in="RG_COMBINED" in2="BLUE_CHANNEL" mode="screen" result="RGB_COMBINED" />

        {/* Final blur */}
        <feGaussianBlur in="RGB_COMBINED" stdDeviation={config.displace || 0} />
      </filter>
    </defs>
  </svg>
)

// Main LiquidGlass Component
export default function LiquidGlass({
  children,
  className = '',
  style = {},
  width,
  height,
  radius,
  mode = 'standard',
  displacementScale,
  blurAmount = 4,
  saturation = 150,
  aberrationIntensity = 2,
  elasticity = 0.15,
  frost,
  border,
  alpha,
  lightness,
  blend,
  draggable = false,
  onClick,
  onMouseEnter,
  onMouseLeave,
}: LiquidGlassProps) {
  const filterId = useId()
  const containerRef = useRef<HTMLDivElement>(null)
  const [isHovered, setIsHovered] = useState(false)
  const [isActive, setIsActive] = useState(false)
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 })
  const [mouseOffset, setMouseOffset] = useState({ x: 0, y: 0 })
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 })
  const [isDragging, setIsDragging] = useState(false)

  // Get preset config
  const preset = presets[mode as keyof typeof presets] || presets.standard

  // Merge config with props and preset
  const config = {
    width: width || preset.width,
    height: height || preset.height,
    radius: radius || preset.radius,
    displacementScale: displacementScale || preset.displacementScale,
    frost: frost !== undefined ? frost : preset.frost,
    border: border !== undefined ? border : preset.border,
    alpha: alpha !== undefined ? alpha : preset.alpha,
    lightness: lightness !== undefined ? lightness : preset.lightness,
    blend: blend || 'difference',
    r: 0,
    g: 10,
    b: 20,
    displace: aberrationIntensity * 0.5,
    blur: 11,
  }

  // Generate displacement map
  const displacementMapUrl = generateDisplacementImage(config)

  // Mouse tracking
  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!containerRef.current) return

    const rect = containerRef.current.getBoundingClientRect()
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2

    setMousePos({ x: e.clientX, y: e.clientY })
    setMouseOffset({
      x: ((e.clientX - centerX) / rect.width) * 100,
      y: ((e.clientY - centerY) / rect.height) * 100,
    })
  }, [])

  // Calculate elastic transformation
  const calculateTransform = useCallback(() => {
    if (!containerRef.current) return 'translate(-50%, -50%)'

    const rect = containerRef.current.getBoundingClientRect()
    const centerX = rect.left + rect.width / 2
    const centerY = rect.top + rect.height / 2

    const deltaX = mousePos.x - centerX
    const deltaY = mousePos.y - centerY
    const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY)

    // Elastic effect within 200px radius
    const maxDistance = 200
    const factor = Math.max(0, 1 - distance / maxDistance) * elasticity

    const translateX = deltaX * factor * 0.1 + dragOffset.x
    const translateY = deltaY * factor * 0.1 + dragOffset.y

    // Scale effect
    const scaleX = 1 + Math.abs(deltaX / rect.width) * factor * 0.3
    const scaleY = 1 + Math.abs(deltaY / rect.height) * factor * 0.3
    const scale = isActive ? 'scale(0.96)' : `scaleX(${Math.max(0.8, scaleX)}) scaleY(${Math.max(0.8, scaleY)})`

    return `translate(calc(-50% + ${translateX}px), calc(-50% + ${translateY}px)) ${scale}`
  }, [mousePos, elasticity, dragOffset, isActive])

  // Drag functionality
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!draggable) return
    setIsDragging(true)
    setIsActive(true)

    const rect = containerRef.current?.getBoundingClientRect()
    if (rect) {
      const startX = e.clientX - rect.left - rect.width / 2
      const startY = e.clientY - rect.top - rect.height / 2

      const handleGlobalMouseMove = (e: MouseEvent) => {
        setDragOffset({
          x: e.clientX - rect.left - rect.width / 2 - startX,
          y: e.clientY - rect.top - rect.height / 2 - startY,
        })
      }

      const handleGlobalMouseUp = () => {
        setIsDragging(false)
        setIsActive(false)
        document.removeEventListener('mousemove', handleGlobalMouseMove)
        document.removeEventListener('mouseup', handleGlobalMouseUp)
      }

      document.addEventListener('mousemove', handleGlobalMouseMove)
      document.addEventListener('mouseup', handleGlobalMouseUp)
    }
  }, [draggable])

  // Combined styles
  const containerStyles: CSSProperties = {
    position: 'relative',
    top: '50%',
    left: '50%',
    width: config.width,
    height: config.height,
    borderRadius: config.radius,
    transform: calculateTransform(),
    transition: isDragging ? 'none' : 'all 0.2s ease-out',
    cursor: draggable ? 'grab' : onClick ? 'pointer' : 'default',
    ...style,
  }

  const glassStyles: CSSProperties = {
    width: '100%',
    height: '100%',
    borderRadius: 'inherit',
    background: `hsl(0 0% 100% / ${config.frost})`,
    backdropFilter: `url(#${filterId}) blur(${blurAmount}px) saturate(${saturation}%)`,
    boxShadow: `
      0 0 2px 1px rgba(255, 255, 255, 0.15) inset,
      0 0 10px 4px rgba(255, 255, 255, 0.1) inset,
      0px 4px 16px rgba(17, 17, 26, 0.05),
      0px 8px 24px rgba(17, 17, 26, 0.05),
      0px 16px 56px rgba(17, 17, 26, 0.05)
    `,
    overflow: 'hidden',
  }

  const contentStyles: CSSProperties = {
    position: 'relative',
    zIndex: 1,
    width: '100%',
    height: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '1rem',
    color: 'white',
    textShadow: '0px 2px 12px rgba(0, 0, 0, 0.4)',
    pointerEvents: onClick ? 'auto' : 'none',
  }

  // Border gradient styles
  const borderStyles: CSSProperties = {
    position: 'absolute',
    inset: 0,
    borderRadius: 'inherit',
    padding: '1.5px',
    background: `linear-gradient(
      ${135 + mouseOffset.x * 1.2}deg,
      rgba(255, 255, 255, 0.0) 0%,
      rgba(255, 255, 255, ${0.12 + Math.abs(mouseOffset.x) * 0.008}) ${Math.max(10, 33 + mouseOffset.y * 0.3)}%,
      rgba(255, 255, 255, ${0.4 + Math.abs(mouseOffset.x) * 0.012}) ${Math.min(90, 66 + mouseOffset.y * 0.4)}%,
      rgba(255, 255, 255, 0.0) 100%
    )`,
    WebkitMask: 'linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)',
    WebkitMaskComposite: 'xor',
    maskComposite: 'exclude',
    mixBlendMode: 'screen',
    opacity: 0.6,
    pointerEvents: 'none',
  }

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
          setIsHovered(true)
          onMouseEnter?.()
        }}
        onMouseLeave={() => {
          setIsHovered(false)
          onMouseLeave?.()
        }}
        onMouseDown={handleMouseDown}
        onClick={onClick}
      >
        {/* Border layer */}
        <div style={borderStyles} />

        {/* Glass layer */}
        <div style={glassStyles}>
          {/* Content */}
          <div style={contentStyles}>
            {children}
          </div>
        </div>

        {/* Hover effects */}
        {onClick && (
          <>
            <div
              style={{
                position: 'absolute',
                inset: 0,
                borderRadius: 'inherit',
                opacity: isHovered || isActive ? 0.3 : 0,
                background: 'radial-gradient(circle at 50% 0%, rgba(255, 255, 255, 0.5) 0%, rgba(255, 255, 255, 0) 50%)',
                mixBlendMode: 'overlay',
                transition: 'opacity 0.2s ease-out',
                pointerEvents: 'none',
              }}
            />
            <div
              style={{
                position: 'absolute',
                inset: 0,
                borderRadius: 'inherit',
                opacity: isActive ? 0.5 : 0,
                background: 'radial-gradient(circle at 50% 0%, rgba(255, 255, 255, 1) 0%, rgba(255, 255, 255, 0) 80%)',
                mixBlendMode: 'overlay',
                transition: 'opacity 0.2s ease-out',
                pointerEvents: 'none',
              }}
            />
          </>
        )}
      </div>
    </>
  )
}

// Export preset configurations for external use
export { presets }
export type { LiquidGlassProps }
