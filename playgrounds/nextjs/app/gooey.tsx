import Goo from "gooey-react";

export default function GooeyPage() {
  return (
    <Goo className="h-48 relative w-48">
      <div className="bg-rose-500 rounded-full h-16 w-16 left-10 top-0 absolute" />
      <div className="bg-pink-500 rounded-full h-16 w-16 left-22 top-10 absolute" />
    </Goo>
  );
}
