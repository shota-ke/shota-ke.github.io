const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const gameDir = join(__dirname, "../static/games/falling_blocks");

function runtime(fetch) {
  const context = vm.createContext({
    console,
    WebAssembly,
    fetch,
    window: {},
    document: { querySelector: () => ({ focus() {} }) },
  });
  vm.runInContext(readFileSync(join(gameDir, "mq_js_bundle.js"), "utf8"), context);
  return context;
}

test("the game binary and bundled JavaScript runtime are compatible", async () => {
  const context = runtime();
  vm.runInContext("register_plugins(plugins)", context);
  const module = await WebAssembly.compile(readFileSync(join(gameDir, "falling_blocks.wasm")));
  const imports = vm.runInContext("importObject", context);
  const missing = WebAssembly.Module.imports(module).filter(({ module, name }) =>
    !(name in (imports[module] ?? {})));
  assert.deepEqual(missing, []);
  const instance = await WebAssembly.instantiate(module, imports);
  assert.equal(instance.exports.crate_version(), vm.runInContext("version", context));
  assert.equal(typeof instance.exports.main, "function");
  assert(instance.exports.memory instanceof WebAssembly.Memory);
});

test("the runtime reports failed downloads to the host page", async () => {
  const context = runtime(async () => { throw new Error("offline"); });
  await assert.rejects(vm.runInContext('load("falling_blocks.wasm")', context), /offline/);
});

function page(load) {
  const elements = Object.fromEntries(["glcanvas", "start-screen", "play", "status"].map(id => [id, {
    hidden: false,
    style: {},
    listeners: {},
    addEventListener(name, callback) { this.listeners[name] = callback; },
    focus() { this.focused = true; },
  }]));
  const window = {
    innerWidth: 274,
    innerHeight: 328,
    addEventListener() {},
  };
  let script;
  const context = vm.createContext({
    console: { error() {} },
    window,
    load,
    document: {
      getElementById: id => elements[id],
      createElement: () => ({}),
      body: { append(element) { script = element; } },
    },
  });
  vm.runInContext(readFileSync(join(gameDir, "game.js"), "utf8"), context);
  return { elements, script: () => script };
}

test("embedding scales the fixed board, waits for a click, and keeps Tab usable", async () => {
  const urls = [];
  const { elements, script } = page(async url => { urls.push(url); });
  const canvas = elements.glcanvas;
  assert.equal(script(), undefined);
  assert.equal(canvas.style.transform, "translate(-50%, -50%) scale(0.5)");
  let stopped = false;
  canvas.listeners.keydown({ key: "Tab", stopImmediatePropagation() { stopped = true; } });
  assert(stopped);
  elements.play.listeners.click();
  assert.equal(script().src, "mq_js_bundle.js");
  await script().onload();
  assert.deepEqual(urls, ["falling_blocks.wasm"]);
  assert(elements["start-screen"].hidden);
  assert(canvas.focused);
});

test("a failed game start shows a readable error instead of a blank canvas", async () => {
  const { elements, script } = page(async () => { throw new Error("invalid wasm"); });
  elements.play.listeners.click();
  await script().onload();
  assert.equal(elements["start-screen"].hidden, false);
  assert(elements.play.hidden);
  assert.match(elements.status.textContent, /Could not start the game/);
});
