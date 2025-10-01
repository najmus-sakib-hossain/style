"use client";

import React, { useState } from "react";

// The Goo component creates a gooey effect for its children using SVG filters.
const Goo = ({
    children,
    className,
    composite = false,
    intensity = "medium",
    id = "gooey-react",
    style,
}: {
    children: React.ReactNode;
    className?: string;
    composite?: boolean;
    intensity?: "weak" | "medium" | "strong";
    id?: string;
    style?: React.CSSProperties;
}) => {
    const blur = intensity === "weak" ? 8 : intensity === "strong" ? 16 : 12;
    const alpha = blur * 6;
    const shift = alpha / -2;
    const r = "1 0 0 0 0";
    const g = "0 1 0 0 0";
    const b = "0 0 1 0 0";
    const a = `0 0 0 ${alpha} ${shift}`;

    return (
        <>
            <svg
                data-testid="svg"
                style={{
                    pointerEvents: "none",
                    position: "absolute",
                    zIndex: 0,
                    width: 0,
                    height: 0,
                }}
            >
                <defs>
                    <filter
                        colorInterpolationFilters="sRGB"
                        data-testid="filter"
                        id={id}
                    >
                        <feGaussianBlur
                            data-testid="blur"
                            stdDeviation={blur}
                        />
                        <feColorMatrix
                            values={`${r} ${g} ${b} ${a}`}
                        />
                        {composite && (
                            <feComposite
                                data-testid="composite"
                                in="SourceGraphic"
                            ></feComposite>
                        )}
                    </filter>
                </defs>
            </svg>
            <div
                className={className}
                data-testid="element"
                style={{
                    ...style,
                    filter: `url(#${id})`,
                }}
            >
                {children}
            </div>
        </>
    );
};

// SVG Icon components for the menu
const Icon = ({ path, className }: { path: string, className?: string }) => (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
        <path d={path} />
    </svg>
);

const EditIcon = () => <Icon path="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" className="text-white"/>;
const CopyIcon = () => <Icon path="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" className="text-white"/>;
const ShareIcon = () => <Icon path="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8 M16 6l-4-4-4 4 M12 2v13" className="text-white"/>;
const DeleteIcon = () => <Icon path="M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" className="text-white"/>;
const SaveIcon = () => <Icon path="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21v-8H7v8 M7 3v5h8" className="text-white"/>;

// This is the main page component that uses the Goo effect and animation.
export default function GooeyPage() {
    const [isExpanded, setIsExpanded] = useState(false);

    const handleClick = () => {
        setIsExpanded(!isExpanded);
    };

    const menuItems = [
        { icon: <EditIcon />, text: "Edit" }, 
        { icon: <CopyIcon />, text: "Copy" }, 
        { icon: <ShareIcon />, text: "Share" },
        { icon: <DeleteIcon />, text: "Delete" }, 
        { icon: <SaveIcon />, text: "Save" }
    ];

    return (
        <main className="flex min-h-screen flex-col items-center justify-center bg-black p-8">
            <div className="text-center mb-12">
                <h1 className="text-4xl font-bold text-white mb-2">Gooey Menu</h1>
                <p className="text-cyan-400">Click the white circle to expand the options!</p>
            </div>
            
            <div className="relative w-96 h-96 flex items-center justify-center">
                <Goo className="w-full h-full relative flex items-center justify-center">
                    
                    {/* The popover-style menu */}
                    <div
                        className="absolute w-64 bg-[#0f0f0f] text-white rounded-2xl p-4 flex flex-col gap-2 transition-all duration-1000"
                        style={{
                            transform: isExpanded ? 'translateY(-180px) scale(1)' : 'translateY(0) scale(0)',
                            opacity: isExpanded ? 1 : 0,
                            transformOrigin: 'bottom center',
                            transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)'
                        }}
                    >
                        {menuItems.map((item, index) => (
                            <div key={index} className="flex items-center gap-4 p-3 rounded-lg hover:bg-indigo-500 cursor-pointer">
                                {item.icon}
                                <span className="text-white font-medium">{item.text}</span>
                            </div>
                        ))}
                    </div>

                    {/* This is the main, clickable circle */}
                    <div
                        onClick={handleClick}
                        className="absolute w-24 h-24 bg-white rounded-full cursor-pointer transition-transform duration-300 hover:scale-110 active:scale-95 z-10 flex items-center justify-center"
                    >
                        <div className={`transition-transform duration-500 ${isExpanded ? 'rotate-45' : 'rotate-0'}`}>
                           <Icon path="M12 5v14m-7-7h14" className="text-black"/>
                        </div>
                    </div>
                </Goo>
            </div>
        </main>
    );
}

