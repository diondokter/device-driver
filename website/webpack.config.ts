import path from 'node:path';
import { fileURLToPath } from "url";
import webpack from "webpack";
import HtmlWebpackPlugin from 'html-webpack-plugin';
import WasmPackPlugin from "@wasm-tool/wasm-pack-plugin";
import CopyWebpackPlugin from 'copy-webpack-plugin';
import MonacoWebpackPlugin from 'monaco-editor-webpack-plugin';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const config: webpack.Configuration = {
    entry: {
        home: './pages/home/index.ts',
        playground: './pages/playground/index.ts',
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
            crateDirectory: path.resolve(__dirname, "../compiler/dd-wasm"),
            watchDirectories: [
                path.resolve(__dirname, "../compiler")
            ],
            outDir: path.resolve(__dirname, "pkg"),
            outName: "device_driver_wasm"
        }),
        new CopyWebpackPlugin({
            patterns: [
                { from: 'assets', to: 'assets' },
                { from: '../book/book', to: 'book' },
            ]
        }),
        new MonacoWebpackPlugin({
            languages: ['rust']
        }),
    ],
    module: {
        rules: [
            {
                test: /\.css$/,
                use: ['style-loader', 'css-loader']
            },
            {
                test: /\.tsx?$/,
                use: "ts-loader",
                exclude: /node_modules/,
            },
        ]
    },
    resolve: {
        extensions: [".tsx", ".ts", ".js"],
    },
    watchOptions: {
        aggregateTimeout: 1000, // Delays the rebuild slightly to let WasmPack finish
    },
    mode: 'development',
    devtool: 'inline-source-map',
    experiments: {
        asyncWebAssembly: true
    }
};

export default config;
