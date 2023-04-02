export default function Memory() {
  const self = this;

  return `
    <div class="memory">
      <div class="memory__entry">
        <div class="memory__address">{{self.address}}</div>
        <div class="memory__contents" @loop={{self.hexcontents}}>
          <div class="memory__content">{{self}}</div>
        </div>
        <div class="memory__contents" @loop={{self.charcontents}}>
          <div class="memory__content">{{self}}</div>
        </div>
      </div>
    </div>
  `;
}
