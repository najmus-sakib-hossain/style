"use client";

import gsap from "gsap";
import { Draggable } from "gsap/all";
import {
  Blend,
  Circle,
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
  ToggleLeft,
  Undo,
  Zap,
} from "lucide-react";
import Head from "next/head";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
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
  },
  pill: {
    ...base,
    width: 200,
    height: 80,
    displace: 0,
    frost: 0,
    radius: 40,
  },
  bubble: {
    ...base,
    radius: 70,
    width: 140,
    height: 140,
    displace: 0,
    frost: 0,
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
  },
};

const LiquidGlassPage = () => {
  const effectRef = useRef<HTMLDivElement>(null);
  const debugPenRef = useRef<HTMLDivElement>(null);
  const dockPlaceholderRef = useRef<HTMLDivElement>(null);

  const [config, setConfig] = useState(presets.dock);

  useEffect(() => {
    if (typeof window !== "undefined") {
      gsap.registerPlugin(Draggable);
    }
  }, []);

  const handleConfigChange = (
    key: keyof Config,
    value: string | number | boolean,
  ) => {
    setConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handlePresetChange = (presetName: string) => {
    const newConfig = presets[presetName as keyof typeof presets];
    gsap.to(config, {
      ...newConfig,
      duration: 0.5,
      onUpdate: () => {
        setConfig({ ...config });
      },
    });
  };

  useEffect(() => {
    if (typeof window === "undefined") return;

    const buildDisplacementImage = () => {
      if (!debugPenRef.current) return;
      const border =
        Math.min(config.width, config.height) * (config.border * 0.5);
      const kids = `
        <svg class="displacement-image" viewBox="0 0 ${
          config.width
        } ${config.height}" xmlns="http://www.w3.org/2000/svg">
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
          <rect x="0" y="0" width="${config.width}" height="${config.height}" rx="${config.radius}" fill="url(#blue)" style="mix-blend-mode: ${config.blend}" />
          <rect x="${border}" y="${border}" width="${config.width - border * 2}" height="${config.height - border * 2}" rx="${config.radius}" fill="hsl(0 0% ${config.lightness}% / ${config.alpha})" style="filter:blur(${config.blur}px)" />
        </svg>
        <div class="label">
          <span>displacement image</span>
          <svg viewBox="0 0 97 52" fill="none" xmlns="http://www.w3.org/2000/svg">
            <title>Hand-drawn arrow</title>
            <path fill-rule="evenodd" clip-rule="evenodd" d="M74.568 0.553803C74.0753 0.881909 73.6295 1.4678 73.3713 2.12401C73.1367 2.70991 72.3858 4.67856 71.6584 6.50658C70.9544 8.35803 69.4526 11.8031 68.3498 14.1936C66.1441 19.0214 65.839 20.2167 66.543 21.576C67.4581 23.3337 69.4527 23.9196 71.3064 22.9821C72.4797 22.3728 74.8965 19.5839 76.9615 16.4435C78.8387 13.5843 78.8387 13.6077 78.1113 18.3418C77.3369 23.4275 76.4687 26.2866 74.5915 30.0364C73.254 32.7316 71.8461 34.6299 69.218 37.3485C65.9563 40.6999 62.2254 42.9732 57.4385 44.4965C53.8718 45.6449 52.3935 45.8324 47.2546 45.8324C43.3594 45.8324 42.1158 45.7386 39.9805 45.2933C32.2604 43.7466 25.3382 40.9577 19.4015 36.9735C15.0839 34.0909 12.5028 31.7004 9.80427 27.9975C6.80073 23.9196 4.36038 17.2403 3.72682 11.475C3.37485 8.1471 3.1402 7.32683 2.43624 7.13934C0.770217 6.71749 0.183578 7.77211 0.0193217 11.5219C-0.26226 18.5996 2.55356 27.1304 7.17619 33.1066C13.8403 41.7545 25.432 48.4103 38.901 51.2696C41.6465 51.8555 42.2566 51.9023 47.4893 51.9023C52.3935 51.9023 53.426 51.832 55.5144 51.3867C62.2723 49.9337 68.5375 46.6292 72.949 42.1998C76.0464 39.1296 78.1113 36.2939 79.8946 32.7081C82.1942 28.0912 83.5317 23.3103 84.2591 17.17C84.3999 15.8576 84.6111 14.7795 84.7284 14.7795C84.8223 14.7795 85.4559 15.1311 86.1364 15.5763C88.037 16.7716 90.3835 17.8965 93.5748 19.0918C96.813 20.3339 97.3996 20.287 96.4141 18.9512C94.9123 16.9122 90.055 11.5219 87.1219 8.63926C84.0949 5.66288 83.8368 5.33477 83.5552 4.1864C83.3909 3.48332 83.0155 2.68649 82.6401 2.31151C82.0065 1.6553 80.4109 1.04595 79.9885 1.30375C79.8712 1.37406 79.2845 1.11626 78.6744 0.717845C77.2431 -0.172727 75.7413 -0.243024 74.568 0.553803Z" fill="currentColor"></path>
          </svg>
        </div>
      `;
      debugPenRef.current.innerHTML = kids;
      const svgEl = debugPenRef.current.querySelector(".displacement-image");
      if (svgEl) {
        const serialized = new XMLSerializer().serializeToString(svgEl);
        const encoded = encodeURIComponent(serialized);
        const dataUri = `data:image/svg+xml,${encoded}`;
        gsap.set("feImage", { attr: { href: dataUri } });
      }
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

    if (effectRef.current) {
      Draggable.create(effectRef.current, { type: "x,y" });
    }

    if (dockPlaceholderRef.current) {
      const { top, left } = dockPlaceholderRef.current.getBoundingClientRect();
      gsap.set(".effect", {
        top: top > window.innerHeight ? window.innerHeight * 0.5 : top,
        left,
        opacity: 1,
      });
    }

    return () => {
      if (effectRef.current) {
        const draggableInstance = Draggable.get(effectRef.current);
        if (draggableInstance) {
          draggableInstance.kill();
        }
      }
    };
  }, [config]);

  return (
    <>
      <Head>
        <title>Liquid Glass</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin="anonymous"
        />
        <link
          href="https://fonts.googleapis.com/css2?family=Gloria+Hallelujah&display=swap"
          rel="stylesheet"
        />
      </Head>

      <Card className="fixed bottom-4 right-4 w-[320px] z-[999999999999999]">
        <CardHeader>
          <CardTitle>Glass Controls</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="bg-background max-h-[500px]">
            <Tabs
              defaultValue="dock"
              className="w-full"
              onValueChange={handlePresetChange}
            >
              <TabsList className="grid w-full grid-cols-4 border">
                <TabsTrigger value="dock">Dock</TabsTrigger>
                <TabsTrigger value="pill">Pill</TabsTrigger>
                <TabsTrigger value="bubble">Bubble</TabsTrigger>
                <TabsTrigger value="free">Free</TabsTrigger>
              </TabsList>
              <TabsContent value="dock" />
              <TabsContent value="pill" />
              <TabsContent value="bubble" />
              <TabsContent value="free">
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
                      <Circle className="inline w-4 h-4 mr-2" />
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

                  <h4 className="font-medium leading-none pt-4">
                    Chromatic Aberration
                  </h4>
                  <div className="space-y-2">
                    <Label>
                      <Redo className="inline w-4 h-4 mr-2" />
                      Red: {config.r}
                    </Label>
                    <Slider
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
                    <Label>
                      <Blend className="inline w-4 h-4 mr-2" />
                      Blend Mode
                    </Label>
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
                    <Label htmlFor="icons-mode">
                      <ToggleLeft className="inline w-4 h-4 mr-2" />
                      Show Icons
                    </Label>
                  </div>
                </div>
              </TabsContent>
            </Tabs>
          </ScrollArea>
        </CardContent>
      </Card>

      <header>
        <h1 className="fluid">
          glass
          <br />
          displacement
        </h1>
        <p className="fluid">
          it&apos;s not perfect, but neither is the platform.
          <br />
          we love it anyway.
        </p>
      </header>
      <div className="effect" ref={effectRef}>
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
        <div className="displacement-debug" ref={debugPenRef} />
      </div>
      <main>
        <section className="placeholder">
          <div className="dock-placeholder" ref={dockPlaceholderRef} />
          <span className="arrow arrow--debug">
            <span>drag, scroll, configure</span>
            <svg
              viewBox="0 0 122 97"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <title>Hand-drawn arrow indicating interaction</title>
              <path
                fillRule="evenodd"
                clipRule="evenodd"
                d="M116.102 0.0996005C114.952 0.334095 112.7 1.53002 111.433 2.53834C110.869 2.98388 109.368 4.15635 108.077 5.11778C103.455 8.6352 102.61 9.40903 102.187 10.4877C101.39 12.5982 102.798 14.5914 105.097 14.5914C106.13 14.5914 108.241 13.7941 109.696 12.8561C110.424 12.3871 111.01 12.0823 111.01 12.1526C111.01 12.692 107.796 17.8274 106.2 19.8206C102.023 25.0733 95.6642 29.6928 86.2548 34.2889C81.0926 36.8214 77.4555 38.2753 73.9123 39.2367C71.7066 39.823 70.6507 39.9871 67.9053 40.0809C66.0516 40.1513 64.5499 40.1747 64.5499 40.1278C64.5499 40.0809 64.808 38.9788 65.1365 37.6891C65.465 36.3993 65.8404 34.1716 66.0047 32.7647C66.4505 28.3796 65.4884 24.2994 63.4704 22.2359C62.1564 20.8758 60.9363 20.3599 59.0121 20.3599C57.6043 20.3599 57.1115 20.4537 55.7975 21.1103C52.8878 22.5407 50.5648 25.9878 49.5089 30.4197C48.453 34.922 49.2742 38.0877 52.3481 41.1127C53.4744 42.2148 54.46 42.9183 55.9852 43.6921C57.1584 44.2549 58.1439 44.7473 58.1909 44.7708C58.5898 45.0053 54.5304 53.4705 52.0666 57.6211C47.4674 65.3125 39.3486 74.575 30.5728 82.0789C22.2427 89.2309 16.7285 92.4435 9.87677 94.1553C8.28116 94.554 7.13138 94.6478 4.2452 94.6478C1.17131 94.6712 0.608154 94.7181 0.608154 95.023C0.608154 95.234 1.19478 95.5857 2.13337 95.9609C3.54126 96.4768 3.96363 96.5472 7.41296 96.5237C10.5572 96.5237 11.4724 96.4299 13.1149 96.0078C21.7265 93.6863 31.1594 87.1908 42.6102 75.7006C49.2977 69.0175 52.5828 64.9373 56.1494 58.9343C58.0501 55.7217 60.6312 50.6801 61.7575 47.9365L62.5553 45.9902L64.0806 46.1543C71.3547 46.9047 77.7136 45.3101 88.3667 40.034C96.2274 36.1414 101.976 32.3426 106.505 28.0748C108.617 26.0816 111.855 22.2828 112.794 20.7117C113.028 20.313 113.286 19.9847 113.357 19.9847C113.427 19.9847 113.662 20.782 113.873 21.72C114.084 22.6814 114.647 24.276 115.093 25.2609C115.82 26.8085 116.008 27.043 116.454 26.9727C116.876 26.9258 117.228 26.4333 117.956 24.9795C119.317 22.2828 119.833 20.2661 120.772 13.8879C121.757 7.25168 121.781 4.4143 120.889 2.56179C119.95 0.615488 118.12 -0.322489 116.102 0.0996005ZM60.7016 25.7767C61.4525 26.9023 61.8279 29.2942 61.6637 31.9205C61.4759 34.7813 60.5139 38.9788 60.0681 38.9788C59.5284 38.9788 57.1584 37.6422 56.2198 36.8214C54.8354 35.6021 54.3426 34.2889 54.5538 32.2957C54.8589 29.2473 56.1964 26.2223 57.5808 25.3547C58.7306 24.6512 60.0681 24.8388 60.7016 25.7767Z"
                fill="currentColor"
              />
            </svg>
          </span>
        </section>
        <section>
          <p className="fluid">
            How do you create backdrop displacement with HTML and CSS? SVG. The
            idea is that you need a displacement map that distorts the input
            image. In this case, the backdrop of our element (whatever is
            underneath).
          </p>
        </section>
        <section>
          <div className="apps">
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/beeper.png"
                alt="Beeper app icon"
                width={80}
                height={80}
              />
              <span>Beeper</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/cursor.png"
                alt="Cursor app icon"
                width={80}
                height={80}
              />
              <span>Cursor</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/screenstudio.png"
                alt="Screen Studio app icon"
                width={80}
                height={80}
              />
              <span>Screen Studio</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/raycast.png"
                alt="Raycast app icon"
                width={80}
                height={80}
              />
              <span>Raycast</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/photos.png"
                alt="Photos app icon"
                width={80}
                height={80}
              />
              <span>Photos</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/signal.png"
                alt="Signal app icon"
                width={80}
                height={80}
              />
              <span>Signal</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/spotify.png"
                alt="Spotify app icon"
                width={80}
                height={80}
              />
              <span>Spotify</span>
            </div>
            <div className="app">
              <Image
                src="https://assets.codepen.io/605876/brave.png"
                alt="Brave browser app icon"
                width={80}
                height={80}
              />
              <span>Brave</span>
            </div>
          </div>
        </section>
        <section>
          <p className="fluid">
            Check the &quot;debug&quot; option to see the displacement map used
            and play with the options. The red and blue of the displacement map
            displaces the backdrop. The caveats? You need to update the map
            image whenever the shape of the element changes. The big one?
            backdrop-filter: url() currently only works in Chromium and not
            Gecko/Webkit.
          </p>
        </section>
        <section className="emojis fluid">
          <span>🧑‍🍳</span>
          <span>🤓</span>
          <span>🤪</span>
          <span>🙄</span>
          <span>🤠</span>
          <span>🥸</span>
        </section>
        <section>
          <p className="fluid">
            Glass adjusts for different surfaces and use cases.
            <br />
            Try different modes or freestyle it in &quot;free&quot; mode.
          </p>
        </section>
        <section className="images">
          <Image
            src="https://fastly.picsum.photos/id/841/300/300.jpg?hmac=59ZNBwU1FjRrwpU3J7NDerfr_DHq-JPYXqnyumDt17U"
            alt="A random placeholder image of a landscape"
            width={300}
            height={300}
          />
          <Image
            src="https://fastly.picsum.photos/id/517/300/300.jpg?hmac=xDY76wxtwOZ5mEJYjf_i69VkVQibAYi036aADsWbaLs"
            alt="A random placeholder image of a city"
            width={300}
            height={300}
          />
          <Image
            src="https://fastly.picsum.photos/id/204/300/300.jpg?hmac=nMn3k2irZFRlOEdAxFaKapzdO5cuwF8eQv5HbhP9Lyw"
            alt="A random placeholder image of architecture"
            width={300}
            height={300}
          />
          <Image
            src="https://fastly.picsum.photos/id/906/300/300.jpg?hmac=UJKxYXpNPOY5aqp_mipycmJFPgr8bd3RxcZChdDu0-c"
            alt="A random placeholder image of nature"
            width={300}
            height={300}
          />
          <Image
            src="https://fastly.picsum.photos/id/546/300/300.jpg?hmac=f8E2wXr3kthnt3BT6h17Y5Baf_0aJUdPIV7GqBVgxzY"
            alt="A random placeholder image of a person"
            width={300}
            height={300}
          />
        </section>
        <section>
          <p className="fluid">
            That&apos;s it. Go forth and play with SVG filters.
            <br />
            But maybe do it in Safari and Firefox first.
          </p>
        </section>
      </main>
      <footer>┬┴┬┴┤•ᴥ•ʔ jhey &copy; 2025 ├┬┴┬┴</footer>
      <a
        aria-label="Follow Jhey"
        className="bear-link"
        href="https://twitter.com/intent/follow?screen_name=jh3yy"
        target="_blank"
        rel="noreferrer noopener"
      >
        <svg
          className="w-9"
          viewBox="0 0 969 955"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <title>Jhey's bear logo, link to Twitter profile</title>
          <circle
            cx="161.191"
            cy="320.191"
            r="133.191"
            stroke="currentColor"
            strokeWidth="20"
          />
          <circle
            cx="806.809"
            cy="320.191"
            r="133.191"
            stroke="currentColor"
            strokeWidth="20"
          />
          <circle cx="695.019" cy="587.733" r="31.4016" fill="currentColor" />
          <circle cx="272.981" cy="587.733" r="31.4016" fill="currentColor" />
          <path
            d="M564.388 712.083C564.388 743.994 526.035 779.911 483.372 779.911C440.709 779.911 402.356 743.994 402.356 712.083C402.356 680.173 440.709 664.353 483.372 664.353C526.035 664.353 564.388 680.173 564.388 712.083Z"
            fill="currentColor"
          />
          <rect
            x="310.42"
            y="448.31"
            width="343.468"
            height="51.4986"
            fill="#FF1E1E"
          />
          <path
            fillRule="evenodd"
            clipRule="evenodd"
            d="M745.643 288.24C815.368 344.185 854.539 432.623 854.539 511.741H614.938V454.652C614.938 433.113 597.477 415.652 575.938 415.652H388.37C366.831 415.652 349.37 433.113 349.37 454.652V511.741L110.949 511.741C110.949 432.623 150.12 344.185 219.845 288.24C289.57 232.295 384.138 200.865 482.744 200.865C581.35 200.865 675.918 232.295 745.643 288.24Z"
            fill="currentColor"
          />
        </svg>
      </a>
    </>
  );
};

export default LiquidGlassPage;
