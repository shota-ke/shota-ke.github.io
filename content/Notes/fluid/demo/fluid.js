const fluidCanvas = document.getElementById("glcanvas");
const startScreen = document.getElementById("start-screen");
const playButton = document.getElementById("play");
const fluidStatus = document.getElementById("status");
const is2d = new URLSearchParams(window.location.search).get("mode") === "2d";
const controls = is2d
  ? "Left drag: add dye and stir."
  : "Left drag: stir and emit particles.\nShift + left drag: rotate camera. Scroll: zoom.";

document.title = `${is2d ? "2D" : "3D"} Fluid Simulation in Rust · Shota Ikari`;
document.getElementById("controls").textContent = controls;
fluidCanvas.setAttribute("aria-label", `${is2d ? "2D" : "3D"} fluid simulation. ${controls}`);

function showFluidError(error) {
  console.error(error);
  fluidCanvas.hidden = true;
  startScreen.hidden = false;
  playButton.hidden = true;
  fluidStatus.textContent = "Could not start the simulation. Please reload and try a browser with WebGL and WebAssembly support.";
}

window.addEventListener("error", (event) => showFluidError(event.error));
fluidCanvas.addEventListener("pointerdown", () => fluidCanvas.focus());
// Let the article scroll in 2D; 3D uses the wheel to zoom.
if (is2d) {
  fluidCanvas.addEventListener("wheel", (event) => event.stopImmediatePropagation(), { capture: true, passive: true });
}
fluidCanvas.addEventListener("keydown", (event) => {
  // Let Tab leave the canvas instead of being captured by macroquad.
  if (event.key === "Tab") event.stopImmediatePropagation();
}, true);

playButton.addEventListener("click", () => {
  playButton.disabled = true;
  fluidStatus.textContent = "Loading simulation…";
  fluidCanvas.hidden = false;
  const script = document.createElement("script");
  script.src = "mq_js_bundle.js";
  script.onerror = showFluidError;
  // The bundled loader returns its promise and propagates errors to this page.
  script.onload = () => {
    if (fluidCanvas.hidden) return;
    return load(is2d ? "fluid-2d.wasm" : "fluid.wasm").then(() => {
      if (fluidCanvas.hidden) return;
      startScreen.hidden = true;
      fluidCanvas.focus();
    }).catch(showFluidError);
  };
  document.body.append(script);
});
