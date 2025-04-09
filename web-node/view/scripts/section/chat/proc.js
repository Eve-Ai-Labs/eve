import { sleep } from "/scripts/lib.js?b=00000000000000";

const EVENTNAME_CHAT_CLEAR = "chat.clear",
  EVENTNAME_CHAT_CALL_ASK = "chat.call.ask",
  EVENTNAME_CHAT_ASK_START = "chat.ask.start",
  EVENTNAME_CHAT_ASK_RESPONSE = "chat.ask.response",
  EVENTNAME_CHAT_ASK_FINISHED = "chat.ask.finished";

let TRACKING = [];

window.on(EVENTNAME_CHAT_CALL_ASK, async (e) => {
  let value = e.detail.trim();
  if (!value) {
    return console.cn_error("The question cannot be empty");
  }

  let query_id = await self.eve_client.ask(value);
  TRACKING.push(query_id);

  window.trigger(EVENTNAME_CHAT_ASK_START, {
    id: query_id,
    text: value,
  });
});

window.on(EVENTNAME_CHAT_CLEAR, async (e) => {
  TRACKING = [];
});

async function tracking_requests() {
  for (let index in TRACKING) {
    let query_id = TRACKING[index];
    let response = (await self.eve_client.status(query_id)).response;
    let finished = response.filter(
      (res) => res.Verified || res.Timeout || res.Error
    ).length;

    if (!TRACKING.length) break;

    if (finished === response.length) {
      TRACKING[index] = null;
      window.trigger(EVENTNAME_CHAT_ASK_FINISHED, {
        id: query_id,
        response: response.map(parse_response),
      });
      continue;
    }
    window.trigger(EVENTNAME_CHAT_ASK_RESPONSE, {
      id: query_id,
      response: response.map(parse_response),
    });
  }

  TRACKING = TRACKING.filter((e) => e);
  await sleep(1000);
  tracking_requests();
}
tracking_requests();
function parse_response(response) {
  if (response.SentRequest) {
    return {
      node_id: response.SentRequest,
      finished: false,
      status: "Send request",
      message: "The request has been sent",
    };
  } else if (response.Error) {
    return {
      node_id: response.Error[0],
      finished: true,
      status: "Error",
      message: response.Error[1],
    };
  } else if (response.Timeout) {
    return {
      node_id: response.Timeout.SentRequest,
      finished: true,
      status: "Time out",
      message: "Time out",
    };
  } else if (response.NodeResponse) {
    let node_response = response.NodeResponse.node_response;
    return {
      node_id: node_response.pubkey,
      finished: false,
      status: "Node response",
      message: node_response.response,
    };
  } else if (response.Verified) {
    let verified_response = response.Verified.result;

    let node_response = verified_response.material.node_response;
    return {
      // verified
      description: verified_response.description,
      inspector: verified_response.inspector,
      relevance: verified_response.relevance,
      // node
      node_id: node_response.pubkey,
      finished: true,
      status: "finished",
      message: node_response.response,
    };
  } else {
    console.error(response);
    throw new Error("Can't parse response");
  }
}
