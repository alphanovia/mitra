// Parse every .mitra file of the golden set with the tree-sitter grammar
// and fail on any ERROR or MISSING node.
const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function find(dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) find(p, out);
    else if (p.endsWith('.mitra')) out.push(p);
  }
  return out;
}

const files = find(path.join(__dirname, '..', 'tests', 'golden'));
let bad = 0;
for (const f of files) {
  try {
    execFileSync('npx', ['tree-sitter', 'parse', '--quiet', f], { stdio: 'pipe' });
  } catch (e) {
    bad += 1;
    console.error(`${f}:\n${e.stdout}${e.stderr}`);
  }
}
console.log(`${files.length - bad} of ${files.length} golden files parse without errors`);
process.exit(bad ? 1 : 0);
