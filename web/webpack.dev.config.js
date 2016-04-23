var path = require('path');
var ExtractTextPlugin = require("extract-text-webpack-plugin");
var CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = {
  devtool: "inline-source-map",
  entry: {
    "app": [
      "bootstrap/css/bootstrap.css", 
      "./src/js/app.js",
      "./src/css/app.scss"
    ],
  },

  output: {
    path: path.join(__dirname, 'dist'),
    filename: "js/app.js"
  },
  
  plugins: [
    new ExtractTextPlugin("css/app.css"),
    new CopyWebpackPlugin([
      { from: path.join(__dirname, 'src/assets') },
    ])
  ],

  resolve: {
    alias: {
      sinon: path.join(__dirname, 'node_modules/sinon/pkg/sinon'),
      bootstrap: path.join(__dirname, 'node_modules/bootstrap/dist'),
      jquery: path.join(__dirname, 'node_modules/jquery/dist'),
      openid_connect: path.join(__dirname, 'src/js')
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

  externals: {
    "jsdom": "window",
    "cheerio": "window",
    'react/lib/ExecutionEnvironment': true,
    'react/lib/ReactContext': 'window',
    'text-encoding': 'window'
  },
}
