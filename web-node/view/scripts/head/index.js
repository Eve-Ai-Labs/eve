import { init as init_settings } from "/scripts/head/settings.js?b=00000000000000";
import { init as init_balance } from "/scripts/head/balance.js?b=00000000000000";
import { init as init_tabs } from "/scripts/head/tabs.js?b=00000000000000";

export async function init() {
  await Promise.all([init_settings(), init_balance(), init_tabs()]);
}
