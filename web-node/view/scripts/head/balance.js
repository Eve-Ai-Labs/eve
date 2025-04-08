const ITEM_VALUE = document.querySelector("header .balance .value"),
  EVENTNAME_WASM_INIT = "wasm.init";

let LOCK = false;

export async function init() {
  document.querySelector("header .balance").addEventListener("click", (e) => {
    e.stopPropagation();
    update_balance();
  });
  window.on(EVENTNAME_WASM_INIT, async (e) => {
    await update_balance();
  });

  update_balance();
  setInterval(update_balance, 60000);
}

async function update_balance() {
  if (LOCK) return;

  if (!window.eve_client) {
    ITEM_VALUE.innerHTML = 0;
    LOCK = false;
    return;
  }
  try {
    let balance = await window.eve_client.balance();
    ITEM_VALUE.innerHTML = balance;
    console.log("Balance: ", balance);
  } finally {
    LOCK = false;
  }
}
