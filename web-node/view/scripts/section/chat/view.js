import { sleep } from "/scripts/lib.js";
import "/scripts/section/chat/proc.js?b=00000000000000";

const ANSWERS_BLOCK = document.querySelector("#chat .answers"),
  INPUT = document.getElementById("question"),
  SEND_BUTTON = document.getElementById("send_question"),
  CLEAR_BUTTON = document.getElementById("clear_history"),
  EVENTNAME_WASM_INIT = "wasm.init",
  EVENTNAME_CHAT_CLEAR = "chat.clear",
  EVENTNAME_CHAT_CALL_ASK = "chat.call.ask",
  EVENTNAME_CHAT_ASK_START = "chat.ask.start",
  EVENTNAME_CHAT_ASK_RESPONSE = "chat.ask.response",
  EVENTNAME_CHAT_ASK_FINISHED = "chat.ask.finished",
  TEMPLATE_ASK = `<div id="query_{{ID}}" class="row question" query-id="{{ID}}">
        <div class="message">
          <div class="hd">you</div>
          <div class="value">{{VALUE}}</div>
        </div>
    </div>`,
  TEMPLATE_ANS_STATUS = `<div class="gl-status" node-id="null">Node <span class="node_id">0x1234567</span>: <span class="status">send request</span></div>`,
  TEMPLATE_ANS_MESSAGE = `<div class="message" node-id="null">
          <div class="hd">
            <div class="name">Node: <span>null</></div>
            <div class="status">Status: <span>waiting...</span></div>
          </div>
          <div class="value">waiting...</div>
        </div>`,
  TEMPLATE_ANS =
    `<div id="answer_{{ID}}" class="row answer" query-id="{{ID}}">
      <div class="tb">
        <div class="prev">🢀</div>
        <div class="nv">0/0</div>
        <div class="next">🢂</div>
      </div>
      <div class="items">
      ` +
    TEMPLATE_ANS_MESSAGE +
    `</div>
    </div>`;

let ANSWERS = [];

export async function init() {
  let settings = await window.eve_settings.get();
  //
  if (settings.private_key !== undefined) {
    unlock_inputs();
  }

  // history
  let history = await self.eve_client.get_history();
  if (history) {
    ANSWERS_BLOCK.querySelectorAll(".empty").forEach((e) => e.remove());
    history.map((item, i) => {
      if (item.role === "User") {
        draw_question(i, item.content);
      } else if (item.role === "Assistant") {
        draw_answer(i, [
          {
            node_id: "history_" + i,
            finished: true,
            status: "finished",
            message: item.content,
          },
        ]);
      }
    });
    ANSWERS_BLOCK.scrollTop = ANSWERS_BLOCK.scrollHeight;
    CLEAR_BUTTON.disabled = false;
  }
}

INPUT.on(["change", "keyup"], () => {
  SEND_BUTTON.disabled = !INPUT.value.trim();
});
INPUT.on("keyup", (e) => {
  if ((e.keyCode == 10 || e.keyCode == 13) && e.ctrlKey) {
    SEND_BUTTON.click();
  }
});

SEND_BUTTON.on("click", () => {
  let value = INPUT.value.trim();
  if (!value) {
    return;
  }
  window.trigger(EVENTNAME_CHAT_CALL_ASK, value);
});
window.on(EVENTNAME_WASM_INIT, unlock_inputs);

CLEAR_BUTTON.on("click", () => {
  window.eve_client.clear_history();
  CLEAR_BUTTON.disabled = true;
  ANSWERS_BLOCK.innerHTML = `<div class="empty">The answers will appear here.</div>`;
  ANSWERS = [];
  window.trigger(EVENTNAME_CHAT_CLEAR);
});

window.on(EVENTNAME_CHAT_ASK_START, (e) => {
  let value = e.detail.text,
    id = e.detail.id;
  ANSWERS_BLOCK.querySelectorAll(".empty").forEach((e) => e.remove());
  INPUT.value = value;
  INPUT.disabled = true;
  SEND_BUTTON.disabled = true;
  CLEAR_BUTTON.disabled = false;

  draw_question(id, value);
  draw_answer(id);
});
window.on([EVENTNAME_CHAT_ASK_FINISHED, EVENTNAME_CHAT_CLEAR], () => {
  INPUT.value = "";
  INPUT.disabled = false;
  SEND_BUTTON.disabled = false;
});

window.on([EVENTNAME_CHAT_ASK_RESPONSE, EVENTNAME_CHAT_ASK_FINISHED], (e) => {
  draw_answer(e.detail.id, e.detail.response);
});

function unlock_inputs() {
  INPUT.disabled = false;
}
function draw_question(id, value) {
  let html = TEMPLATE_ASK.replaceAll("{{ID}}", id).replaceAll(
    "{{VALUE}}",
    value
  );
  ANSWERS_BLOCK.insertAdjacentHTML("beforeend", html);
  scroll_to_end();
}

function draw_answer(query_id, response) {
  let answer = ANSWERS.find((item) => item.query_id == query_id);
  if (!answer) {
    answer = new Answer(query_id);
    ANSWERS.push(answer);
  }
  answer.set(response);
  if (!response) scroll_to_end();
}

function scroll_to_end() {
  let items = ANSWERS_BLOCK.childNodes;
  if (!items || !items.length) return;
  let last = items[items.length - 1];
  let last_height = last.offsetHeight + 60; // 60 - margin

  if (
    ANSWERS_BLOCK.scrollHeight -
      ANSWERS_BLOCK.scrollTop -
      ANSWERS_BLOCK.clientHeight -
      last_height >
    30
  ) {
    return;
  }

  ANSWERS_BLOCK.scrollTop = ANSWERS_BLOCK.scrollHeight;
}

class Answer {
  block;
  block_items;
  block_navigator;
  block_navigator_counter;
  query_id;
  messages;
  statuses;
  active_index;

  constructor(query_id) {
    this.query_id = query_id;
    if (!this.block_exists()) this.create_block();
    this.block_items = this.block.querySelector(".items");
    this.block_navigator = this.block.querySelector(".tb");
    this.block_navigator_counter = this.block_navigator.querySelector(".nv");
  }
  block_exists() {
    if (!this.block) {
      this.block = document.getElementById("answer_" + this.query_id);
    }
    return !!this.block;
  }

  create_block() {
    let html = TEMPLATE_ANS.replaceAll("{{ID}}", this.query_id);
    ANSWERS_BLOCK.insertAdjacentHTML("beforeend", html);
    this.block = document.getElementById("answer_" + this.query_id);

    let self = this;
    this.block.addEventListener("click", (e) => {
      let element = e.target;
      if (!element) return;
      if (element.hasClass("prev")) {
        self.prev();
      } else if (element.hasClass("next")) {
        self.next();
      }
    });
  }

  set(response) {
    if (!response) {
      this.block_items.querySelector(".message").addClass("active");
      return;
    }

    this.remove_empty_message();

    if (!this.messages) {
      this.messages = response.map(
        (options) => new AnswerMessage(this.block_items, options)
      );
      this.messages.sort((a, b) =>
        a.node_id < b.node_id ? -1 : a.node_id > b.node_id ? 1 : 0
      );
      if (!this.messages.find((item) => item.active)) {
        this.activate_message(0);
        this.block_navigator_counter.innerHTML = "1/" + response.length;
      }
    }

    response.forEach((options) => {
      let i = this.messages.findIndex(
        (message) => message.node_id == options.node_id
      );
      if (i === undefined) {
        console.error("Can't find message", options);
        return;
      }
      this.messages[i].set(options);
    });

    if (!this.statuses) {
      this.statuses = response.map(
        (options) => new AnswerStatus(this.block, options)
      );
    }

    response.forEach((options) => {
      let i = this.statuses.findIndex(
        (status) => status.node_id == options.node_id
      );
      if (i === undefined) {
        console.error("Can't find status", options);
        return;
      }
      this.statuses[i].set(options);
    });
  }

  remove_empty_message() {
    let empty_block = this.block.querySelectorAll(
      '.message[node-id="null"]'
    )[0];
    if (empty_block) empty_block.remove();
  }

  activate_message(index) {
    if (index >= this.messages.length) {
      index = 0;
    } else if (index < 0) {
      index = this.messages.length - 1;
    }
    this.active_index = index;
    this.block_navigator_counter.innerHTML =
      index + 1 + "/" + this.messages.length;
    this.messages.forEach((message, i) => {
      if (index == i) {
        message.activate();
      } else {
        message.deactivate();
      }
    });
  }

  prev() {
    if (this.messages.length <= 1) return;
    this.activate_message(this.active_index - 1);
  }
  next() {
    if (this.messages.length <= 1) return;
    this.activate_message(this.active_index + 1);
  }
}

class AnswerMessage {
  parent;
  block;
  node_id;
  active;
  status_block;
  value_block;

  constructor(parent, options) {
    this.parent = parent;
    this.node_id = options.node_id;

    this.create_if_not_exists();
    this.active = this.block.hasClass("active");
    this.status_block = this.block.querySelector(".status span");
    this.value_block = this.block.querySelector(".value");
  }

  activate() {
    if (this.active) return;
    this.active = true;
    this.block.addClass("active");
  }
  deactivate() {
    if (!this.active) return;
    this.active = false;
    this.block.removeClass("active");
  }
  set(options) {
    if (this.status_block.innerHTML != options.status) {
      this.status_block.innerHTML = options.status;
    }
    if (this.value_block.innerHTML != options.message) {
      this.value_block.innerHTML = options.message;
    }
  }

  create_if_not_exists() {
    if (this.block) return;
    this.block = this.parent.querySelector(
      '.message[node-id="' + this.node_id + '"]'
    );
    if (this.block) return;

    this.parent.insertAdjacentHTML("beforeend", TEMPLATE_ANS_MESSAGE);
    let messages = this.parent.querySelectorAll(".message");
    this.block = messages[messages.length - 1];
    this.block.querySelector(".name span").innerHTML = this.node_id.substring(
      0,
      7
    );
    this.block.attr("node-id", this.node_id);
  }
}
class AnswerStatus {
  parent;
  block;
  node_id;
  status_block;

  constructor(parent, options) {
    this.parent = parent;
    this.node_id = options.node_id;

    this.create_if_not_exists();

    this.status_block = this.block.querySelector(".status");
  }
  set(options) {
    if (options.finished) {
      return this.block.addClass("hide");
    }
    if (this.status_block.innerHTML != options.status) {
      this.status_block.innerHTML = options.status;
    }
  }

  create_if_not_exists() {
    if (this.block) return;

    this.block = this.parent.querySelector(
      '.gl-status[node-id="' + this.node_id + '"]'
    );
    if (this.block) return;

    this.parent.insertAdjacentHTML("beforeend", TEMPLATE_ANS_STATUS);
    let status_blocks = this.parent.querySelectorAll(".gl-status");
    this.block = status_blocks[status_blocks.length - 1];
    this.block.attr("node-id", this.node_id);
    this.block.querySelector(".node_id").innerHTML = this.node_id.substring(
      0,
      7
    );
  }

  delete() {
    this.parent
      .querySelector('.gl-status[node-id="' + this.node_id + '"]')
      .remove();
  }
}
