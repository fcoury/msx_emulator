export default function Hexdump({ data, columns = 16 }) {
  let dump = [];
  for (let i = 0; i < data.length; i += columns) {
    const address = i.toString(16).padStart(4, "0");
    const contents = data
      .slice(i, i + columns)
      .map((c) => (
        <div className="hexdump__content">
          {c.toString(16).padStart(2, "0")}
        </div>
      ));
    const chars = data
      .slice(i, i + columns)
      .map((c) => (
        <div className="hexdump__content">
          {c >= 32 && c <= 126 ? String.fromCharCode(c) : "."}
        </div>
      ));

    dump.push(
      <div className="hexdump__entry" key={i}>
        <div className="hexdump__address">{address}</div>
        <div className="hexdump__contents">{contents}</div>
        <div className="hexdump__contents">{chars}</div>
      </div>
    );
  }

  return <div className="hexdump">{dump}</div>;
}
