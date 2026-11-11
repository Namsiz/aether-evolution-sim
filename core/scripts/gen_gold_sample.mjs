// Generate binary gold sample for aether-rng integration test.
// Usage: node scripts/gen_gold_sample.mjs [seed] [count]
// Output: /tmp/mulberry32_gold_sample_bytes.bin (little-endian f64 doubles)
import { mulberry32 } from '/root/workspace/world/living/src/world.js';
import { writeFileSync } from 'fs';

const seed = Number(process.argv[2]) || 20261102;
const count = Number(process.argv[3]) || 1_000_000;
const rng = mulberry32(seed);
const buf = Buffer.alloc(count * 8);
for (let i = 0; i < count; i++) {
  buf.writeDoubleLE(rng(), i * 8);
}
writeFileSync('/tmp/mulberry32_gold_sample_bytes.bin', buf);
console.log(`Wrote ${count} × f64 samples (seed=${seed}) → /tmp/mulberry32_gold_sample_bytes.bin`);
