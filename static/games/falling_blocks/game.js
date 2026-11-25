const gameCanvas = document.getElementById("glcanvas");
const startScreen = document.getElementById("start-screen");
const playButton = document.getElementById("play");
const gameStatus = document.getElementById("status");

function fitGame() {
  const scale = Math.min(1, window.innerWidth / 548, window.innerHeight / 656);
  gameCanvas.style.transform = `translate(-50%, -50%) scale(${scale})`;
}

function showGameError(error) {
  console.error(error);
  startScreen.hidden = false;
  playButton.hidden = true;
  gameStatus.textContent = "Could not start the game. Please reload the page and try again.";
}

fitGame();
window.addEventListener("resize", fitGame);
window.addEventListener("error", (event) => showGameError(event.error));
gameCanvas.addEventListener("pointerdown", () => gameCanvas.focus());
gameCanvas.addEventListener("keydown", (event) => {
  // Let Tab leave the game instead of being captured by macroquad.
  if (event.key === "Tab") event.stopImmediatePropagation();
}, true);

playButton.addEventListener("click", () => {
  playButton.disabled = true;
  gameStatus.textContent = "Loading game…";
  gameCanvas.hidden = false;
  const script = document.createElement("script");
  script.src = "mq_js_bundle.js";
  script.onerror = showGameError;
  script.onload = () => load("falling_blocks.wasm").then(() => {
    startScreen.hidden = true;
    gameCanvas.focus();
  }).catch(showGameError);
  document.body.append(script);
});
