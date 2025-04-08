import "/scripts/ai.js?b=00000000000000";

import { sleep } from "/scripts/lib.js?b=00000000000000";
import { init_wasm } from "/scripts/wasm.js?b=00000000000000";
import { init as init_head } from "/scripts/head/index.js?b=00000000000000";
import { init as init_section } from "/scripts/section/mod.js?b=00000000000000";
import { init as init_welcome } from "/scripts/welcome.js?b=00000000000000";
import { init as init_console } from "/scripts/console.js?b=00000000000000";

async function run() {
  // @todo fix the progress bar
  await init_wasm();
  await Promise.all([
    init_head(),
    init_section(),
    init_welcome(),
    init_console(),
  ]);
}

window.addEventListener("load", run);
