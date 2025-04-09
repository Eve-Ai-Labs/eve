import "/scripts/lib.js?b=00000000000000";

const DEFAULT_TAB = "web-node";

export async function init() {
  document.querySelectorAll("button.tablink").forEach((button) => {
    button.addEventListener("click", () => set_tab(button.value));
  });
  set_tab();
}

function set_tab(tab) {
  tab = tab ? tab : localStorage.getItem("active-tab");
  tab = tab ? tab : DEFAULT_TAB;

  let body = document.getElementById(tab);
  if (!body) {
    tab = DEFAULT_TAB;
    body = document.getElementById(tab);
    if (!body) {
      console.error("Failed to find default tab.");
    }
  }

  document.querySelectorAll(".tab_body").forEach((tab_body) => {
    tab_body.attr("id") == tab
      ? tab_body.addClass("active").removeClass("hide")
      : tab_body.removeClass("active").addClass("hide");
  });
  document.querySelectorAll("button.tablink").forEach((button) => {
    button.value == tab
      ? button.addClass("active")
      : button.removeClass("active");

    button.disabled = false;
  });

  document.querySelectorAll("button.tablink").forEach((button) => {});

  localStorage.setItem("active-tab", tab);
}
