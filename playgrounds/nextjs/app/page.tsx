import SwitcherIcon from "@/components/switcher-icon";

export default function Home() {
  return (
    <div className="h-screen w-full flex justify-center items-center space-x-8 p-4">
      <div className="flex flex-col items-center">
        <SwitcherIcon />
        <span className="text-sm mt-2 text-center">Light</span>
      </div>
      <div className="flex flex-col items-center">
        <SwitcherIcon />
        <span className="text-sm mt-2 text-center">Dark</span>
      </div>
      <div className="flex flex-col items-center">
        <SwitcherIcon />
        <span className="text-sm mt-2 text-center">Tinted
          {/*  - Color + Linear + Radial + Conical Gradient */}
        </span>
      </div>
      <div className="flex flex-col items-center">
        <SwitcherIcon />
        <span className="text-sm mt-2 text-center">Clear
          {/*  - Image + Mesh + Pattern */}
        </span>
      </div>
    </div>
  );
}
