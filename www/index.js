import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg.wasm";

const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

// Construct the universe, and get its width and height.

const universe = Universe.new();
const width = universe.width();
const height = universe.height()

// Give the canvas room for all of our cells and a 1px border
// around each of them.

const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;
const ctx = canvas.getContext("2d");

// Grid/Cells

const drawGrid = () => {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines.
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }

  // Horizontal lines.
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
};

let cells = null;

const forceDrawCells = () => {
  ctx.beginPath();
  const cellsPtr = universe.cells();
  cells = cells == null ? new Uint8Array(memory.buffer, cellsPtr, width * height) : cells;
  for (let row = 0; row < height; row++) {
    for (let col = 0; col < width; col++) {
      const idx = getIndex(row, col);
      ctx.fillStyle = cells[idx] === Cell.Dead ? DEAD_COLOR : ALIVE_COLOR;
      ctx.fillRect(col * (CELL_SIZE + 1) + 1, row * (CELL_SIZE + 1) + 1, CELL_SIZE, CELL_SIZE);
    }
  }
  ctx.stroke();
};

const drawCells = () => {
  if (!cells) {
    forceDrawCells();
    return;
  }
  ctx.beginPath();
  const diffPtr = universe.diff();
  const diff = new Int32Array(memory.buffer, diffPtr, width * height);
  for (let i = 0; i <= diff.length; i++) {
    const idx = diff[i];
    if (idx == -1) return;
    const row = Math.floor(idx / width);
    const col = idx % width;
    ctx.fillStyle = cells[idx] === Cell.Dead ? DEAD_COLOR : ALIVE_COLOR;
    ctx.fillRect(col * (CELL_SIZE + 1) + 1, row * (CELL_SIZE + 1) + 1, CELL_SIZE, CELL_SIZE);
    ctx.stroke();
  }
};

// Inserting patterns

canvas.addEventListener("click", (event) => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  if (event.ctrlKey) {
    universe.insert_glider(row, col);
  } else if (event.shiftKey) {
    universe.insert_pulsar(row, col);
  } else {
    universe.toggle_cell(row, col);
  }

  forceDrawCells();
});

// FPS counter

const fps = new (class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimeStamp = performance.now();
  }

  render() {
    // Convert the delta time since the last frame render into a measure
    // of frames per second.
    const now = performance.now();
    const delta = now - this.lastFrameTimeStamp;
    this.lastFrameTimeStamp = now;
    const fps = (1 / delta) * 1000;

    // Save only the latest 100 timings.
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }

    // Find the max, min, and mean of our 100 latest timings.
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;
    let framesSorted = this.frames.slice().sort((a, b) => a - b);
    let median = framesSorted[Math.floor(this.frames.length / 2)];

    // Render the statistics.
    this.fps.textContent =
      `FPS: ${Math.round(fps)}, (last 100: min ${Math.round(min)}, max ${Math.round(max)}, mean ${Math.round(mean)}, median ${Math.round(median)})`.trim();
  }
})();

// The animation

let animationId = null;
let timer = 0;
let speed = 0.2;

const renderLoop = () => {
  fps.render();

  timer += speed;
  if (timer >= 1) {
    universe.tick();
    drawCells();
    timer = 0;
  }

  animationId = requestAnimationFrame(renderLoop);
};

// Play/pause

const isPaused = () => {
  return animationId === null;
};

const playPauseButton = document.getElementById("play-pause");

const play = () => {
  playPauseButton.textContent = "⏸";
  renderLoop();
};

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
};

playPauseButton.addEventListener("click", (_) => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

// Randomize

const randomizeButton = document.getElementById("randomize");
randomizeButton.onclick = () => {
  universe.randomize();
  forceDrawCells();
};

// Clear

const clearButton = document.getElementById("clear");
clearButton.onclick = () => {
  universe.clear();
  forceDrawCells();
};

// Speed control

const speedInput = document.getElementById("speed");
speedInput.value = Math.floor(speed * 100);
speedInput.addEventListener("input", (_) => {
  speed = Number(speedInput.value) / 100;
});

// Start the animation

drawGrid();
play();
