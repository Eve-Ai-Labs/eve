fetch("/icons.html?b=00000000000000")
  .then((response) => response.text())
  .then((html) => document.body.insertAdjacentHTML("beforeend", html));
