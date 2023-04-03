import { useStore } from "../Store";

export default function Registers() {
  const status = useStore((state) => state.status);

  if (!status) return null;

  const registers = status.registers.map((r) => (
    <div className="register" key={r.name}>
      <div className="register__name">{r.name.toUpperCase()}</div>
      <div className="register__value">{r.value}</div>
    </div>
  ));

  return <div className="registers">{registers}</div>;
}
