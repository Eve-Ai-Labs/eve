const AI_TPS_BLOCK = document.getElementById("tps-block"),
  EVENTNAME_AI_JOB_STATUS = "web-node.ai-job.status",
  EVENTNAME_AI_STOP = "ai.stop";

window.on(EVENTNAME_AI_JOB_STATUS, (e) => {
  let detail = e.detail;
  if (detail === "Started") {
    AI_TPS_BLOCK.innerHTML = "0.0 tps";
  } else if (detail === "Done") {
    AI_TPS_BLOCK.innerHTML = "idel";
  } else if (detail.Update) {
    AI_TPS_BLOCK.innerHTML = detail.Update.tps.toFixed(1) + " tps";
  }
});
window.on(EVENTNAME_AI_STOP, () => {
  AI_TPS_BLOCK.innerHTML = detail.Update.tps.toFixed(1) + " tps";
});
