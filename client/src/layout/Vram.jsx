import { useEffect } from "react";
import Hexdump from "../components/Hexdump";
import { useStore } from "../Store";

export default function Vram() {
  const vram = useStore((state) => state.vram);
  const status = useStore((state) => state.status);
  const fetchVram = useStore((state) => state.fetchVram);

  // useEffect(() => {
  //   fetchVram();
  // }, [status?.pc]);

  return (
    <div className="vram">
      <Hexdump data={vram} columns={8} />
    </div>
  );
}
