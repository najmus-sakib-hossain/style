"use client";

import gsap from "gsap";
import {
  Cloud,
  Eye,
  Frame,
  MoveHorizontal,
  MoveVertical,
  Palette,
  Palette as PaletteIcon,
  Redo,
  Settings,
  Snowflake,
  Sun,
  Undo,
  Zap,
} from "lucide-react";
import Image from "next/image";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const base = {
  icons: false,
  scale: -180,
  radius: 16,
  border: 0.07,
  lightness: 50,
  displace: 0,
  blend: "difference",
  x: "R",
  y: "B",
  alpha: 0.93,
  blur: 11,
  r: 0,
  g: 10,
  b: 20,
  saturation: 1,
  width: 336,
  height: 96,
  frost: 0.05,
  elasticity: 0.15,
};

type Config = typeof base;

const presets = {
  dock: {
    ...base,
    width: 336,
    height: 96,
    displace: 0.2,
    icons: true,
    frost: 0.05,
    elasticity: 0.1,
  },
  pill: {
    ...base,
    width: 200,
    height: 80,
    displace: 0,
    frost: 0,
    radius: 40,
    elasticity: 0.15,
  },
  bubble: {
    ...base,
    radius: 70,
    width: 140,
    height: 140,
    displace: 0,
    frost: 0,
    elasticity: 0.2,
  },
  free: {
    ...base,
    width: 140,
    height: 280,
    radius: 80,
    border: 0.15,
    alpha: 0.74,
    lightness: 60,
    blur: 10,
    displace: 0,
    scale: -300,
    elasticity: 0.25,
  },
};

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
        <svg class="displacement-image" viewBox="0 0 ${config.width} ${
        config.height
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
          <rect x="0" y="0" width="${config.width}" height="${
        config.height
      }" fill="black"></rect>
          <rect x="0" y="0" width="${config.width}" height="${
        config.height
      }" rx="${config.radius}" fill="url(#red)" />
          <rect x="0" y="0" width="${config.width}" height="${
        config.height
      }" rx="${
        config.radius
      }" fill="url(#blue)" style="mix-blend-mode: ${config.blend}" />
          <rect x="${border}" y="${border}" width="${
        config.width - border * 2
      }" height="${config.height - border * 2}" rx="${
        config.radius
      }" fill="hsl(0 0% ${config.lightness}% / ${
        config.alpha
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

  const transformStyle = `translate(${elasticTranslation.x}px, ${
    elasticTranslation.y
  }px) ${isActive ? "scale(0.96)" : directionalScale}`;

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
  };

  const borderStyle: React.CSSProperties = {
    ...baseStyle,
    borderRadius: `${config.radius}px`,
    pointerEvents: "none",
  };

  return (
    <>
      <Card className="fixed bottom-4 right-4 w-[320px] z-[999999999999999] rounded-[7px]">
        <CardHeader>
          <CardTitle>Liquid Glass Controls</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="bg-background max-h-[500px]">
            <Tabs
              defaultValue="dock"
              className="w-full"
              onValueChange={handlePresetChange}
            >
              <TabsList className="grid w-full grid-cols-4 border rounded-2xl">
                <TabsTrigger value="dock">Dock</TabsTrigger>
                <TabsTrigger value="pill">Pill</TabsTrigger>
                <TabsTrigger value="bubble">Bubble</TabsTrigger>
                <TabsTrigger value="free">Free</TabsTrigger>
              </TabsList>
              <TabsContent value="dock" />
              <TabsContent value="pill" />
              <TabsContent value="bubble" />
              <TabsContent value="free">
                <ScrollArea className="max-h-[400px]">
                  <div className="space-y-4 pt-4">
                    <h4 className="font-medium leading-none">Settings</h4>
                    <div className="space-y-2">
                      <Label>
                        <MoveHorizontal className="inline w-4 h-4 mr-2" />
                        Width: {config.width}px
                      </Label>
                      <Slider
                        defaultValue={[config.width]}
                        min={80}
                        max={500}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("width", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <MoveVertical className="inline w-4 h-4 mr-2" />
                        Height: {config.height}px
                      </Label>
                      <Slider
                        defaultValue={[config.height]}
                        min={35}
                        max={500}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("height", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Cloud className="inline w-4 h-4 mr-2" />
                        Radius: {config.radius}px
                      </Label>
                      <Slider
                        defaultValue={[config.radius]}
                        min={0}
                        max={500}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("radius", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Snowflake className="inline w-4 h-4 mr-2" />
                        Frost: {config.frost.toFixed(2)}
                      </Label>
                      <Slider
                        defaultValue={[config.frost]}
                        min={0}
                        max={1}
                        step={0.01}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("frost", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Palette className="inline w-4 h-4 mr-2" />
                        Saturation: {config.saturation.toFixed(1)}
                      </Label>
                      <Slider
                        defaultValue={[config.saturation]}
                        min={0}
                        max={2}
                        step={0.1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("saturation", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Frame className="inline w-4 h-4 mr-2" />
                        Border: {config.border.toFixed(2)}
                      </Label>
                      <Slider
                        defaultValue={[config.border]}
                        min={0}
                        max={1}
                        step={0.01}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("border", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Eye className="inline w-4 h-4 mr-2" />
                        Alpha: {config.alpha.toFixed(2)}
                      </Label>
                      <Slider
                        defaultValue={[config.alpha]}
                        min={0}
                        max={1}
                        step={0.01}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("alpha", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Sun className="inline w-4 h-4 mr-2" />
                        Lightness: {config.lightness}
                      </Label>
                      <Slider
                        defaultValue={[config.lightness]}
                        min={0}
                        max={100}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("lightness", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Cloud className="inline w-4 h-4 mr-2" />
                        Input Blur: {config.blur}
                      </Label>
                      <Slider
                        defaultValue={[config.blur]}
                        min={0}
                        max={20}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("blur", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Zap className="inline w-4 h-4 mr-2" />
                        Output Blur: {config.displace.toFixed(1)}
                      </Label>
                      <Slider
                        defaultValue={[config.displace]}
                        min={0}
                        max={12}
                        step={0.1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("displace", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Settings className="inline w-4 h-4 mr-2" />
                        Scale: {config.scale}
                      </Label>
                      <Slider
                        defaultValue={[config.scale]}
                        min={-1000}
                        max={1000}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("scale", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Zap className="inline w-4 h-4 mr-2" />
                        Elasticity: {config.elasticity.toFixed(2)}
                      </Label>
                      <Slider
                        defaultValue={[config.elasticity]}
                        min={0}
                        max={0.5}
                        step={0.01}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("elasticity", val)
                        }
                      />
                    </div>

                    <div className="flex items-center justify-between pt-4">
                      <h4 className="font-medium leading-none">
                        Chromatic Aberration
                      </h4>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={toggleChromaticAberration}
                      >
                        {isChromaticEnabled ? "Disable" : "Enable"}
                      </Button>
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Redo className="inline w-4 h-4 mr-2" />
                        Red: {config.r}
                      </Label>
                      <Slider
                        disabled={!isChromaticEnabled}
                        defaultValue={[config.r]}
                        min={-100}
                        max={100}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("r", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <Undo className="inline w-4 h-4 mr-2" />
                        Green: {config.g}
                      </Label>
                      <Slider
                        disabled={!isChromaticEnabled}
                        defaultValue={[config.g]}
                        min={-100}
                        max={100}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("g", val)
                        }
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>
                        <PaletteIcon className="inline w-4 h-4 mr-2" />
                        Blue: {config.b}
                      </Label>
                      <Slider
                        disabled={!isChromaticEnabled}
                        defaultValue={[config.b]}
                        min={-100}
                        max={100}
                        step={1}
                        onValueChange={([val]: number[]) =>
                          handleConfigChange("b", val)
                        }
                      />
                    </div>

                    <h4 className="font-medium leading-none pt-4">
                      Channels & Blend
                    </h4>
                    <div className="flex items-center space-x-4">
                      <div className="space-y-2 w-1/2">
                        <Label>Channel X</Label>
                        <Select
                          value={config.x}
                          onValueChange={(val: string) =>
                            handleConfigChange("x", val)
                          }
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="R">Red</SelectItem>
                            <SelectItem value="G">Green</SelectItem>
                            <SelectItem value="B">Blue</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2 w-1/2">
                        <Label>Channel Y</Label>
                        <Select
                          value={config.y}
                          onValueChange={(val: string) =>
                            handleConfigChange("y", val)
                          }
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="R">Red</SelectItem>
                            <SelectItem value="G">Green</SelectItem>
                            <SelectItem value="B">Blue</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>Blend Mode</Label>
                      <Select
                        value={config.blend}
                        onValueChange={(val: string) =>
                          handleConfigChange("blend", val)
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {[
                            "normal",
                            "multiply",
                            "screen",
                            "overlay",
                            "darken",
                            "lighten",
                            "color-dodge",
                            "color-burn",
                            "hard-light",
                            "soft-light",
                            "difference",
                            "exclusion",
                            "hue",
                            "saturation",
                            "color",
                            "luminosity",
                            "plus-darker",
                            "plus-lighter",
                          ].map((mode) => (
                            <SelectItem key={mode} value={mode}>
                              {mode}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="flex items-center space-x-2 pt-4">
                      <Switch
                        id="icons-mode"
                        checked={config.icons}
                        onCheckedChange={(val: boolean) =>
                          handleConfigChange("icons", val)
                        }
                      />
                      <Label htmlFor="icons-mode">Show Icons</Label>
                    </div>
                  </div>
                </ScrollArea>
              </TabsContent>
            </Tabs>
          </ScrollArea>
        </CardContent>
      </Card>

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

      <span
        className="effect-border"
        style={{
          ...borderStyle,
          mixBlendMode: "screen",
          opacity: 0.2,
          padding: "0.5px",
          WebkitMask:
            "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
          WebkitMaskComposite: "xor",
          maskComposite: "exclude",
          background: `linear-gradient(
            ${135 + mouseOffset.x * 1.2}deg,
            rgba(255, 255, 255, 0.0) 0%,
            rgba(255, 255, 255, ${
              0.12 + Math.abs(mouseOffset.x) * 0.008
            }) ${Math.max(10, 33 + mouseOffset.y * 0.3)}%,
            rgba(255, 255, 255, ${
              0.4 + Math.abs(mouseOffset.x) * 0.012
            }) ${Math.min(90, 66 + mouseOffset.y * 0.4)}%,
            rgba(255, 255, 255, 0.0) 100%
          )`,
        }}
      />
      <span
        className="effect-border"
        style={{
          ...borderStyle,
          mixBlendMode: "overlay",
          padding: "0.5px",
          WebkitMask:
            "linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)",
          WebkitMaskComposite: "xor",
          maskComposite: "exclude",
          background: `linear-gradient(
            ${135 + mouseOffset.x * 1.2}deg,
            rgba(255, 255, 255, 0.0) 0%,
            rgba(255, 255, 255, ${
              0.32 + Math.abs(mouseOffset.x) * 0.008
            }) ${Math.max(10, 33 + mouseOffset.y * 0.3)}%,
            rgba(255, 255, 255, ${
              0.6 + Math.abs(mouseOffset.x) * 0.012
            }) ${Math.min(90, 66 + mouseOffset.y * 0.4)}%,
            rgba(255, 255, 255, 0.0) 100%
          )`,
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
