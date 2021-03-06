const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const { CheckerPlugin } = require('awesome-typescript-loader');

const build_dir = __dirname + "/build";

module.exports = function (env) {
  if (!env.rpi_url) {
    console.log("Please pass --env.rpi_url=http://localhost:1337/!");
    process.exit();
  }
  return {
    entry: "./src/index.tsx",
    output: {
      filename: "bundle.js",
      path: build_dir
    },
    stats: "errors-only",

    // Enable sourcemaps for debugging webpack's output.
    devtool: "source-map",

    resolve: {
      // Add '.ts' and '.tsx' as resolvable extensions.
      extensions: [".ts", ".tsx", ".js", ".json"]
    },

    plugins: [
      new HtmlWebpackPlugin({
        filename: build_dir + '/GM.html',
        template: 'src/react-template.ejs',
        rpi_url: env.rpi_url,
        pt_app: 'GM',
      }),
      new HtmlWebpackPlugin({
        filename: build_dir + '/Player.html',
        template: 'src/react-template.ejs',
        rpi_url: env.rpi_url,
        pt_app: 'Player',
      }),
      new webpack.DllReferencePlugin({
        context: ".",
        manifest: require("./build/vendor-manifest.json")
      }),
      new webpack.optimize.ModuleConcatenationPlugin(),
      new CheckerPlugin(),
    ],
    module: {
      rules: [
        // All files with a '.ts' or '.tsx' extension will be handled by 'awesome-typescript-loader'.
        { test: /\.tsx?$/, loader: "awesome-typescript-loader" },

        // All output '.js' files will have any sourcemaps re-processed by 'source-map-loader'.
        { enforce: "pre", test: /\.js$/, loader: "source-map-loader" }
      ]
    },

    devServer: {
      contentBase: build_dir,
    }

    // When importing a module whose path matches one of the following, just
    // assume a corresponding global variable exists and use that instead.
    // This is important because it allows us to avoid bundling all of our
    // dependencies, which allows browsers to cache those libraries between builds.
    // externals: {
    //     "react": "React",
    //     "react-dom": "ReactDOM"
    // },
  };
}
