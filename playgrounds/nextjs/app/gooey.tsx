"use client";
import React, { useRef, useState } from "react";

const Goo = ({
  children,
  className,
  composite = false,
  intensity = "strong",
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
        style={{ pointerEvents: "none", position: "absolute", zIndex: 0, width: 0, height: 0 }}
      >
        <defs>
          <filter
            id={id}
            colorInterpolationFilters="sRGB"
            x="-50%"
            y="-50%"
            width="200%"
            height="200%"
          >
            <feGaussianBlur data-testid="blur" stdDeviation={blur} />
            <feColorMatrix values={`${r} ${g} ${b} ${a}`} />
            {composite && <feComposite data-testid="composite" in="SourceGraphic" />}
          </filter>
        </defs>
      </svg>
      <div className={className} data-testid="element" style={{ ...style, filter: `url(#${id})` }}>
        {children}
      </div>
    </>
  );
};

const Icon = ({ path, className }: { path: string; className?: string }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    className={className}
  >
    <path d={path} />
  </svg>
);

const EditIcon = () => (
  <Icon
    path="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
    className="text-black"
  />
);
const CopyIcon = () => (
  <Icon
    path="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"
    className="text-black"
  />
);
const ShareIcon = () => (
  <Icon
    path="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8 M16 6l-4-4-4 4 M12 2v13"
    className="text-black"
  />
);
const DeleteIcon = () => (
  <Icon
    path="M3 6h18 M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
    className="text-black"
  />
);
const SaveIcon = () => (
  <Icon
    path="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21v-8H7v8 M7 3v5h8"
    className="text-black"
  />
);

const Gooey = () => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isMoving, setIsMoving] = useState(false); // keep visible during motion

  const handleClick = () => {
    setIsMoving(true);
    setIsExpanded((prev) => !prev);
  };

  const menuItems = [
    { icon: <EditIcon />, text: "Edit" },
    { icon: <CopyIcon />, text: "Copy" },
    { icon: <ShareIcon />, text: "Share" },
    { icon: <DeleteIcon />, text: "Delete" },
    { icon: <SaveIcon />, text: "Save" },
  ];

  const OPEN_EASE = "cubic-bezier(0.55, 0.085, 0.68, 0.53)";
  const CLOSE_EASE = "cubic-bezier(0.25, 0.46, 0.45, 0.94)";
  const ease = isExpanded ? OPEN_EASE : CLOSE_EASE;

  const OPEN_MS = 450;
  const CLOSE_MS = 400;
  const transformMs = isExpanded ? OPEN_MS : CLOSE_MS;

  const y = isExpanded ? -160 : -100;
  const move = `translateX(-50%) translateY(${y}px)`;

  const skeletonStyle: React.CSSProperties = {
    transform: move,
    opacity: isExpanded || isMoving ? 1 : 0,
    transformOrigin: "bottom center",
    willChange: "transform, opacity",
    transition: [`transform ${transformMs}ms ${ease}`, "opacity 150ms ease-out"].join(", "),
  };

  const contentStyle: React.CSSProperties = {
    transform: move,
    opacity: isExpanded || isMoving ? 1 : 0,
    pointerEvents: isExpanded ? "auto" : "none",
    transformOrigin: "bottom center",
    willChange: "transform, opacity",
    transition: `transform ${transformMs}ms ${ease}, opacity 150ms ease`,
  };
  
  const onMoveEnd = (e: React.TransitionEvent<HTMLDivElement>) => {
    if (e.propertyName === "transform") {
      setIsMoving(false);
    }
  };

  return (
    <>
      <div className="fixed bottom-0 left-80 w-96 h-96 flex items-center justify-center">
        <Goo id="gooey-menu-filter" className="w-full h-full absolute flex items-center justify-center border">
          <div
            className="top-[-25px] left-1/2 absolute w-64 h-76 rounded-2xl bg-white"
            style={skeletonStyle}
            onTransitionEnd={onMoveEnd}
          />
          <div className="absolute w-24 h-24 rounded-full bg-white" />
        </Goo>

        <div
          className="top-[-25px] left-1/2 absolute w-64 h-80 rounded-2xl p-4 flex flex-col gap-2"
          style={contentStyle}
        >
          {menuItems.map((item, index) => (
            <div
              key={index}
              className="rounded-2xl flex items-center gap-4 p-3 hover:bg-[#e9e9e9] cursor-pointer"
            >
              {item.icon}
              <span className="text-black font-medium">{item.text}</span>
            </div>
          ))}
        </div>

        <div
          onClick={handleClick}
          className="absolute w-24 h-24 rounded-full cursor-pointer transition-transform duration-200 hover:scale-110 active:scale-95 z-10 flex items-center justify-center"
        >
          <div
            className={`transition-transform duration-300 ${isExpanded ? "rotate-45" : "rotate-0"
              }`}
          >
            <Icon path="M12 5v14m-7-7h14" className="text-black" />
          </div>
        </div>
      </div>
    </>
  );
};

export default Gooey;
