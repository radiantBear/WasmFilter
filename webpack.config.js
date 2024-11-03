const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
    mode: "production",
    entry: {
        index: "./ts/bootstrap.ts"
    },
    devtool: 'source-map',
    output: {
        path: dist,
        filename: "[name].js"
    },
    devServer: {
        static: {
            directory: dist,
        }
    },
    experiments: {
        asyncWebAssembly: true
    },
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: path.resolve(__dirname, "static"), to: dist }
            ]
        }),

        new WasmPackPlugin({
            crateDirectory: __dirname,
        }),
    ],
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    }
};