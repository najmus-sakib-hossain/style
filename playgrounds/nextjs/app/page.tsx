export default function Home() {
  return (
    <div className="border h-10 w-10 rounded-full fixed top-1/2 left-1/2 translate-x-[-50%] translate-y-[-50%]">
      <div className="absolute top-0 left-0 h-full w-1/2 bg-background rounded-tl-full rounded-bl-full" />
      <div className="z-50 border h-4 w-4 rounded-full fixed top-1/2 left-1/2 translate-x-[-50%] translate-y-[-50%]">
        <div className="absolute top-0 left-0 h-full w-1/2 bg-foreground rounded-tl-full rounded-bl-full" />
        <div className="absolute top-0 right-0 h-full w-1/2 bg-background rounded-tr-full rounded-br-full" />
      </div>
      <div className="absolute top-0 right-0 h-full w-1/2 bg-foreground rounded-tr-full rounded-br-full" />
    </div>
  );
}
