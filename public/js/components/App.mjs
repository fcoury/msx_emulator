export default function App() {
  const self = this;
  self.status = "Loading...";

  return `
    <div class="container">
      <Navbar />
      <div class="main">
        <Opcode />
        <div class="status">
          <Registers />
          <div class="split">
            <Memory />
            <Video />
          </div>
        </div>
      </div>
    </div>
  `;
}
