import { useEffect } from "react";
import Hexdump from "../components/Hexdump";
import { useStore } from "../Store";

export default function Memory() {
  const memory = useStore((state) => state.memory);
  const status = useStore((state) => state.status);
  // const memoryLoading = useStore((state) => state.memoryLoading);
  const memoryError = useStore((state) => state.memoryError);
  const fetchMemory = useStore((state) => state.fetchMemory);

  useEffect(() => {
    fetchMemory();
  }, [status?.pc]);

  // if (memoryLoading) return null;
  if (memoryError) return <div className="error">{memoryError}</div>;

  return (
    <div className="memory">
      <Hexdump data={memory} columns={8} />
    </div>
  );
}
