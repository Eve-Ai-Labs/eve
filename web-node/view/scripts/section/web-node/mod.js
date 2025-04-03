import { init as init_network } from "/scripts/section/web-node/network.js?b=00000000000000";
import { init as init_status } from "/scripts/section/web-node/wonnx.status.js?b=00000000000000";
import "/scripts/section/web-node/tps.js?b=00000000000000";
import "/scripts/section/web-node/req-counter.js?b=00000000000000";
import "/scripts/section/web-node/control.js?b=00000000000000";

export async function init() {
  await Promise.all([init_network(), init_status()]);
}
