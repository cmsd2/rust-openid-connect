require('!!script!jquery/jquery.js');
require("bootstrap/js/bootstrap.js");

import React from 'react';
import ReactDOM from 'react-dom';
import App from './components/app';

$(document).ready(function() {
  ReactDOM.render(
    <App />,
    document.getElementById('root')
  );
});
