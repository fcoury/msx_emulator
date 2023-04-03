import { useEffect, useMemo, useRef, useState } from "react";
import { calculateMemoryHash } from "../Utils";

export default function Hexdump({ data, columns = 16 }) {
  const [prevData, setPrevData] = useState(null);
  const hash = calculateMemoryHash(data);
  const firstChangedRef = useRef(null);

  useEffect(() => {
    if (prevData) {
      if (firstChangedRef.current) {
        firstChangedRef.current.scrollIntoView({ behavior: "smooth" });
      }
    }
    setPrevData(data);
  }, [hash]);

  const dump = useMemo(() => {
    const dump = [];
    let firstChangedSet = false;

    for (let i = 0; i < data.length; i += columns) {
      const address = i.toString(16).padStart(4, "0");
      const contents = Array.from(data.slice(i, i + columns)).map((c, idx) => {
        const changed =
          prevData &&
          prevData[i + idx] !== undefined &&
          prevData[i + idx] !== c;
        if (changed && !firstChangedSet) {
          firstChangedSet = true;
          return (
            <div
              ref={firstChangedRef}
              className="hexdump__content hexdump__content--changed"
              key={`${i}_${idx}`}
            >
              {c.toString(16).padStart(2, "0")}
            </div>
          );
        } else {
          return (
            <div
              className={`hexdump__content${
                changed ? " hexdump__content--changed" : ""
              }`}
              key={`${i}_${idx}`}
            >
              {c.toString(16).padStart(2, "0")}
            </div>
          );
        }
      });
      const chars = Array.from(data.slice(i, i + columns)).map((c, idx) => (
        <div className="hexdump__content" key={`${c}_${idx}`}>
          {c >= 32 && c <= 126 ? String.fromCharCode(c) : "."}
        </div>
      ));

      dump.push(
        <div className="hexdump__entry" key={address}>
          <div className="hexdump__address">{address}</div>
          <div className="hexdump__contents">{contents}</div>
          <div className="hexdump__contents">{chars}</div>
        </div>
      );
    }

    return dump;
  }, [hash, prevData]);

  return <div className="hexdump">{dump}</div>;
}
