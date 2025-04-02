import "/scripts/lib.js?b=00000000000000";

const EVENTNAME_AI_START = "ai.start",
  EVENTNAME_AI_START_CALL = "ai.call.start",
  EVENTNAME_AI_LOADING = "ai.start.loading",
  EVENTNAME_AI_LOAD_PROGRESS = "ai.start.loading.progress",
  EVENTNAME_AI_LOADED = "ai.start.loaded",
  EVENTNAME_AI_STOP = "ai.stop",
  EVENTNAME_AI_STOP_CALL = "ai.call.stop",
  EVENTNAME_AI_JOB_STATUS = "web-node.ai-job.status";

let AI_START_BUTTON,
  AI_STOP_BUTTON,
  AI_LOAD_PROGRESS,
  AI_COUNTER_BLOCK,
  AI_TPS_BLOCK,
  AI_COUNTER = 0;

export async function init() {
  await Promise.all([
    init_start_button(),
    init_stop_button(),
    init_progress(),
    init_counter(),
  ]);
}

async function init_start_button() {
  AI_START_BUTTON = document.querySelector('button[name="ai-start"]');
  if (!AI_START_BUTTON) {
    throw new Error("(AI) start button not found");
  }

  AI_START_BUTTON.addEventListener("click", async (e) => {
    e.stopPropagation();
    window.trigger(EVENTNAME_AI_START_CALL);
  });

  window.on(EVENTNAME_AI_START, () => {
    AI_START_BUTTON.disabled = true;
    AI_START_BUTTON.addClass("hide");
  });

  window.on(EVENTNAME_AI_STOP, () => {
    AI_START_BUTTON.disabled = false;
    AI_START_BUTTON.removeClass("hide");
  });
}

async function init_stop_button() {
  AI_STOP_BUTTON = document.querySelector('button[name="ai-stop"]');
  if (!AI_STOP_BUTTON) {
    throw new Error("(AI) stop button not found");
  }

  AI_STOP_BUTTON.addEventListener("click", () => {
    AI_STOP_BUTTON.disabled = true;
    window.trigger(EVENTNAME_AI_STOP_CALL);
  });

  window.on(EVENTNAME_AI_START, () => {
    AI_STOP_BUTTON.disabled = false;
    AI_STOP_BUTTON.removeClass("hide");
  });
  window.on(EVENTNAME_AI_STOP, () => {
    AI_STOP_BUTTON.disabled = true;
    AI_STOP_BUTTON.addClass("hide");
  });
}

export function init_progress() {
  AI_LOAD_PROGRESS = document.querySelector("progress");
  if (!AI_LOAD_PROGRESS) {
    throw new Error("(AI) progress bar not found");
  }

  window.on(EVENTNAME_AI_LOADING, () => AI_LOAD_PROGRESS.removeClass("hide"));
  window.on(
    EVENTNAME_AI_LOAD_PROGRESS,
    (e) => (AI_LOAD_PROGRESS.value = e.detail.progress)
  );
  window.on([EVENTNAME_AI_LOADED, EVENTNAME_AI_STOP], () =>
    AI_LOAD_PROGRESS.addClass("hide")
  );
}

async function init_counter() {
  AI_COUNTER_BLOCK = document.querySelector(".request-counter .value");
  if (!AI_COUNTER_BLOCK) {
    throw new Error("(AI) counter block not found");
  }

  AI_TPS_BLOCK = document.querySelector(".tps-block .value");
  if (!AI_TPS_BLOCK) {
    throw new Error("(AI) tps block not found");
  }

  window.on(EVENTNAME_AI_JOB_STATUS, (e) => {
    let detail = e.detail;
    if (detail === "Started") {
      AI_TPS_BLOCK.innerHTML = "0.0 tps";
    } else if (detail === "Done") {
      AI_COUNTER++;
      AI_TPS_BLOCK.innerHTML = "idel";
    } else if (detail.Update) {
      AI_TPS_BLOCK.innerHTML = detail.Update.tps.toFixed(1) + " tps";
    }
  });
}
