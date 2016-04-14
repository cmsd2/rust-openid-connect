var ExtractTextPlugin = require("extract-text-webpack-plugin");
var CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = {
  devtool: "source-map",
  entry: {
    "app": [
      "./web/css/app.scss",
      "bootstrap/js/bootstrap.js",
      "bootstrap/css/bootstrap.css", 
      "./web/js/app.js"
    ],
  },

  output: {
    path: "./priv/",
    filename: "js/app.js"
  },

  resolve: {
    alias: {
      sinon: __dirname + '/node_modules/sinon/pkg/sinon',
      bootstrap: __dirname + '/node_modules/bootstrap/dist',
      openid_connect: __dirname + "/web/js"
    }
  },

  module: {
    loaders: [
      {
        test: /sinon\.js$/,
        loader: 'imports?define=>false,require=>false'
      },
      {
        test: /\.js$/,
        exclude: /node_modules/,
        loader: "babel",
        query: {
          presets: ['es2015', 'react']
        }
      },
      {
        test: /\.css$/,
        loader: ExtractTextPlugin.extract("style", "css")
      },
      {
        test: /\.scss$/,
        loader: ExtractTextPlugin.extract(
          "style",
          "css!sass?includePaths[]=" + __dirname +  "/node_modules"
        )
      },
      {
        test: /\.(jpe|jpg|woff|woff2|eot|ttf|svg)(\?.*$|$)/, 
        loader: 'url-loader?importLoaders=1&limit=100000' 
      },      
    ]
  },

  plugins: [
    new ExtractTextPlugin("css/app.css"),
    new CopyWebpackPlugin([
      { from: "./web/assets" },
    ])
  ],

  externals: {
    "jsdom": "window",
    "cheerio": "window",
    'react/lib/ExecutionEnvironment': true,
    'react/lib/ReactContext': 'window',
    'text-encoding': 'window'
  },
}
