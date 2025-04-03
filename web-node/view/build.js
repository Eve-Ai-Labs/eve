const glob = require("glob");
const fs = require("fs");

let datetime = new Date();
datetime =
  "" +
  datetime.getFullYear() +
  datetime.getMonth() +
  datetime.getDate() +
  datetime.getHours() +
  datetime.getMinutes() +
  datetime.getSeconds();

let files = glob.sync("dist/**/*.{js,mjs,map,css,html}");

files.forEach((path) => {
  fs.readFile(path, "utf8", function (err, data) {
    if (err) {
      console.error(err);
      return;
    }
    data = data.replace(/\?b=\d+/g, "?b=" + datetime);
    if (path.endsWith(".html")) {
      data = data.replace(/\<link\s+rel=\"stylesheet".*/g, "");
      data = data.replace("/scripts/index.js", "bundle.js");
      console.warn(path);
    }
    fs.writeFile(path, data, "utf8", function (err) {
      if (err) return console.error(err);
    });
  });
});
