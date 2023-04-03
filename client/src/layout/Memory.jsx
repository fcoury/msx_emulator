import { useEffect } from "react";
import Hexdump from "../components/Hexdump";
import { useStore } from "../Store";

export default function Memory() {
  const memory = useStore((state) => state.memory);
  const status = useStore((state) => state.status);
  const fetchMemory = useStore((state) => state.fetchMemory);

  useEffect(() => {
    fetchMemory();
  }, [status?.pc]);

  return (
    <div className="memory">
      <Hexdump data={memory} columns={8} />
    </div>
  );
}
