import { useEffect } from "react";
import { useStore } from "../Store.js";

export default function Program() {
  const fetchProgram = useStore((state) => state.fetchProgram);
  const program = useStore((state) => state.program);
  const status = useStore((state) => state.status);
  const addresses = program.map((c) => c.address);

  useEffect(() => {
    fetchProgram();
  }, [addresses && addresses.includes(status?.pc)]);

  useEffect(() => {
    const el = document.querySelector(".opcode.selected");
    if (el) {
      el.scrollIntoViewIfNeeded();
    }
  }, [status?.pc, program]);

  if (!program) return null;
  if (!status) return null;

  const opcodes = program.map((c) => {
    const classNames = ["opcode"];
    if (c.address === status.pc) classNames.push("selected");
    return (
      <div className={classNames.join(" ")} key={c.address}>
        <div className="opcode__column opcode__address">{c.address}</div>
        <div className="opcode__column opcode__hex">{c.hexcontents}</div>
        <div className="opcode__column opcode__instruction">
          {c.instruction}
        </div>
      </div>
    );
  });

  return <div className="opcodes">{opcodes}</div>;
}
