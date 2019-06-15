const { discover } = require('@joystream/discovery')
const debug = require('debug')('joystream:api:discovery');

const MAX_CACHE_AGE = 30 * 60 * 1000;
const USE_CACHE = true;

module.exports = function(config, runtime)
{
  var doc = {
    // parameters for all operations in this path
    parameters: [
      {
        name: 'id',
        in: 'path',
        required: true,
        description: 'Actor accouuntId',
        schema: {
          type: 'string',
        },
      },
    ],

    // Resolve Service Information
    get: async function(req, res)
    {
        const id = req.params.id;

        // todo - validate id before querying

        try {
          debug(`resolving ${id}`);
          const info = await discover.discover(id, runtime, USE_CACHE, MAX_CACHE_AGE);
          if (info == null) {
            debug('info not found');
            res.status(404).end();
          } else {
            res.status(200).send(info);
          }

        } catch (err) {
          debug(`Error: ${err}`);
          res.status(400).end()
        }
    }
  };

    // OpenAPI specs
    doc.get.apiDoc = {
        description: 'Resolve Service Information',
        operationId: 'discover',
        //tags: ['asset', 'data'],
        responses: {
            200: {
                description: 'Wrapped JSON Service Information',
                content: {
                  'application/json': {
                    schema: {
                      required: ['serialized'],
                      properties: {
                        'serialized': {
                          type: 'string'
                        },
                        'signature': {
                          type: 'string'
                        }
                      },
                    },
                  }
                }
            }
        }
    }

    return doc;
};
