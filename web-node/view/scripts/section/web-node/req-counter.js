const AI_REQUEST_COUNTER_BLOCK = document.getElementById("request-counter"),
  EVENTNAME_AI_JOB_STATUS = "web-node.ai-job.status";
let AI_COUNTER = 0;

window.on(EVENTNAME_AI_JOB_STATUS, (e) => {
  let detail = e.detail;
  if (detail === "Done") {
    AI_COUNTER++;
    AI_REQUEST_COUNTER_BLOCK.innerHTML = AI_COUNTER;
  }
});
