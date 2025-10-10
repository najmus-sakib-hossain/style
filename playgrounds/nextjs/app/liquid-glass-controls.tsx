"use client";

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

type Config = {
    icons: boolean;
    scale: number;
    radius: number;
    border: number;
    lightness: number;
    displace: number;
    blend: string;
    x: string;
    y: string;
    alpha: number;
    blur: number;
    r: number;
    g: number;
    b: number;
    saturation: number;
    width: number;
    height: number;
    frost: number;
    elasticity: number;
};

interface LiquidGlassControlsProps {
    config: Config;
    handleConfigChange: (key: keyof Config, value: string | number | boolean) => void;
    handlePresetChange: (presetName: string) => void;
    toggleChromaticAberration: () => void;
    isChromaticEnabled: boolean;
}

export default function LiquidGlassControls({
    config,
    handleConfigChange,
    handlePresetChange,
    toggleChromaticAberration,
    isChromaticEnabled,
}: LiquidGlassControlsProps) {
    return (
        <Card className="fixed bottom-4 right-4 w-[320px] z-[999999999999999] rounded-[7px]">
            <CardHeader>
                <CardTitle>Liquid Glass Controls</CardTitle>
            </CardHeader>
            <CardContent>
                <ScrollArea className="bg-background max-h-[500px]">
                    <Tabs
                        defaultValue="bubble"
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
    );
}
