const EVENTNAME_AI_START = "ai.start",
  EVENTNAME_AI_START_CALL = "ai.call.start",
  EVENTNAME_AI_LOADING = "ai.start.loading",
  EVENTNAME_AI_LOAD_PROGRESS = "ai.start.loading.progress",
  EVENTNAME_AI_LOADED = "ai.start.loaded",
  EVENTNAME_AI_STOP = "ai.stop",
  EVENTNAME_AI_STOP_CALL = "ai.call.stop",
  AI_START_BUTTON = document.getElementById("start-node"),
  AI_STOP_BUTTON = document.getElementById("stop-node"),
  AI_LOAD_PROGRESS_BLOCK = document.getElementById("loading-model"),
  AI_LOAD_PROGRESS = AI_LOAD_PROGRESS_BLOCK.querySelector("progress");

AI_START_BUTTON.addEventListener("click", async (e) => {
  e.stopPropagation();
  window.trigger(EVENTNAME_AI_START_CALL);
});

AI_STOP_BUTTON.addEventListener("click", () => {
  AI_STOP_BUTTON.disabled = true;
  window.trigger(EVENTNAME_AI_STOP_CALL);
});

window.on(EVENTNAME_AI_START, () => {
  AI_START_BUTTON.disabled = true;
  AI_START_BUTTON.addClass("hide");

  AI_STOP_BUTTON.disabled = false;
  AI_STOP_BUTTON.removeClass("hide");
});

window.on(EVENTNAME_AI_STOP, () => {
  AI_START_BUTTON.disabled = false;
  AI_START_BUTTON.removeClass("hide");

  AI_STOP_BUTTON.disabled = true;
  AI_STOP_BUTTON.addClass("hide");
});

window.on(EVENTNAME_AI_LOADING, () => {
  AI_LOAD_PROGRESS_BLOCK.removeClass("hide");
  AI_LOAD_PROGRESS.value = 0;
});
window.on(
  EVENTNAME_AI_LOAD_PROGRESS,
  (e) => (AI_LOAD_PROGRESS.value = e.detail.progress)
);
window.on([EVENTNAME_AI_LOADED, EVENTNAME_AI_STOP], () =>
  AI_LOAD_PROGRESS_BLOCK.addClass("hide")
);
