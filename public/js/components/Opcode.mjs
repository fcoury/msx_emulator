export default function Opcode() {
  const self = this;

  lemonade.set("Program", function (s) {
    console.log("Program", s);
    self.entries = s;
  });

  return `
    <div class="opcodes" @loop="self.entries">
      <div class="opcode">
        <div class="opcode__column opcode__address">{{self.address}}</div>
        <div class="opcode__column opcode__hex">{{self.hexcontents}}</div>
        <div class="opcode__column opcode__instruction">{{self.instruction}}</div>
      </div>
    </div>
  `;
}
