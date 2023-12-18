const CopyWebpackPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const TsconfigPathsPlugin = require('tsconfig-paths-webpack-plugin');

const path = require('path');

module.exports = {
  entry: './src/bootstrap.ts',
  devtool: 'inline-source-map',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      }
    ]
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    plugins: [new TsconfigPathsPlugin({ configFile: 'tsconfig.json' })],
  },
  output: {
    filename: 'main.js',
    path: path.resolve(__dirname, 'dist'),
  },
  mode: 'development',
  plugins: [
    new CopyWebpackPlugin({
      patterns: [
        {
          from: "src/index.html",
          to: "index.html"
        }
      ],
    }),
    new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, '../'),

                // Check https://rustwasm.github.io/wasm-pack/book/commands/build.html for
                // the available set of arguments.
                //
                // Optional space delimited arguments to appear before the wasm-pack
                // command. Default arguments are `--verbose`.
                args: '--log-level warn',
                // Default arguments are `--typescript --target browser --mode normal`.
                //extraArgs: '--profiling',

                // Optional array of absolute paths to directories, changes to which
                // will trigger the build.
                // watchDirectories: [
                //   path.resolve(__dirname, "another-crate/src")
                // ],

                // The same as the `--out-dir` option for `wasm-pack`
                outDir: "pkg",

                // The same as the `--out-name` option for `wasm-pack`
                outName: "mywasm",

                // If defined, `forceWatch` will force activate/deactivate watch mode for
                // `.rs` files.
                //
                // The default (not set) aligns watch mode for `.rs` files to Webpack's
                // watch mode.
                // forceWatch: true,

                // If defined, `forceMode` will force the compilation mode for `wasm-pack`
                //
                // Possible values are `development` and `production`.
                //
                // the mode `development` makes `wasm-pack` build in `debug` mode.
                // the mode `production` makes `wasm-pack` build in `release` mode.
                // forceMode: "development",

                // Controls plugin output verbosity, either 'info' or 'error'.
                // Defaults to 'info'.
                // pluginLogLevel: 'info'
            }),
  ],
  experiments:{
    asyncWebAssembly: true
  }
};
