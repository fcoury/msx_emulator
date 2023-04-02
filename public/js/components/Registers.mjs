export default function Registers() {
  const self = this;

  lemonade.set("Registers", self);

  return `
    <div class="registers">
      <div class="register" @list="self.registers">
        <div class="register__name">{{self.name}}</div>
        <div class="register__value">{{self.value}}</div>
      </div>
    </div>
  `;
}
