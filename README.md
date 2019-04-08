Storage Nodes for Joystream
===========================

Installation
------------

This project uses [yarn](https://yarnpkg.com/) as Node package manager.

```bash
$ yarn run build
```

The command will run `yarn install`, perform post-install fixes and build
TypeScript files.

To make the `js_storage` script available globally, run:

```bash
$ npm install -g
```

This might be best if used within e.g. an [nvm](https://github.com/creationix/nvm)
environment.

Development
-----------

Run a development server:

```bash
$ yarn run dev
```

Run tests:
```bash
$ yarn run test
```

Run linter:
```bash
$ yarn run lint
```

TypeScript files are used for type declarations for interaction with the chain.
They are taken from [the app](https://github.com/Joystream/apps), and renamed
to suit our use:

`apps/packages/joy-members/src/types.ts` becomes `lib/joystream/types/members.ts`,
etc. `lib/joystream/types/index.js` is manually maintained, but other JavaScript
files in that folder are generated.

Command-Line
------------

Running a storage server is as easy as running the bundled `js_storage`
executable:

```bash
$ js_storage --storage=/path/to/storage/directory
```

Run with `--help` to see a list of available CLI options.

Configuration
-------------

Most common configuration options are available as command-line options
for the CLI.

However, some advanced configuration options are only possible to set
via the configuration file.

* `filter` is a hash of upload filtering options.
  * `max_size` sets the maximum permissible file upload size. If unset,
    this defaults to 100 MiB.
  * `mime` is a hash of...
    * `accept` is an Array of mime types that are acceptable for uploads,
      such as `text/plain`, etc. Mime types can also be specified for
      wildcard matching, such as `video/*`.
    * `reject` is an Array of mime types that are unacceptable for uploads.

Upload Filtering
----------------

The upload filtering logic first tests whether any of the `accept` mime types
are matched. If none are matched, the upload is rejected. If any is matched,
then the upload is still rejected if any of the `reject` mime types are
matched.

This allows inclusive and exclusive filtering.

* `{ accept: ['text/plain', 'text/html'] }` accepts *only* the two given mime types.
* `{ accept: ['text/*'], reject: ['text/plain'] }` accepts any `text/*` that is not
  `text/plain`.

More advanced filtering is currently not available.

API Packages
------------

Since it's not entirely clear yet how APIs will develop in future, the approach
taken here is to package individual APIs up individually. That is, instead of
providing an overall API version in `api-base.yml`, it should be part of each
API package's path.

For example, for a `foo` API in its version `v1`, its definitions should live
in `./paths/foo/v1.js` and `./paths/foo/v1/*.js` respectively.

*Note:* until a reasonably stable API is reached, this project uses a `v0`
version prefix.

Interface/implementation
------------------------

For reusability across API versions, it's best to keep files in the `paths`
subfolder very thin, and instead inject implementations via the `dependencies`
configuration value of `express-openapi`.

These implementations line to the `./lib` subfolder. Adjust `server.js` as
needed to make them available to API packages.

Streaming Notes
---------------

For streaming content, it is required that stream metadata is located at the
start of the stream. Most software writes metadata at the end of the stream,
because it is when the stream is committed to disk that the entirety of the
metadata is known.

To move metadata to the start of the stream, a CLI tool such as
[qtfaststart](https://github.com/danielgtaylor/qtfaststart) for MP4 files might
be used.
