var path = require('path');
var express = require('express');
var webpack = require('webpack');
var config = require('./webpack.dev.config');
var history = require('connect-history-api-fallback');
var proxy = require('http-proxy-middleware');

var app = express();
var compiler = webpack(config);

var proxiedRoutes = [
      '^/api/',
      '^/token',
      '^/authorize',
      '^/login',
      '^/register',
      '^/$'
    ];
    
var filter = function(path, req) {
  var i, len = proxiedRoutes.length;
  for(i = 0; i < len; i++) {
    if(path.match(proxiedRoutes[i])) {
      return true;
    }
  }
  return false;
};

app.use(proxy(filter, {
  target: 'http://localhost:8080', 
  changeOrigin: false,
  xfwd: true,
}));

//app.use(history());

app.use(require('webpack-dev-middleware')(compiler, {
  noInfo: true,
  publicPath: config.output.publicPath
}));

app.use(require('webpack-hot-middleware')(compiler));

app.listen(3000, 'localhost', (err) => {
  if (err) {
    console.log(err);
    return;
  }

  console.log('Listening at http://localhost:3000');
});
