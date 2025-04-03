const ITEM_VALUE = document.querySelector("header .balance .value"),
  EVENTNAME_SETTINGS_CHANGED = "settings.changed";

let LOCK = false;

export async function init() {
  document.querySelector("header .balance").addEventListener("click", (e) => {
    e.stopPropagation();
    update_balance();
  });
  window.on(EVENTNAME_SETTINGS_CHANGED, async (e) => {
    await update_balance();
  });

  update_balance();
  setInterval(update_balance, 60000);
}

async function update_balance() {
  if (LOCK) return;
  try {
    let balance = await window.web_node.balance();
    ITEM_VALUE.innerHTML = balance;
    console.log("Balance: ", balance);
  } finally {
    LOCK = false;
  }
}
