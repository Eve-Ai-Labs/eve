HTMLElement.prototype.addClass = function (name) {
  var el = this;
  name
    .split(" ")
    .map(function (name) {
      return name.trim();
    })
    .filter(function (name) {
      return name;
    })
    .forEach(function (name) {
      el.classList.add(name);
    });
  return this;
};
HTMLElement.prototype.removeClass = function (name) {
  var el = this;
  name
    .split(" ")
    .map(function (name) {
      return name.trim();
    })
    .filter(function (name) {
      return name;
    })
    .forEach(function (name) {
      el.classList.remove(name);
    });
  return this;
};
HTMLElement.prototype.hasClass = function (name) {
  return this.classList.contains(name);
};
HTMLElement.prototype.toggleClass = function (name) {
  let el = this;
  name
    .split(" ")
    .map((name) => name.trim())
    .filter((name) => name)
    .forEach((name) => {
      el[el.hasClass(name) ? "removeClass" : "addClass"](name);
    });
  return el;
};
HTMLElement.prototype.parentByClass = function (class_name) {
  for (
    var parent = this;
    parent.parentNode &&
    (parent = parent.parentNode).hasClass !== undefined &&
    !parent.hasClass(class_name);

  );
  return parent !== document ? parent : undefined;
};
HTMLElement.prototype.attr = function (attr_name, attr_value) {
  if (attr_value === undefined) {
    return this.getAttribute(attr_name);
  }
  if (attr_value) {
    this.setAttribute(attr_name, attr_value);
  } else {
    this.removeAttribute(attr_name);
  }
  return this;
};
HTMLElement.prototype.event_run_name = function (name) {
  this.dispatchEvent(new Event(name, { bubbles: true, cancelable: true }));
  return this;
};
HTMLElement.prototype.change = function () {
  this.event_run_name("change");
  return this.event_run_name("onchange");
};
HTMLElement.prototype.click = function () {
  this.event_run_name("onclick");
  return this.event_run_name("click");
};
HTMLElement.prototype.submit = function () {
  this.event_run_name("onsubmit");
  return this.event_run_name("submit");
};

HTMLElement.prototype.trigger = function (name, options, strict) {
  name = Array.isArray(name) ? name : [name];
  name.forEach((name) => {
    let event_p = name.split(".");
    while (event_p.length) {
      window.dispatchEvent(
        new CustomEvent(event_p.join("."), {
          detail: options,
        })
      );
      if (strict) {
        break;
      }
      event_p.pop();
    }
  });
};
window.trigger = HTMLElement.prototype.trigger;
document.trigger = HTMLElement.prototype.trigger;
HTMLElement.prototype.on = function (name, event) {
  Array.isArray(name)
    ? name.forEach((name) => this.addEventListener(name, event))
    : this.addEventListener(name, event);
};
window.on = HTMLElement.prototype.on;
document.on = HTMLElement.prototype.on;

NodeList.prototype.for_each = NodeList.prototype.forEach;
NodeList.prototype.filter = Array.prototype.filter;
NodeList.prototype.map = Array.prototype.map;

export function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
