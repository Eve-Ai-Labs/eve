import "/scripts/lib.js?b=00000000000000";

const EVENTNAME_SETTINGS_CHANGED = "settings.changed",
  EVENTNAME_WONNX_LOADING_PROGRESS = "web-node.loading.progress",
  EVENTNAME_AI_START_CALL = "ai.call.start",
  EVENTNAME_AI_START = "ai.start",
  EVENTNAME_AI_READY = "ai.start.ready",
  EVENTNAME_AI_LOADING = "ai.start.loading",
  EVENTNAME_AI_LOAD_PROGRESS = "ai.start.loading.progress",
  EVENTNAME_AI_LOADED = "ai.start.loaded",
  EVENTNAME_AI_STOP_CALL = "ai.call.stop",
  EVENTNAME_AI_STOP = "ai.stop";

let LOADED = false,
  STOP;

window.on(EVENTNAME_WONNX_LOADING_PROGRESS, (e) => {
  let ok = e.detail?.Ok;
  if (!ok) {
    console.error(e.detail);
    return window.trigger(EVENTNAME_AI_STOP_CALL);
  }
  if (STOP) return;
  if (ok == "Start") {
    return window.trigger(EVENTNAME_AI_LOADING);
  }
  if (ok == "Compile") {
    return window.trigger(EVENTNAME_AI_LOADED);
  }
  if (ok == "Done") {
    return window.trigger(EVENTNAME_AI_READY);
  }

  let download = ok.Download?.progress;
  if (download !== undefined) {
    return window.trigger(
      EVENTNAME_AI_LOAD_PROGRESS,
      {
        progress: download,
      },
      true
    );
  }
  let progress = ok.Progress?.progress;
  if (progress !== undefined) {
    return window.trigger(
      EVENTNAME_AI_LOAD_PROGRESS,
      {
        progress,
      },
      true
    );
  }
});

window.on(EVENTNAME_AI_START_CALL, async () => {
  STOP = false;
  if (LOADED) {
    return console.error("The AI has already been launched");
  }
  window.trigger(EVENTNAME_AI_START);

  try {
    LOADED = true;
    await window.eve_node.start();
  } catch (err) {
    console.cn_error(err);
    window.trigger(EVENTNAME_AI_STOP_CALL);
  }
});

window.on(EVENTNAME_AI_STOP_CALL, async () => {
  STOP = true;
  if (!LOADED) return;

  await window.eve_node.stop_wait_disconnect();
  LOADED = false;
  window.trigger(EVENTNAME_AI_STOP);
  STOP = false;
});

window.on(EVENTNAME_SETTINGS_CHANGED, () => {
  window.trigger(EVENTNAME_AI_STOP_CALL);
});
