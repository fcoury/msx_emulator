import { useEffect } from "react";
import { useStore } from "../Store";

export default function Memory() {
  const memory = useStore((state) => state.memory);
  const status = useStore((state) => state.status);
  const fetchMemory = useStore((state) => state.fetchMemory);

  useEffect(() => {
    fetchMemory();
  }, [status?.pc]);

  let dump = [];
  for (let i = 0; i < memory.length; i += 16) {
    const address = i.toString(16).padStart(4, "0");
    const contents = memory
      .slice(i, i + 16)
      .map((c) => (
        <div className="memory__content">{c.toString(16).padStart(2, "0")}</div>
      ));
    const chars = memory
      .slice(i, i + 16)
      .map((c) => (
        <div className="memory__content">
          {c >= 32 && c <= 126 ? String.fromCharCode(c) : "."}
        </div>
      ));

    dump.push(
      <div className="memory__entry" key={i}>
        <div className="memory__address">{address}</div>
        <div className="memory__contents">{contents}</div>
        <div className="memory__contents">{chars}</div>
      </div>
    );
  }

  return <div className="memory">{dump}</div>;
}
