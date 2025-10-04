import gsap from "gsap";
import React from "react";
import { Config } from "./liquid-glass-configs";

export const GlassFilter: React.FC = () => {
    return (
        <svg className="filter" xmlns="http://www.w3.org/2000/svg">
            <title>Glass effect SVG filter</title>
            <defs>
                <filter id="liquid-glass-filter" colorInterpolationFilters="sRGB">
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
    );
};

export const updateDisplacementFilter = (config: Config) => {
    const buildDisplacementImage = () => {
        const border =
            Math.min(config.width, config.height) * (config.border * 0.5);
        const svgString = `
      <svg class="displacement-image" viewBox="0 0 ${config.width} ${config.height}" xmlns="http://www.w3.org/2000/svg">
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
    `;
        const encoded = encodeURIComponent(svgString);
        const dataUri = `data:image/svg+xml,${encoded}`;
        gsap.set("#liquid-glass-filter feImage", { attr: { href: dataUri } });
        gsap.set("#liquid-glass-filter feDisplacementMap", {
            attr: { xChannelSelector: config.x, yChannelSelector: config.y },
        });
    };

    buildDisplacementImage();
    gsap.set("#liquid-glass-filter feDisplacementMap", { attr: { scale: config.scale } });
    gsap.set("#redchannel", { attr: { scale: config.scale + config.r } });
    gsap.set("#greenchannel", {
        attr: { scale: config.scale + config.g },
    });
    gsap.set("#bluechannel", { attr: { scale: config.scale + config.b } });
    gsap.set("#liquid-glass-filter feGaussianBlur", {
        attr: { stdDeviation: config.displace },
    });
};