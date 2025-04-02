import "/scripts/lib.js?b=00000000000000";

const EVENTNAME_WONNX_STATUS = "web-node.status";
let STATUS_BLOCK;

export async function init() {
  STATUS_BLOCK = document.querySelector(".connect-status span");
  console.log(STATUS_BLOCK);
  if (!STATUS_BLOCK) {
    throw new Error("Status block not found");
  }

  window.on(EVENTNAME_WONNX_STATUS, (e) => {
    let status = e.detail;
    STATUS_BLOCK.innerHTML = e.detail;
    STATUS_BLOCK.attr("class", null);
    STATUS_BLOCK.addClass(status.toLowerCase());
  });
}
