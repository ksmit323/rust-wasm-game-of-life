/******************************************************
 *                 1) Imports and Constants
 *****************************************************/
import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg.wasm";

// Visual constants
const CELL_SIZE   = 10;        // px
const GRID_COLOR  = "#CCCCCC";
const DEAD_COLOR  = "#FFFFFF";
const ALIVE_COLOR = "#000000";

/******************************************************
 *           2) Create Universe and Canvas
 *****************************************************/
// Construct the universe
const universe = Universe.new();
const width    = universe.width();
const height   = universe.height();

// Set canvas to fit the grid (cells + 1px border per cell)
const canvas  = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width  = (CELL_SIZE + 1) * width  + 1;

// 2D rendering context
const ctx = canvas.getContext("2d");

/******************************************************
 *         3) Helper Functions (Drawing, etc.)
 *****************************************************/
/**
 * Draw the full grid lines (vertical + horizontal).
 */
function drawGrid() {
  ctx.beginPath();
  ctx.strokeStyle = GRID_COLOR;

  // Vertical lines
  for (let i = 0; i <= width; i++) {
    ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
    ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
  }
  // Horizontal lines
  for (let j = 0; j <= height; j++) {
    ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
    ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
  }
  ctx.stroke();
}

/** 
 * Return index in bit-packed array from (row, col).
 * (Included for clarity, but your logic is the same.)
 */
function getIndex(row, column) {
  return row * width + column;
}

/**
 * Check if bit `n` is set in the bit-packed `Uint8Array`.
 */
function bitIsSet(n, arr) {
  const byte = Math.floor(n / 8);
  const mask = 1 << (n % 8);
  return (arr[byte] & mask) === mask;
}

/**
 * Draw only cells that changed (from the universe's changed_cells).
 */
function drawChangedCells() {
  const changedPointer = universe.changed_cells_ptr();
  const changedLength  = universe.changed_cells_length();
  const changedCells   = new Uint32Array(memory.buffer, changedPointer, changedLength);
  
  const cellsPtr = universe.cells();
  const cells    = new Uint8Array(memory.buffer, cellsPtr, (width * height) / 8);

  ctx.beginPath();
  
  for (let i = 0; i < changedLength; i++) {
    const index = changedCells[i];
    const row   = Math.floor(index / width);
    const col   = index % width;
    
    const alive = bitIsSet(index, cells);
    ctx.fillStyle = alive ? ALIVE_COLOR : DEAD_COLOR;
    
    ctx.fillRect(
      col * (CELL_SIZE + 1) + 1,
      row * (CELL_SIZE + 1) + 1,
      CELL_SIZE,
      CELL_SIZE
    );
  }
  ctx.stroke();
}

/******************************************************
 *                4) FPS Counter Class
 *****************************************************/
class FPSCounter {
  constructor() {
    this.fpsElement         = document.getElementById("fps");
    this.frames             = [];
    this.lastFrameTimestamp = performance.now();
  }
  
  render() {
    const now   = performance.now();
    const delta = now - this.lastFrameTimestamp;
    this.lastFrameTimestamp = now;
    
    const fps = 1000 / delta;  // frames per second
    this.frames.push(fps);
    if (this.frames.length > 100) {
      this.frames.shift();
    }
    
    let min = Infinity, max = -Infinity, sum = 0;
    for (let i = 0; i < this.frames.length; i++) {
      sum += this.frames[i];
      min = Math.min(min, this.frames[i]);
      max = Math.max(max, this.frames[i]);
    }
    const mean = sum / this.frames.length;
    
    this.fpsElement.textContent = `
      Frames per Second:
              latest = ${Math.round(fps)}
      avg of last 100 = ${Math.round(mean)}
      min of last 100 = ${Math.round(min)}
      max of last 100 = ${Math.round(max)}
    `.trim();
  }
}

/******************************************************
 *   5) Animation Loop & Play/Pause (renderloop)
 *****************************************************/
let animationId = null;
const fps = new FPSCounter();

/**
 * The main animation loop.
 * 1) Render FPS
 * 2) Draw grid
 * 3) Read slider -> do N ticks
 * 4) For each tick, draw changed cells
 * 5) schedule next frame
 */
function renderloop() {
  fps.render();
  drawGrid();
  
  const ticksPerFrame = parseInt(ticksRange.value, 10) || 1;
  for (let i = 0; i < ticksPerFrame; i++) {
    universe.tick();
    drawChangedCells();
  }
  
  animationId = requestAnimationFrame(renderloop);
}

/** Check if currently paused. */
function isPaused() {
  return animationId === null;
}

/** Start the simulation. */
function play() {
  playPauseButton.textContent = "⏸";
  renderloop();
}

/** Pause the simulation. */
function pause() {
  playPauseButton.textContent = "▶";
  cancelAnimationFrame(animationId);
  animationId = null;
}

/******************************************************
 *       6) DOM Elements and Event Listeners
 *****************************************************/
// Slider controlling ticks per frame
const ticksRange   = document.getElementById("ticks-range");
const ticksDisplay = document.getElementById("ticks-display");

ticksRange.addEventListener("input", () => {
  ticksDisplay.textContent = ticksRange.value;
});

// Play/Pause button
const playPauseButton = document.getElementById("play-pause");
playPauseButton.addEventListener("click", () => {
  if (isPaused()) {
    play();
  } else {
    pause();
  }
});

// Random button
const randomButton = document.getElementById("random-btn");
randomButton.addEventListener("click", () => {
  universe.randomize();
  drawGrid();
  drawChangedCells();
});

// Clear button
const clearButton = document.getElementById("clear-btn");
clearButton.addEventListener("click", () => {
  universe.clear();
  drawGrid();
  drawChangedCells();
});

/**
 * Canvas click -> toggles or inserts a pattern.
 *  - Ctrl + click => glider
 *  - Shift + click => pulsar
 *  - Otherwise => toggle cell
 */
canvas.addEventListener("click", event => {
  const boundingRect = canvas.getBoundingClientRect();

  const scaleX = canvas.width  / boundingRect.width;
  const scaleY = canvas.height / boundingRect.height;

  const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
  const canvasTop  = (event.clientY - boundingRect.top ) * scaleY;

  const row = Math.min(Math.floor(canvasTop  / (CELL_SIZE + 1)), height - 1);
  const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width  - 1);

  if (event.ctrlKey) {
    universe.insert_glider_at(row, col);
  } else if (event.shiftKey) {
    universe.insert_pulsar_at(row, col);
  } else {
    universe.toggle_cell(row, col);
  }
  
  drawGrid();
  drawChangedCells();
});

/******************************************************
 *                 7) Initialize and Start
 *****************************************************/
drawGrid();
drawChangedCells();
play();
