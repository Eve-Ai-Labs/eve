import init, {
  EveNode,
  EveSettings,
  EveClient,
} from "/wasm/webnode__web.js?b=00000000000000";
import { sleep } from "/scripts/lib.js?b=00000000000000";

const EVENTNAME_SETTINGS_CHANGED = "settings.changed",
  EVENTNAME_WASM_INIT = "wasm.init";

// window.eve_settings
// window.eve_node;
// window.eve_client;
export async function init_wasm() {
  await init();
  window.eve_settings = await EveSettings.load();

  await init_settings();
  await init_client();
  await init_web_node();
  window.trigger(EVENTNAME_WASM_INIT);
}

async function init_settings() {
  window.eve_settings = await EveSettings.load();
}

async function init_web_node() {
  if (window.eve_node) {
    try {
      await window.eve_node.stop();
    } catch (e) {
      console.log(e);
    }
  }
  // patch: Error: recursive use of an object detected which would lead to unsafe aliasing in rust
  let node = await EveNode.new();
  window.eve_node = add_lock_to_object(node);
}

async function init_client() {
  if (!(await window.eve_settings.get()).private_key) {
    window.eve_client = null;
  }
  let client = await EveClient.new();

  window.eve_client = add_lock_to_object(client);
}

window.on(EVENTNAME_SETTINGS_CHANGED, init_web_node);
window.on(EVENTNAME_SETTINGS_CHANGED, init_client);
window.on(EVENTNAME_SETTINGS_CHANGED, init_settings);

function add_lock_to_object(obj) {
  // patch: Error: recursive use of an object detected which would lead to unsafe aliasing in rust
  let lock = false,
    lock_wait = async () => {
      while (lock) {
        await sleep(200);
      }
      lock = true;
    };

  Object.getOwnPropertyNames(Object.getPrototypeOf(obj))
    .filter(
      (name) =>
        !name.startsWith("__") && name != "constructor" && name != "free"
    )
    .forEach((fn_name) => {
      let old_fn_name = "old__" + fn_name;
      obj[old_fn_name] = obj[fn_name];

      obj[fn_name] = async (...args) => {
        await lock_wait();

        let result;
        try {
          result = await obj[old_fn_name](...args);
        } finally {
          lock = false;
        }

        return result;
      };
    });

  return obj;
}
