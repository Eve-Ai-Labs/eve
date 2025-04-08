import { init as init_view } from "/scripts/section/chat/view.js?b=00000000000000";

export async function init() {
  await Promise.all([init_view()]);
}
