const path = require("path");

module.exports = {
  mode: "development",
  entry: ["./scripts/index.js", "./styles/index.css"],
  output: {
    filename: "bundle.js",
    path: path.resolve(__dirname, "dist"),
  },
  module: {
    rules: [
      {
        test: /\.(scss|css)$/,
        use: [
          "style-loader", // 3. Инжектит стили в DOM
          "css-loader", // 2. Преобразует CSS в CommonJS
          "sass-loader", // 1. Компилирует SCSS в CSS
        ],
      },
    ],
  },
};
