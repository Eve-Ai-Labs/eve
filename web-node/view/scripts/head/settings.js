import "/scripts/lib.js?b=00000000000000";

const MODAL_ID = "dialog_settings",
  TEMPLATE_DIALOG = `<div id="${MODAL_ID}" class="modal">
    <div class="bg"></div>
    <div class="content">
        <button type="button" class="close"><svg class="close-icon"><use xlink:href="#iclose"></use></svg></button>
        <div class="menu-toolbar">
          <label class="btn import-button" title="import a private key" >
            <svg viewBox="0 0 1920 1920" xmlns="http://www.w3.org/2000/svg"><path d="m807.186 686.592 272.864 272.864H0v112.94h1080.05l-272.864 272.978 79.736 79.849 409.296-409.183-409.296-409.184-79.736 79.736ZM1870.419 434.69l-329.221-329.11C1509.688 74.07 1465.979 56 1421.48 56H451.773v730.612h112.94V168.941h790.584v451.762h451.762v1129.405H564.714v-508.233h-112.94v621.173H1920V554.52c0-45.176-17.619-87.754-49.58-119.83Zm-402.181-242.37 315.443 315.442h-315.443V192.319Z" fill-rule="evenodd"/></svg>
            Import
            <input name="import-key" type="file" title="import a settings file" />
          </label>
          <a href="#" class="btn export-button" title="export a private key">
            <svg viewBox="0 0 1920 1920" xmlns="http://www.w3.org/2000/svg">
                <path d="m0 1016.081 409.186 409.073 79.85-79.736-272.867-272.979h1136.415V959.611H216.169l272.866-272.866-79.85-79.85L0 1016.082ZM1465.592 305.32l315.445 315.445h-315.445V305.32Zm402.184 242.372-329.224-329.11C1507.042 187.07 1463.334 169 1418.835 169h-743.83v677.647h112.94V281.941h564.706v451.765h451.765v903.53H787.946V1185.47H675.003v564.705h1242.353V667.522c0-44.498-18.07-88.207-49.581-119.83Z" fill-rule="evenodd"/>
            </svg>
            Export
          </a>
        </div>
        <div class="header"><div class="title">Settings</div></div>
        <div class="bd">
            <div class="row private_key">
              <label class="row">
                <span class="label">Key</span>
                <input type="text" name="key" value="{{KEY}}" placeholder="Example: 0x....12345" />
                <button type="button" class="generate-button" title="generate a new private key" ><svg version="1.1"  xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 66.459 66.46" xml:space="preserve">
                    <g><path d="M65.542,11.777L33.467,0.037c-0.133-0.049-0.283-0.049-0.42,0L0.916,11.748c-0.242,0.088-0.402,0.32-0.402,0.576 l0.09,40.484c0,0.25,0.152,0.475,0.385,0.566l31.047,12.399v0.072c0,0.203,0.102,0.393,0.27,0.508 c0.168,0.111,0.379,0.135,0.57,0.062l0.385-0.154l0.385,0.154c0.072,0.028,0.15,0.045,0.227,0.045c0.121,0,0.24-0.037,0.344-0.105 c0.168-0.115,0.27-0.305,0.27-0.508v-0.072l31.047-12.399c0.232-0.093,0.385-0.316,0.385-0.568l0.027-40.453 C65.943,12.095,65.784,11.867,65.542,11.777z M32.035,63.134L3.052,51.562V15.013l28.982,11.572L32.035,63.134L32.035,63.134z M33.259,24.439L4.783,13.066l28.48-10.498l28.735,10.394L33.259,24.439z M63.465,51.562L34.484,63.134V26.585l28.981-11.572 V51.562z M14.478,38.021c0-1.692,1.35-2.528,3.016-1.867c1.665,0.663,3.016,2.573,3.016,4.269 c-0.001,1.692-1.351,2.529-3.017,1.867C15.827,41.626,14.477,39.714,14.478,38.021z M5.998,25.375c0-1.693,1.351-2.529,3.017-1.866 c1.666,0.662,3.016,2.572,3.016,4.267c0,1.695-1.351,2.529-3.017,1.867C7.347,28.979,5.998,27.069,5.998,25.375z M22.959,32.124 c0-1.694,1.351-2.53,3.017-1.867c1.666,0.663,3.016,2.573,3.016,4.267c0,1.695-1.352,2.53-3.017,1.867 C24.309,35.728,22.959,33.818,22.959,32.124z M5.995,43.103c0.001-1.692,1.351-2.529,3.017-1.867 c1.666,0.664,3.016,2.573,3.016,4.269c0,1.694-1.351,2.53-3.017,1.867C7.344,46.709,5.995,44.797,5.995,43.103z M22.957,49.853 c0.001-1.695,1.351-2.529,3.017-1.867s3.016,2.572,3.016,4.269c0,1.692-1.351,2.528-3.017,1.866 C24.306,53.458,22.957,51.546,22.957,49.853z M27.81,12.711c-0.766,1.228-3.209,2.087-5.462,1.917 c-2.253-0.169-3.46-1.301-2.695-2.528c0.765-1.227,3.207-2.085,5.461-1.916C27.365,10.352,28.573,11.484,27.81,12.711z M43.928,13.921c-0.764,1.229-3.208,2.086-5.46,1.917c-2.255-0.169-3.46-1.302-2.696-2.528c0.764-1.229,3.209-2.086,5.462-1.918 C43.485,11.563,44.693,12.695,43.928,13.921z M47.04,42.328c-1.041-1.278-0.764-3.705,0.619-5.421 c1.381-1.716,3.344-2.069,4.381-0.792c1.041,1.276,0.764,3.704-0.617,5.42S48.079,43.604,47.04,42.328z"/></g>
                    </svg></button>
              </label>
            </div>
        </div>
        <div class="footer">
            <button type="button" class="cancel">cancel</button>
            <button type="button" class="submit-button">save</button>
        </div>
        <div class="preloader"></div>
    </div>
</div>`,
  EVENTNAME_SETTINGS_OPEN_CALL = "settings.call.open",
  EVENTNAME_SETTINGS_CHANGED_STORAGE = "settings.changed.storage",
  EVENTNAME_AI_START = "ai.start",
  EVENTNAME_AI_STOP = "ai.stop";

let SETTINGS_BUTTON;

export async function init() {
  SETTINGS_BUTTON = document.querySelector("#nav-settings button");
  if (!SETTINGS_BUTTON) {
    return console.error("Settings button not found");
  }

  window.on(EVENTNAME_AI_START, () => {
    SETTINGS_BUTTON.disabled = true;
  });
  window.on(EVENTNAME_AI_STOP, () => {
    SETTINGS_BUTTON.disabled = false;
  });

  window.on(EVENTNAME_SETTINGS_OPEN_CALL, (e) => {
    open(e.detail?.key);
  });

  SETTINGS_BUTTON.addEventListener("click", async (e) => {
    e.stopPropagation();

    await open().catch(console.cn_error);
  });
  SETTINGS_BUTTON.disabled = false;
}

async function open(key) {
  if (!key) {
    key = key ? key : (await window.eve_settings.get()).private_key;
  }

  if (document.getElementById(MODAL_ID)) {
    console.warn("The window is already open");
    return;
  }
  key = !key ? "" : key;

  let html = TEMPLATE_DIALOG.replaceAll("{{KEY}}", key);
  document.body.insertAdjacentHTML("beforeend", html);

  let dialog = document.getElementById(MODAL_ID);
  dialog.querySelector("button.submit-button").disabled = true;

  let input_key = dialog.querySelector(".private_key input");
  input_key.focus();

  dialog.addEventListener("click", (e) => {
    if (e.target.tagName !== "INPUT" && e.target.tagName !== "SELECT")
      input_key.focus();
  });

  dialog.querySelector(".generate-button").addEventListener("click", (e) => {
    e.stopPropagation();
    let key_input = dialog.querySelector('input[name="key"]');
    key_input.value = genRanHex(64);
    key_input.change();
  });

  return Promise.all([
    init_event_close(dialog),
    init_event_input(dialog),
    init_button_save(dialog),
    init_import_key(dialog),
    init_export_key(dialog),
  ]);
}

async function init_event_close(dialog) {
  dialog.querySelectorAll("button.close, button.cancel").forEach((button) => {
    button.addEventListener("click", (e) => {
      e.stopPropagation();
      dialog.remove();
    });
  });

  function dialog_close(e) {
    if (e.key === "Escape" || e.keyCode === 27) {
      e.stopPropagation();

      dialog.remove();

      window.removeEventListener("keyup", dialog_close);
    }
  }

  window.addEventListener("keyup", dialog_close);
}

async function init_event_input(dialog) {
  let button_submit = dialog.querySelector("button.submit-button"),
    input_key = dialog.querySelector('input[name="key"]');

  function on_changed(e) {
    let success = dialog_validate_proc(input_key, button_submit);

    if (success && (e.keyCode == 10 || e.keyCode == 13) && e.ctrlKey) {
      e.stopPropagation();
      button_submit.click();
    }
  }

  input_key.addEventListener("keyup", on_changed);
  input_key.addEventListener("change", on_changed);
}

async function init_button_save(dialog) {
  let button_submit = dialog.querySelector("button.submit-button"),
    input_key = dialog.querySelector('input[name="key"]');
  input_key.value = input_key.value.toLowerCase().trim();

  button_submit.addEventListener("click", async (e) => {
    e.stopPropagation();

    if (!dialog_validate_proc(input_key, button_submit)) {
      return;
    }

    await window.eve_settings
      .set_private_key(input_key.value)
      .then((message) => {
        console.cn_info(message);
        window.trigger(EVENTNAME_SETTINGS_CHANGED_STORAGE, {}, true);
      })
      .catch((e) => {
        console.cn_error("Error saving settings: ", e);
      });
    dialog.remove();
  });
}

async function init_import_key(dialog) {
  let input_import_key = dialog.querySelector("input[name='import-key']");
  input_import_key.addEventListener("change", async function (e) {
    let file = this.files[0];
    let reader = new FileReader();
    reader.readAsText(file);

    dialog.addClass("load");

    reader.onerror = function () {
      dialog.removeClass("load");
      console.cn_error(reader.error);
    };
    reader.onload = function () {
      dialog.removeClass("load");
      let key = reader.result.toLowerCase().trim();

      if (!/^(0x)?[\da-f]{64}$/i.test(key)) {
        console.cn_error("The private key is invalid");
        return;
      }
      input_import_key.value = "";
      let input = dialog.querySelector('input[name="key"]');
      input.value = key;
      input.change();
    };
  });
}

async function init_export_key(dialog) {
  dialog
    .querySelector(".export-button")
    .addEventListener("click", async function (e) {
      let private_key = dialog
        .querySelector('input[name="key"]')
        .value.toLowerCase()
        .trim();
      if (!private_key) {
        console.cn_error("The private key is not set");
        return;
      }
      this.attr("download", "private_key.txt");
      this.attr("target", "_blank");
      let taBlob = new Blob([private_key], { type: "text/plain" });
      this.setAttribute("href", URL.createObjectURL(taBlob));
      setTimeout(() => {
        URL.revokeObjectURL(this.href), 5000;
        this.attr("download", null);
        this.attr("target", null);
        this.attr("href", null);
      });
    });
}

function dialog_validate_proc(input_key, button) {
  let error_key = !/^(0x)?[\da-f]{64}$/i.test(input_key.value);

  [[input_key, error_key]].forEach(async (v) => {
    let list = v[0].classList;
    list[v[1] ? "add" : "remove"]("error");
    list[!v[1] ? "add" : "remove"]("success");
  });
  button.disabled = error_key;
  return !error_key;
}

function genRanHex(size) {
  return [...Array(size)]
    .map(() => Math.floor(Math.random() * 16).toString(16))
    .join("");
}
