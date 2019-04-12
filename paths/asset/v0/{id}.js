/*
 * This file is part of the storage node for the Joystream project.
 * Copyright (C) 2019 Joystream Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
'use strict';

const fswalk = require('joystream/util/fs/walk');

const debug = require('debug')('joystream:api:asset');

module.exports = function(config, storage)
{
  var doc = {
    // parameters for all operations in this path
    parameters: [
      {
        name: 'id',
        in: 'path',
        required: true,
        description: 'Repository ID',
        schema: {
          type: 'string',
        },
      },
    ],

    // Get content
    get: function(req, res, _next)
    {
      const id = req.params.id;

      const repo = storage.get(id);
      if (!repo) {
        res.status(404).send({ message: `Repository with id "${id}" not found.` });
        return;
      }

      var contents = [];

      fswalk('/', repo.archive, (err, relname, stat) => {
        if (err) {
          res.status(500).send(err);
          return;
        }

        if (!relname) {
          res.status(200).send({ contents: contents });
          return; // done
        }

        if (stat.isDirectory()) {
          // Hide, because we shouldn't be supporting them anwyay.
          return;
        }

        contents.push({
          name: relname,
          size: stat.size,
        });
      });
    }
  };

  // OpenAPI specs
  doc.get.apiDoc =
  {
    description: 'List assets.',
    operationId: 'assetList',
    tags: ['asset', 'data'],
    responses: {
      200: {
        description: 'Asset list.',
        content: {
          'application/json': {
            schema: {
              required: ['contents'],
              properties: {
                'contents': {
                  type: 'array',
                  items: {
                    required: ['name', 'size'],
                    properties: {
                      'name': {
                        type: 'string',
                      },
                      'size': {
                        type: 'integer',
                        format: 'int64',
                      },
                    },
                  },
                },
              },
            },
          },
        },
      },
      default: {
        description: 'Unexpected error',
        content: {
          'application/json': {
            schema: {
              '$ref': '#/components/schemas/Error'
            },
          },
        },
      },
    },
  };

  return doc;
};


