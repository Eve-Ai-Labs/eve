import "/scripts/ai.js?b=00000000000000";

import { sleep } from "/scripts/lib.js?b=00000000000000";
import { init as init_head } from "/scripts/head/index.js?b=00000000000000";
import { init as init_section } from "/scripts/section/mod.js?b=00000000000000";
import { init as init_welcome } from "/scripts/welcome.js?b=00000000000000";
import { init as init_console } from "/scripts/console.js?b=00000000000000";
import init, { WebNode } from "/wasm/webnode__web.js?b=00000000000000";

async function run() {
  await init_web_node();
  await Promise.all([
    init_head(),
    init_section(),
    init_welcome(),
    init_console(),
  ]);
}

async function init_web_node() {
  await init();

  // patch: Error: recursive use of an object detected which would lead to unsafe aliasing in rust
  let lock = false,
    lock_wait = async () => {
      while (lock) {
        await sleep(200);
      }
      lock = true;
    },
    node = await WebNode.new();

  Object.getOwnPropertyNames(Object.getPrototypeOf(node))
    .filter(
      (name) =>
        !name.startsWith("__") && name != "constructor" && name != "free"
    )
    .forEach((fn_name) => {
      let old_fn_name = "old__" + fn_name;
      node[old_fn_name] = node[fn_name];

      node[fn_name] = async (...args) => {
        await lock_wait();

        let result;
        try {
          result = await node[old_fn_name](...args);
        } finally {
          lock = false;
        }

        return result;
      };
    });
  window.web_node = node;
}

window.addEventListener("load", run);
