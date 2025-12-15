import GameClient from "./GameClient";

export function generateStaticParams() {
  return [{ id: 'game' }];
}

export default function GamePage() {
  return <GameClient />;
}
