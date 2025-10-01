/* @layer liquid-glass {
    @import url("https://unpkg.com/normalize.css") layer(normalize);

    @layer normalize, base, demo;

    @layer demo {
        :root {
            --content-width: 720px;
            scrollbar-color: canvasText #0000;
        }

        section p {
            line-height: 1.5;
        }

        .emojis {
            --font-level: 4;
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }

        .arrow {
            display: inline-block;
            opacity: 0.8;
            position: absolute;
            font-size: 0.875rem;
            font-family: "Gloria Hallelujah", cursive;
            transition: opacity 0.26s ease-out;
        }

        .arrow.arrow--debug {
            top: 140px;
            left: 30%;
            transform: translate(-100%, 0);
            width: 80px;
        }

        .arrow.arrow--debug span {
            display: inline-block;
            transform: rotate(-24deg) translateY(100%);
        }

        .arrow.arrow--debug svg {
            transform: translate(80%, -80%) rotate(-25deg);
            left: 0%;
            width: 100%;
        }

        .filter {
            width: 100%;
            height: 100%;
            pointer-events: none;
            position: absolute;
            inset: 0;
        }

        header,
        main {
            width: var(--content-width);
            max-width: calc(100vw - 2rem);
            margin: 0 auto;
        }

        section {
            margin-block: 4rem;
        }

        .images {
            display: flex;
            flex-direction: column;
            gap: 2rem;
        }

        .images img {
            width: 300px;
        }

        footer {
            padding: 1rem;
            text-align: center;
            font-size: 0.875rem;
            opacity: 0.7;
        }

        header {
            margin-block: 4rem;
        }

        header p {
            --font-level: 2;
            text-wrap: balance;
            color: color-mix(in oklch, canvasText, canvas 35%);
        }

        main {
            flex: 1;
        }

        main img {
            border-radius: 12px;
        }

        .apps {
            display: grid;
            grid-template-columns: repeat(4, 80px);
            gap: 1rem;
        }

        .app {
            width: 80px;
            font-size: 0.875rem;
            font-weight: 300;
        }

        .app span {
            display: block;
            text-align: center;
            white-space: nowrap;
        }

        .app img {
            width: 100%;
        }

        .nav-wrap {
            width: 100%;
            height: 100%;
            overflow: hidden;
            border-radius: inherit;
        }

        [data-icons="true"] .effect nav {
            opacity: 1;
        }

        [data-mode="dock"] .effect {
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
        }

        .effect nav img {
            width: 80px;
            aspect-ratio: 1;
        }

        .effect {
            opacity: 0;
            transition: opacity 0.26s ease-out;
            height: calc(var(--height) * 1px);
            width: calc(var(--width) * 1px);
            border-radius: calc(var(--radius) * 1px);
            position: fixed;
            background: light-dark(hsl(var(--hue) 50% 70% / 0.2),
                    hsl(var(--hue) 50% 30% / 0.2));
            backdrop-filter: url(#filter) saturate(var(--saturation, 1));
        }
        
        .effect * {
            pointer-events: none;
        }
        
        .effect-border {
             position: fixed;
             opacity: 0;
             transition: opacity 0.26s ease-out;
        }

        .placeholder {
            width: 336px;
            height: 96px;
            max-width: 100%;
            position: relative;
            margin-bottom: 200px;
        }

        .dock-placeholder {
            width: 336px;
            height: 96px;
            border-radius: 16px;
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }

        [data-debug="true"] .displacement-debug {
            transform: translateY(calc(100% + 1rem));
            scale: 1;
            opacity: 1;
        }

        .displacement-debug {
            pointer-events: none;
            height: 100%;
            width: 100%;
            position: absolute;
            inset: 0;
            transform: translateY(calc(200% + 1rem));
            scale: 0.8;
            opacity: 0;
            transition-property: transform, opacity, scale;
            transition-duration: 0.26s;
            transition-timing-function: ease-out;
        }

        .displacement-debug .label {
            position: absolute;
            left: 50%;
            top: calc(100% + 0.2lh);
        }

        .displacement-debug .label span {
            display: inline-block;
            font-size: 0.875rem;
            font-family: "Gloria Hallelujah", cursive;
            padding: 0.5rem 0.75rem;
            background: color-mix(in oklch, canvas, #0000 25%);
            backdrop-filter: blur(4px);
            border-radius: 6px;
            white-space: nowrap;
        }

        .displacement-debug .label svg {
            position: absolute;
            filter: drop-shadow(0 2px 10px canvas);
            right: 100%;
            transform: translate(25%, 60%) rotate(40deg) scale(-1, 1);
            width: 40px;
        }

        .displacement-debug .displacement-image {
            height: 100%;
            width: 100%;
            pointer-events: none;
            border-radius: calc(var(--radius) * 1px);
        }

        h1 {
            --font-level: 6;
            line-height: 0.9;
            margin: 0;
            margin-bottom: 0.25lh;
        }
    }

    @layer base {
        :root {
            --font-size-min: 16;
            --font-size-max: 20;
            --font-ratio-min: 1.2;
            --font-ratio-max: 1.33;
            --font-width-min: 375;
            --font-width-max: 1500;
        }

        html {
            color-scheme: light dark;
        }

        [data-theme="light"] {
            color-scheme: light only;
        }

        [data-theme="dark"] {
            color-scheme: dark only;
        }

        .fluid {
            --fluid-min: calc(var(--font-size-min) * pow(var(--font-ratio-min), var(--font-level, 0)));
            --fluid-max: calc(var(--font-size-max) * pow(var(--font-ratio-max), var(--font-level, 0)));
            --fluid-preferred: calc((var(--fluid-max) - var(--fluid-min)) / (var(--font-width-max) - var(--font-width-min)));
            --fluid-type: clamp(calc(var(--fluid-min) / 16 * 1rem),
                    calc((var(--fluid-min) / 16 * 1rem) - (var(--fluid-preferred) * var(--font-width-min) / 16 * 1rem)) + (var(--fluid-preferred) * 100vi),
                    calc(var(--fluid-max) / 16 * 1rem));
            font-size: var(--fluid-type);
        }

        *,
        *:after,
        *:before {
            box-sizing: border-box;
        }

        body {
            background: light-dark(#fff, #000);
            overflow-x: hidden;
            min-height: 100vh;
            font-family:
                "SF Pro Text", "SF Pro Icons", "AOS Icons", "Helvetica Neue",
                Helvetica, Arial, sans-serif, system-ui;
        }

        body::before {
            --size: 45px;
            --line: color-mix(in hsl, canvasText, transparent 80%);
            content: "";
            height: 100vh;
            width: 100vw;
            position: fixed;
            background:
                linear-gradient(90deg,
                    var(--line) 1px,
                    transparent 1px var(--size)) calc(var(--size) * 0.36) 50% / var(--size) var(--size),
                linear-gradient(var(--line) 1px, transparent 1px var(--size)) 0% calc(var(--size) * 0.32) / var(--size) var(--size);
            mask: linear-gradient(-20deg, transparent 50%, white);
            top: 0;
            transform-style: flat;
            pointer-events: none;
        }

        .bear-link {
            color: canvasText;
            position: fixed;
            top: 1rem;
            left: 1rem;
            width: 48px;
            aspect-ratio: 1;
            display: grid;
            place-items: center;
            opacity: 0.8;
        }

        .x-link:hover,
        .x-link:focus-visible,
        .bear-link:hover,
        .bear-link:focus-visible {
            opacity: 1;
        }

        .bear-link svg {
            width: 75%;
        }

        .sr-only {
            position: absolute;
            width: 1px;
            height: 1px;
            padding: 0;
            margin: -1px;
            overflow: hidden;
            clip: rect(0, 0, 0, 0);
            white-space: nowrap;
            border-width: 0;
        }
    }
} */
