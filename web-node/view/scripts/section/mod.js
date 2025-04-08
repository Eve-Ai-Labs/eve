import { init as init_webnode } from "/scripts/section/web-node/mod.js?b=00000000000000";
import { init as init_chat } from "/scripts/section/chat/mod.js?b=00000000000000";

export async function init() {
  await Promise.all([init_webnode(), init_chat()]);
}
