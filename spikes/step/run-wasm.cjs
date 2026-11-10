// Runs the wasm build in Node and compares its output with the native files.
const fs = require('fs');
const path = require('path');
const out = path.join(__dirname, 'out');
const dir = path.join(out, 'wasm-node');
const m = require(path.join(dir, 'step_spike.js'));
const t0 = Date.now();
const single = m.single_part_step();
const assy = m.assembly_step();
console.log('wasm ran in', Date.now() - t0, 'ms');
for (const [name, text] of [['single-part-named.step', single], ['assembly-named.step', assy]]) {
  const native = fs.readFileSync(path.join(out, name), 'utf8');
  console.log(name, 'wasm bytes', text.length, 'identical to native:', text === native);
}
