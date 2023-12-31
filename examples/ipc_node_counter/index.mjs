/**
 * @example
 * SOCKET_PATH=demo.sock cargo run ipc -- node examples/ipc_node_counter/index.mjs 0
 */

import assert from "assert";
import * as net from "net";
import { setTimeout } from "timers/promises";

const { SOCKET_PATH: socketPath } = process.env;
let iStr = process.argv[2];
let i = Number(iStr);

assert(socketPath, "SOCKET_PATH not set");
assert(Number.isInteger(i), "missing param");

async function main() {
  if (i >= 10) {
    console.log(`${i} reached, exiting with code 2`);
    return process.exit(2);
  }
  const sock = net.connect(socketPath);
  await setTimeout(100);
  const j = i + 1;
  const cmd = `node examples/ipc_node_counter/index.mjs ${j}\n`;
  console.log(`[child] started with id: ${i}`);
  console.log(`[child] sending cmd: ${cmd}`);
  sock.write(cmd, async (err) => {
    if (err) {
      throw err;
    }
    await setTimeout(1000);
    console.error("damn, i wasn't swapped per expectation");
    process.exit(1);
  });
}

main();
