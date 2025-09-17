const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require('copy-webpack-plugin');
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');

module.exports = {
    entry: {
        home: './pages/home/index.js',
        playground: './pages/playground/index.js',
    },
    output: {
        path: path.resolve(__dirname, '..', 'dist', 'website'),
        library: 'Website',
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: 'pages/home/index.html',
            chunks: ["home"],
        }),
        new HtmlWebpackPlugin({
            template: 'pages/playground/index.html',
            filename: 'playground/index.html',
            chunks: ["playground"],
        }),
        new WasmPackPlugin({
            crateDirectory: __dirname + "/compiler-wasm"
        }),
        new CopyWebpackPlugin({
            patterns: [
                { from: 'assets', to: 'assets' },
                { from: '../book/book', to: 'book' },
            ]
        }),
        new MonacoWebpackPlugin({
            languages: ['kdl', 'rust']
        }),
    ],
    module: {
        rules: [
            {
                test: /\.css$/,
                use: ['style-loader', 'css-loader']
            },
            {
                test: /\.ttf$/,
                type: 'asset/resource'
            }
        ]
    },
    mode: 'development',
    experiments: {
        asyncWebAssembly: true
    }
};
