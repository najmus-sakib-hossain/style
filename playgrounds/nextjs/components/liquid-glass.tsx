'use client';

import { useEffect, useRef, useState } from 'react';
import { Pane } from 'tweakpane';
import gsap from 'gsap';
import Draggable from 'gsap/Draggable';

gsap.registerPlugin(Draggable);

const base = {
  icons: false,
  scale: -180,
  radius: 16,
  border: 0.07,
  lightness: 50,
  displace: 0,
  blend: 'difference',
  x: 'R',
  y: 'B',
  alpha: 0.93,
  blur: 11,
  r: 0,
  g: 10,
  b: 20,
  saturation: 1,
};

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

const initialConfig = {
  ...presets.dock,
  theme: 'system',
  debug: false,
  top: false,
  preset: 'dock',
};

export default function LiquidGlass() {
  const [displacementImage, setDisplacementImage] = useState('');
  const effectRef = useRef<HTMLDivElement>(null);
  const debugPenRef = useRef<HTMLDivElement>(null);
  const paneRef = useRef<Pane | null>(null);
  const configRef = useRef(initialConfig);
  const [, forceUpdate] = useState(0);

  const buildDisplacementImage = () => {
    const config = configRef.current;
    const border = Math.min(config.width, config.height) * (config.border * 0.5);
    const svgContent = `
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
        <rect x="${border}" y="${Math.min(config.width, config.height) * (config.border * 0.5)}" width="${config.width - border * 2}" height="${config.height - border * 2}" rx="${config.radius}" fill="hsl(0 0% ${config.lightness}% / ${config.alpha})" style="filter:blur(${config.blur}px)" />
      </svg>
      <div class="label">
        <span>displacement image</span>
        <svg viewBox="0 0 97 52" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path fill-rule="evenodd" clip-rule="evenodd" d="M74.568 0.553803C74.0753 0.881909 73.6295 1.4678 73.3713 2.12401C73.1367 2.70991 72.3858 4.67856 71.6584 6.50658C70.9544 8.35803 69.4526 11.8031 68.3498 14.1936C66.1441 19.0214 65.839 20.2167 66.543 21.576C67.4581 23.3337 69.4527 23.9196 71.3064 22.9821C72.4797 22.3728 74.8965 19.5839 76.9615 16.4435C78.8387 13.5843 78.8387 13.6077 78.1113 18.3418C77.3369 23.4275 76.4687 26.2866 74.5915 30.0364C73.254 32.7316 71.8461 34.6299 69.218 37.3485C65.9563 40.6999 62.2254 42.9732 57.4385 44.4965C53.8718 45.6449 52.3935 45.8324 47.2546 45.8324C43.3594 45.8324 42.1158 45.7386 39.9805 45.2933C32.2604 43.7466 25.3382 40.9577 19.4015 36.9735C15.0839 34.0909 12.5028 31.7004 9.80427 27.9975C6.80073 23.9196 4.36038 17.2403 3.72682 11.475C3.37485 8.1471 3.1402 7.32683 2.43624 7.13934C0.770217 6.71749 0.183578 7.77211 0.0193217 11.5219C-0.26226 18.5996 2.55356 27.1304 7.17619 33.1066C13.8403 41.7545 25.432 48.4103 38.901 51.2696C41.6465 51.8555 42.2566 51.9023 47.4893 51.9023C52.3935 51.9023 53.426 51.832 55.5144 51.3867C62.2723 49.9337 68.5375 46.6292 72.949 42.1998C76.0464 39.1296 78.1113 36.2939 79.8946 32.7081C82.1942 28.0912 83.5317 23.3103 84.2591 17.17C84.3999 15.8576 84.6111 14.7795 84.7284 14.7795C84.8223 14.7795 85.4559 15.1311 86.1364 15.5763C88.037 16.7716 90.3835 17.8965 93.5748 19.0918C96.813 20.3339 97.3996 20.287 96.4141 18.9512C94.9123 16.9122 90.055 11.5219 87.1219 8.63926C84.0949 5.66288 83.8368 5.33477 83.5552 4.1864C83.3909 3.48332 83.0155 2.68649 82.6401 2.31151C82.0065 1.6553 80.4109 1.04595 79.9885 1.30375C79.8712 1.37406 79.2845 1.11626 78.6744 0.717845C77.2431 -0.172727 75.7413 -0.243024 74.568 0.553803Z" fill="currentColor"></path>
        </svg>
      </div>
    `;
    setDisplacementImage(svgContent);

    const svgEl = new DOMParser().parseFromString(svgContent, 'image/svg+xml').querySelector('.displacement-image');
    if (svgEl) {
      const serialized = new XMLSerializer().serializeToString(svgEl);
      const encoded = encodeURIComponent(serialized);
      const dataUri = `data:image/svg+xml,${encoded}`;

      gsap.set('feImage', {
        attr: {
          href: dataUri,
        },
      });
    }
  };

  const update = () => {
    const config = configRef.current;
    buildDisplacementImage();
    gsap.set(document.documentElement, {
      '--width': config.width,
      '--height': config.height,
      '--radius': config.radius,
      '--frost': config.frost,
      '--output-blur': config.displace,
      '--saturation': config.saturation,
    });
    gsap.set('feDisplacementMap', {
      attr: {
        scale: config.scale,
      },
    });
    gsap.set('#redchannel', {
      attr: {
        scale: config.scale + config.r,
      },
    });
    gsap.set('#greenchannel', {
      attr: {
        scale: config.scale + config.g,
      },
    });
    gsap.set('#bluechannel', {
      attr: {
        scale: config.scale + config.b,
      },
    });
    gsap.set('feGaussianBlur', {
      attr: {
        stdDeviation: config.displace,
      },
    });

    document.documentElement.dataset.icons = config.icons.toString();
    document.documentElement.dataset.mode = config.preset;
    document.documentElement.dataset.top = config.top.toString();
    document.documentElement.dataset.debug = config.debug.toString();
    document.documentElement.dataset.theme = config.theme;
  };

  useEffect(() => {
    // @ts-ignore
    const ctrl = new Pane({
      title: 'config',
      expanded: true,
    });
    paneRef.current = ctrl;

    // @ts-ignore
    const settings = ctrl.addFolder({
      title: 'settings',
      disabled: true,
      expanded: false,
    });

    // @ts-ignore
    ctrl.addBinding(configRef.current, 'debug');
    // @ts-ignore
    ctrl.addBinding(configRef.current, 'top');
    // @ts-ignore
    ctrl
      // @ts-ignore
      .addBinding(configRef.current, 'preset', {
        label: 'mode',
        options: {
          dock: 'dock',
          pill: 'pill',
          bubble: 'bubble',
          free: 'free',
        },
      })
      .on('change', () => {
        const newPreset = configRef.current.preset;
        document.documentElement.dataset.mode = newPreset;
        settings.expanded = newPreset === 'free';
        settings.disabled = newPreset !== 'free';
        if (newPreset !== 'free') {
          configRef.current = { ...presets[newPreset as keyof typeof presets], theme: configRef.current.theme, debug: configRef.current.debug, top: configRef.current.top, preset: newPreset } as typeof initialConfig;
        }
        forceUpdate(Math.random());
      });
    // @ts-ignore
    ctrl.addBinding(configRef.current, 'theme', {
      label: 'theme',
      options: {
        system: 'system',
        light: 'light',
        dark: 'dark',
      },
    });

    // @ts-ignore
    settings.addBinding(configRef.current, 'frost', {
      label: 'frost',
      min: 0,
      max: 1,
      step: 0.01,
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'saturation', {
      min: 0,
      max: 2,
      step: 0.1,
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'icons');
    // @ts-ignore
    settings.addBinding(configRef.current, 'width', {
      min: 80,
      max: 500,
      step: 1,
      label: 'width (px)',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'height', {
      min: 35,
      max: 500,
      step: 1,
      label: 'height (px)',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'radius', {
      min: 0,
      max: 500,
      step: 1,
      label: 'radius (px)',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'border', {
      min: 0,
      max: 1,
      step: 0.01,
      label: 'border',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'alpha', {
      min: 0,
      max: 1,
      step: 0.01,
      label: 'alpha',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'lightness', {
      min: 0,
      max: 100,
      step: 1,
      label: 'lightness',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'blur', {
      min: 0,
      max: 20,
      step: 1,
      label: 'input blur',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'displace', {
      min: 0,
      max: 12,
      step: 0.1,
      label: 'output blur',
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'x', {
      label: 'channel x',
      options: {
        r: 'R',
        g: 'G',
        b: 'B',
      },
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'y', {
      label: 'channel y',
      options: {
        r: 'R',
        g: 'G',
        b: 'B',
      },
    });
    // @ts-ignore
    settings.addBinding(configRef.current, 'blend', {
      options: {
        normal: 'normal',
        multiply: 'multiply',
        screen: 'screen',
        overlay: 'overlay',
        darken: 'darken',
        lighten: 'lighten',
        'color-dodge': 'color-dodge',
        'color-burn': 'color-burn',
        'hard-light': 'hard-light',
        'soft-light': 'soft-light',
        difference: 'difference',
        exclusion: 'exclusion',
        hue: 'hue',
        saturation: 'saturation',
        color: 'color',
        luminosity: 'luminosity',
        'plus-darker': 'plus-darker',
        'plus-lighter': 'plus-lighter',
      },
      label: 'blend',
    });

    // @ts-ignore
    settings.addBinding(configRef.current, 'scale', {
      min: -1000,
      max: 1000,
      step: 1,
      label: 'scale',
    });

    // @ts-ignore
    const abb = settings.addFolder({ title: 'chromatic' });
    // @ts-ignore
    abb.addBinding(configRef.current, 'r', {
      min: -100,
      max: 100,
      step: 1,
      label: 'red',
    });
    // @ts-ignore
    abb.addBinding(configRef.current, 'g', {
      min: -100,
      max: 100,
      step: 1,
      label: 'green',
    });
    // @ts-ignore
    abb.addBinding(configRef.current, 'b', {
      min: -100,
      max: 100,
      step: 1,
      label: 'blue',
    });

    // @ts-ignore
    ctrl.on('change', () => {
      update();
      forceUpdate(Math.random());
    });

    update();

    // Draggable
    if (effectRef.current) {
      Draggable.create(effectRef.current, {
        type: 'x,y',
      });
    }

    return () => {
      ctrl.dispose();
    };
  }, []);

  useEffect(() => {
    // Position the effect initially
    if (effectRef.current) {
      const placeholder = document.querySelector('.dock-placeholder') as HTMLElement;
      if (placeholder) {
        const { top, left } = placeholder.getBoundingClientRect();
        gsap.set(effectRef.current, {
          top: top > window.innerHeight ? window.innerHeight * 0.5 : top,
          left,
          opacity: 1,
        });
      }
    }
  }, []);

  return (
    <>
      <svg className="filter" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <filter id="filter" colorInterpolationFilters="sRGB">
            <feImage x="0" y="0" width="100%" height="100%" result="map"></feImage>
            <feDisplacementMap in="SourceGraphic" in2="map" id="redchannel" xChannelSelector="R" yChannelSelector="G" result="dispRed" />
            <feColorMatrix in="dispRed" type="matrix" values="1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0" result="red" />
            <feDisplacementMap in="SourceGraphic" in2="map" id="greenchannel" xChannelSelector="R" yChannelSelector="G" result="dispGreen" />
            <feColorMatrix in="dispGreen" type="matrix" values="0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 0" result="green" />
            <feDisplacementMap in="SourceGraphic" in2="map" id="bluechannel" xChannelSelector="R" yChannelSelector="G" result="dispBlue" />
            <feColorMatrix in="dispBlue" type="matrix" values="0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 1 0" result="blue" />
            <feBlend in="red" in2="green" mode="screen" result="rg" />
            <feBlend in="rg" in2="blue" mode="screen" result="output" />
            <feGaussianBlur in="output" stdDeviation="0.7" />
          </filter>
        </defs>
      </svg>
      <div className="effect" ref={effectRef}>
        <div className="nav-wrap">
          <nav>
            <img src="https://assets.codepen.io/605876/finder.png" alt="" />
            <img src="https://assets.codepen.io/605876/launch-control.png" alt="" />
            <img src="https://assets.codepen.io/605876/safari.png" alt="" />
            <img src="https://assets.codepen.io/605876/calendar.png" alt="" />
          </nav>
        </div>
        <div className="displacement-debug" ref={debugPenRef} dangerouslySetInnerHTML={{ __html: displacementImage }}></div>
      </div>
      <style jsx>{`
        .filter {
          width: 100%;
          height: 100%;
          pointer-events: none;
          position: absolute;
          inset: 0;
        }

        .effect {
          opacity: 0;
          transition: opacity 0.26s ease-out;
          height: calc(var(--height) * 1px);
          width: calc(var(--width) * 1px);
          border-radius: calc(var(--radius) * 1px);
          position: fixed;
          z-index: 999999;
          background: light-dark(
            hsl(0 0% 100% / var(--frost, 0)),
            hsl(0 0% 0% / var(--frost, 0))
          );
          backdrop-filter: url(#filter) saturate(var(--saturation, 1));
          box-shadow: 0 0 2px 1px
              light-dark(
                color-mix(in oklch, canvasText, #0000 85%),
                color-mix(in oklch, canvasText, #0000 65%)
              )
              inset,
            0 0 10px 4px
              light-dark(
                color-mix(in oklch, canvasText, #0000 90%),
                color-mix(in oklch, canvasText, #0000 85%)
              )
              inset,
            0px 4px 16px rgba(17, 17, 26, 0.05), 0px 8px 24px rgba(17, 17, 26, 0.05),
            0px 16px 56px rgba(17, 17, 26, 0.05),
            0px 4px 16px rgba(17, 17, 26, 0.05) inset,
            0px 8px 24px rgba(17, 17, 26, 0.05) inset,
            0px 16px 56px rgba(17, 17, 26, 0.05) inset;
          * {
            pointer-events: none;
          }
        }

        [data-icons='true'] .effect nav {
          opacity: 1;
        }

        [data-mode='dock'] .effect {
          backdrop-filter: url(#filter) brightness(1.1) saturate(1.5);
        }

        .effect nav {
          width: 100%;
          height: 100%;
          flex-wrap: wrap;
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 0.4rem;
          opacity: 0;
          overflow: hidden;
          border-radius: inherit;
          transition: opacity 0.26s ease-out;
          img {
            width: 80px;
            aspect-ratio: 1;
          }
        }

        .nav-wrap {
          width: 100%;
          height: 100%;
          overflow: hidden;
          border-radius: inherit;
        }

        [data-debug='true'] .displacement-debug {
          translate: 0 calc(100% + 1rem);
          scale: 1;
          opacity: 1;
        }

        .displacement-debug {
          pointer-events: none;
          height: 100%;
          width: 100%;
          position: absolute;
          inset: 0;
          translate: 0 calc(200% + 1rem);
          scale: 0.8;
          opacity: 0;
          transition-property: translate, opacity, scale;
          transition-duration: 0.26s;
          transition-timing-function: ease-out;
          z-index: -1;
          .label {
            position: absolute;
            left: 50%;
            top: calc(100% + 0.2lh);
            span {
              display: inline-block;
              font-size: 0.875rem;
              font-family: 'Gloria Hallelujah', cursive;
              padding: 0.5rem 0.75rem;
              background: color-mix(in oklch, canvas, #0000 25%);
              backdrop-filter: blur(4px);
              border-radius: 6px;
              white-space: nowrap;
            }
            svg {
              position: absolute;
              filter: drop-shadow(0 2px 10px canvas);
              right: 100%;
              rotate: 40deg;
              translate: 25% 60%;
              scale: -1 1;
              width: 40px;
            }
          }
          .displacement-image {
            height: 100%;
            width: 100%;
            pointer-events: none;
            border-radius: calc(var(--radius) * 1px);
          }
        }
      `}</style>
    </>
  );
}