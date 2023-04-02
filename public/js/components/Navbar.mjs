export default function Navbar() {
  const self = this;

  self.update = () => {
    fetch("/status")
      .then((response) => response.json())
      .then((data) => {
        lemonade.dispatch("State", data);
      });
  };

  self.step = function () {
    fetch("/step", { method: "POST" })
      .then((response) => response.json())
      .then((data) => {
        lemonade.dispatch("State", data);
      });
  };

  return `
    <div class="navbar">
      <div class="navbar__item"><button onclick="self.update()">Refresh</button></div>
      <div class="navbar__item"><button onclick="self.step()">Step</button></div>
    </div>
  `;
}
