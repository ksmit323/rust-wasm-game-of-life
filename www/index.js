import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg.wasm";

const CELL_SIZE = 10; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

// Construct the universe
const universe = Universe.new();
const width = universe.width();
const height = universe.height();

// Give the canvas room for all of the cells and a 1px border
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop = (event.clientY - boundingRect.top) * scaleY;

  const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
  const column = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

  if (event.ctrlKey) {
    universe.insert_glider_at(row, column);
  } else if (event.shiftKey) {
    universe.insert_pulsar_at(row, column);
  } else {
    universe.toggle_cell(row, column);
  }
  
  drawGrid();
  drawChangedCells();
});

const ctx = canvas.getContext('2d');

const ticksRange = document.getElementById("ticks-range");
const ticksDisplay = document.getElementById("ticks-display");
ticksRange.addEventListener("input", () => {
  ticksDisplay.textContent = ticksRange.value;
});

const randomButton = document.getElementById("random-btn");
randomButton.addEventListener("click", () => {
  universe.randomize();
  drawGrid();
  drawChangedCells();
});

const clearButton = document.getElementById("clear-btn");
clearButton.addEventListener("click", () => {
  universe.clear();
  drawGrid();
  drawChangedCells();
});

let animationId = null;

const renderloop = () => {
  fps.render();
  
  drawGrid();
  
  // Convert slider value to number 
  const ticksPerFrame = parseInt(ticksRange.value, 10);
  
  // Run multiple ticks before rendering the next frame
  for (let i = 0; i < ticksPerFrame; i++) {
    universe.tick()
    drawChangedCells();
  }
  
  animationId = requestAnimationFrame(renderloop);
};

const isPaused = () => {
  return animationId === null;
}

const playPauseButton = document.getElementById("play-pause");

const play = () => {
  playPauseButton.textContent = "⏸";
  renderloop();
}

const pause = () => {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);  
  animationId = null;
}

playPauseButton.addEventListener("click", event => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

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
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }

  ctx.stroke();
};

const getIndex = (row, column) => {
  return row * width + column;
}

const bitIsSet = (n, arr) => {
  const byte = Math.floor(n / 8);
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
};

const drawChangedCells = () => {
  const changedPointer = universe.changed_cells_ptr();
  const changedLength = universe.changed_cells_length();
  const changedCells = new Uint32Array(memory.buffer, changedPointer, changedLength);
  
  const cellsPtr = universe.cells();
  // Because each cell is 1 bit, total bits = (width * height).
  // The number of bytes is (width * height) / 8.
  const cells = new Uint8Array(memory.buffer, cellsPtr, width * height / 8);

  ctx.beginPath();
  
  for (let i = 0; i < changedLength; i++) {
    const index = changedCells[i];
    const row = Math.floor(index / width);
    const column = index % width;
    
    const alive = bitIsSet(index, cells);
    
    ctx.fillStyle = alive ? ALIVE_COLOR : DEAD_COLOR;
    
    ctx.fillRect(
      column * (CELL_SIZE + 1) + 1,
      row * (CELL_SIZE + 1) + 1,
      CELL_SIZE,
      CELL_SIZE
    );  
  }
  
  ctx.stroke();
};

const fps = new class {
  constructor() {
    this.fps = document.getElementById("fps");
    this.frames = [];
    this.lastFrameTimestamp = performance.now();
  }
  
  render() {
    const now = performance.now();
    const delta = now - this.lastFrameTimestamp;
    this.lastFrameTimestamp = now;
    const fps = 1 / delta * 1000;
    
    // Save only the latest 100 timings
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }
    
    // Find max, min and mean of the 100 timings
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(this.frames[i], min);
      max = Math.max(this.frames[i], max);
    }
    let mean = sum / this.frames.length;
    
    // Render the statistics
    this.fps.textContent = `
      Frames per Second:
              latest = ${Math.round(fps)}
      avg of last 100 = ${Math.round(mean)}
      min of last 100 = ${Math.round(min)}
      max of last 100 = ${Math.round(max)}
      `.trim();
  }
};

drawGrid();
drawChangedCells();
play();