const path = require("path");
const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const output = path.resolve(__dirname, "dist");

module.exports = {
  // mode: "development",
  mode: "production",
  entry: ["./scripts/index.js", "./styles/index.css"],
  output: {
    filename: "bundle.js",
    path: output,
  },
  plugins: [
    new CleanWebpackPlugin(),
    new CopyWebpackPlugin({
      patterns: [
        {
          from: path.resolve(__dirname, "scripts/wonnx/"),
          to: path.resolve(__dirname, "dist/scripts/wonnx/"),
        },
        {
          from: path.resolve(__dirname, "scripts/transformers"),
          to: path.resolve(__dirname, "dist/scripts/transformers"),
        },
        {
          from: path.resolve(__dirname, "index.html"),
          to: output + "/index.html",
        },
      ],
    }),
  ],
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
