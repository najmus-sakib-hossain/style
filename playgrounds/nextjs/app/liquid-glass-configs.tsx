export type Config = {
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

export const base: Config = {
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

export const presets = {
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
