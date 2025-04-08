import "/scripts/lib.js?b=00000000000000";

const MAX_ITEM = 300,
  STATUS_TITLE = { false: "Open the console", true: "Minimize the console" },
  STATUS_CLASS = {
    false: "console_status_close",
    true: "console_status_open",
  };
let MAIN_BLOCK = document.querySelector("main"),
  CONSOLE_BLOCK = document.getElementById("console"),
  SWITCH_BUTTON = CONSOLE_BLOCK.querySelector(".switch"),
  MESSAGE_BLOCK = CONSOLE_BLOCK.querySelector(".bd"),
  // null = default
  // true = open
  // false = close
  CONSOLE_STATUS = null;

export async function init() {
  console_status();

  SWITCH_BUTTON.addEventListener("click", () =>
    console_status(!CONSOLE_STATUS)
  );
}

function console_status(value) {
  let status =
    value != undefined
      ? value
      : CONSOLE_STATUS != null
      ? CONSOLE_STATUS
      : localStorage.getItem("console_status") == "true";
  status = status != null ? status && true : true;

  CONSOLE_STATUS = status;
  localStorage.setItem("console_status", status);

  MAIN_BLOCK.removeClass(STATUS_CLASS[!status]);
  MAIN_BLOCK.addClass(STATUS_CLASS[status]);
  SWITCH_BUTTON.attr("title", STATUS_TITLE[status]);
}

function log(type, ...message) {
  message = message.join(" ");
  if (!message.length) {
    return;
  }
  let row = document.createElement("p");
  row.addClass(type);
  let now = new Date();
  row.innerHTML = "[" + now.toLocaleTimeString("it-IT") + "] " + message;

  MESSAGE_BLOCK.append(row);

  if (
    MESSAGE_BLOCK.scrollHeight <
    MESSAGE_BLOCK.scrollTop + MESSAGE_BLOCK.clientHeight + row.clientHeight + 30
  ) {
    if (MESSAGE_BLOCK.childNodes.length > MAX_ITEM + 50) {
      for (let i = MESSAGE_BLOCK.childNodes.length - MAX_ITEM; i > 0; i--) {
        MESSAGE_BLOCK.firstChild.remove();
      }
    }

    row.scrollIntoView();
  }
  return row;
}

console.cn_trace = (...message) => {
  console.trace(...message);
  return log("trace", ...message);
};

console.cn_debug = (...message) => {
  console.debug(...message);
  return log("debug", ...message);
};

console.cn_log = (...message) => {
  console.log(...message);
  return log("log", ...message);
};

console.cn_info = (...message) => {
  console.info(...message);
  return log("info", ...message);
};

console.cn_warn = (...message) => {
  console.warn(...message);
  return log("warn", ...message);
};

console.cn_error = (...message) => {
  console.error(...message);
  return log("error", ...message);
};
