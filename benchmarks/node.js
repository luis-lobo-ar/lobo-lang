'use strict';

// BigInt preserves the full I64 domain; Number would change the language semantics.
const MIN_I64 = -(1n << 63n);
const MAX_I64 = (1n << 63n) - 1n;

function checked(value) {
  if (value < MIN_I64 || value > MAX_I64) throw new RangeError('I64 overflow');
  return value;
}

function increment(x) {
  if (x >= MAX_I64) throw new Error('precondition');
  const result = checked(x + 1n);
  if (result !== checked(x + 1n)) throw new Error('postcondition');
  return result;
}

function absolute(x) {
  if (x === MIN_I64) throw new Error('precondition');
  const result = x < 0n ? checked(-x) : x;
  if (result < 0n) throw new Error('postcondition');
  return result;
}

function batch(fn, count) {
  let checksum = 0n;
  for (let i = 0; i < count; ++i) {
    checksum = BigInt.asUintN(64, checksum + fn(BigInt((i & 2047) - 1024)));
  }
  return checksum;
}

function selfTest() {
  for (const x of [MIN_I64, MIN_I64 + 1n, -1024n, -1n, 0n, 1n, 1023n, MAX_I64 - 1n, MAX_I64]) {
    if (x !== MAX_I64 && increment(x) !== x + 1n) throw new Error('incorrect increment');
    if (x !== MIN_I64 && absolute(x) !== (x < 0n ? -x : x)) throw new Error('incorrect absolute');
  }
  for (const [fn, x, message] of [
    [increment, MAX_I64, 'precondition'], [absolute, MIN_I64, 'precondition'],
    [checked, MAX_I64 + 1n, 'I64 overflow'], [checked, MIN_I64 - 1n, 'I64 overflow'],
  ]) {
    let caught = false;
    try { fn(x); } catch (error) {
      if (error.message !== message) throw error;
      caught = true;
    }
    if (!caught) throw new Error('missing contract/overflow failure');
  }
  console.log('self-test passed');
}

const args = process.argv.slice(2);
if (args.length === 1 && args[0] === '--self-test') {
  selfTest();
} else {
  const count = Number(args[1]);
  const warmup = Number(args[2]);
  if (args.length !== 3 || !['increment', 'absolute'].includes(args[0]) ||
      !Number.isInteger(count) || count < 1 || count > 1e9 ||
      !Number.isInteger(warmup) || warmup < 0 || warmup > 1e9) {
    throw new Error('usage: node.js increment|absolute ITERATIONS WARMUP_ITERATIONS');
  }
  const fn = args[0] === 'increment' ? increment : absolute;
  batch(fn, warmup);
  const start = process.hrtime.bigint();
  const checksum = batch(fn, count);
  const elapsed = process.hrtime.bigint() - start;
  console.log(JSON.stringify({ns_per_call: Number(elapsed) / count, elapsed_ns: String(elapsed),
                             checksum: String(checksum), iterations: count}));
}
