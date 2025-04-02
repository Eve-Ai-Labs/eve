import { init as init_settings } from "/scripts/head/settings.js?b=00000000000000";
import { init as init_balance } from "/scripts/head/balance.js?b=00000000000000";
import { init as init_network } from "/scripts/head/network.js?b=00000000000000";
import { init as init_ai } from "/scripts/head/ai.js?b=00000000000000";
import { init as init_status } from "/scripts/head/wonnx.status.js?b=00000000000000";

export async function init() {
  await Promise.all([
    init_settings(),
    init_balance(),
    init_network(),
    init_ai(),
    init_status(),
  ]);
}
