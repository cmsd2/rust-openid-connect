var fs = require('fs');
var path = require('path');
var express = require('express');
var webpack = require('webpack');
var config = require('./webpack.dev.config');
var history = require('connect-history-api-fallback');
var proxy = require('http-proxy-middleware');
var http = require('http');
var https = require('https');

var app = express();
var compiler = webpack(config);

var ssl = {
	  key: fs.readFileSync('C:/users/cmsd2/.ssl/certs/server-key.pem', 'utf8'),
    ca: fs.readFileSync('C:/users/cmsd2/.ssl/certs/ca.pem', 'utf8'),
    cert: fs.readFileSync('C:/users/cmsd2/.ssl/certs/server-cert.pem', 'utf8')
  };

var proxiedRoutes = [
      '^/api/',
      '^/token',
      '^/authorize',
      '^/complete',
      '^/login',
      '^/consent',
      '^/register',
      '^/applications',
      '^/grants',
      '^/identity',
      '^/connect',
      '^/.well-known/',
      '^/jwks',
      '^/$'
    ];
    
var filter = function(path, req) {
  var i, len = proxiedRoutes.length;
  console.log(req.method, path);
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
  ssl: ssl
}));

//app.use(history());

app.use(require('webpack-dev-middleware')(compiler, {
  noInfo: true,
  publicPath: config.output.publicPath
}));

app.use(require('webpack-hot-middleware')(compiler));

var httpsServer = https.createServer(ssl, app);

httpsServer.listen(3000, 'localhost', (err) => {
  if (err) {
    console.log(err);
    return;
  }

  console.log('Listening at https://localhost:3000');
});
