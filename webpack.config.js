const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  entry: {
    index: "./js/index.js",
  },
  module: {
    rules: [
      {
        test: /worker\.js$/,
        use: { loader: "worker-loader" },
      }
    ],
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    contentBase: dist,
  },
  plugins: [
    new CopyPlugin({
      patterns: [{
        from: path.resolve(__dirname, "static"),
        to: dist
      }]
    }),
    new WasmPackPlugin({ crateDirectory: __dirname }),
  ]
};
