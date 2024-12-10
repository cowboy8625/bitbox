let memoryBuffer;
const imports = {
  core: {
    write: (ptr, len) => {
      const stringData = new TextDecoder("utf-8").decode(
        memoryBuffer.slice(ptr, ptr + len),
      );
      console.log(stringData);
    },
  },
};

async function getWasmBuffer(name) {
  if (typeof window !== "undefined" && typeof window.document !== "undefined") {
    const result = await fetch(name);
    const buffer = await result.arrayBuffer();
    return buffer;
  } else {
    const fs = require("fs");
    return fs.readFileSync(name);
  }
}

async function main() {
  const buffer = await getWasmBuffer("hello_world.wasm");
  const module = await WebAssembly.instantiate(buffer, imports);
  const memory = module.instance.exports.memory;
  memoryBuffer = new Uint8Array(memory.buffer);
  const exitCode = module.instance.exports.main();
  if (exitCode !== 0) {
    throw new Error(`wasm exited with code ${exitCode}`);
  }
}

main();
