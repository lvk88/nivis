const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: './src/index.js',
  output: {
    filename: 'main.js',
    path: path.resolve(__dirname, 'dist'),
  },
  mode: 'development',
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "src/**/*",
          to: "[name][ext]"
        }
      ],
    }),
  ],
};
