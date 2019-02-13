'use strict';

// Node requires
const fs = require('fs');
const path = require('path');

// npm requires
const express = require('express');
const openapi = require('express-openapi');
const bodyParser = require('body-parser');
const cors = require('cors');
const yaml = require('js-yaml');

// Project root
const project_root = path.resolve(__dirname, '..');

// Allow require relative to project root
require.main.paths.push(path.resolve(project_root, 'lib'))

// Project requires
const validateResponses = require.main.require('middleware/validate_responses');
const fileUploads = require.main.require('middleware/file_uploads');
const pagination = require.main.require('util/pagination');

// Configure app
const app = express();
app.use(cors());
app.use(bodyParser.json());
// FIXME app.use(bodyParser.urlencoded({ extended: true }));

// Load & extend/configure API docs
var api = yaml.safeLoad(fs.readFileSync(
  path.resolve(project_root, 'api-base.yml')));
api['x-express-openapi-additional-middleware'] = [validateResponses];
api['x-express-openapi-validation-strict'] = true;

api = pagination.openapi(api);

openapi.initialize({
  apiDoc: api,
  app: app,
  paths: path.resolve(project_root, 'paths'),
  docsPath: '/swagger.json',
  consumesMiddleware: {
    'multipart/form-data': fileUploads
  },
});

// If no other handler gets triggered (errors), respond with the
// error serialized to JSON.
app.use(function(err, req, res, next) {
  res.status(err.status).json(err);
});

module.exports = app;
