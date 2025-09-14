const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, '..', 'dist', 'website'),
        filename: 'index.js',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'index.html'
        }),
        new WasmPackPlugin({
            crateDirectory: __dirname
        }),
        new CopyWebpackPlugin({
            patterns: [{ from: 'assets', to: 'assets' }]
        }),
    ],
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    }
};
