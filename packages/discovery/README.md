# Discovery

The `@joystream/discovery` package provides an API for role services to publish
discovery information about themselves, and for consumers to resolve this
information.

In the Joystream network, services are provided by having members stake for a
role. The role is identified by a unique actor key. Resolving service information
associated with the actor key is the main purpose of this module.

This implementation is based on [IPNS](https://docs.ipfs.io/guides/concepts/ipns/)
as well as runtime information.

## Discovery Workflow

The discovery workflow provides an actor key to the `discover()` function, which
will eventuall return structured data.

Clients can verify that the structured data has been signed by the identifying
actor. This is normally done automatically, unless a `verify: false` option is
passed to `discover()`. Then, a separate `verify()` call can be used for
verification.

Under the hood, `discover()` uses any known participating node in the discovery
network. If no other nodes are known, the bootstrap nodes from the runtime are
used.

There is a distinction in the discovery workflow:

1. If run in the browser environment, a HTTP request to a participating node
  is performed to discover nodes.
2. If run in a node.js process, instead:
  - A trusted (local) IPFS node must be configured.
  - The chain is queried to resolve an actor key to an IPNS peer ID.
  - The trusted IPFS node is used to resolve the IPNS peer ID to an IPFS
    file.
  - The IPFS file is fetched; this contains the structured data.

Web services providing the HTTP endpoint used in the first approach will
themselves use the second approach for fulfilling queries.

## Publishing Workflow

The publishing workflow is a little more involved, and requires more interaction
with the runtime and the trusted IPFS node.

1. A service information file is created.
1. The file is signed with the actor key (see below).
1. The file is published on IPFS.
1. The IPNS name of the trusted IPFS node is updated to refer to the published
   file.
1. The runtime mapping from the actor ID to the IPNS name is updated.

## Published Information

Any JSON data can theoretically be published with this system; however, the
following structure is currently imposed:

- The JSON must be an Object at the top-level, not an Array.
- Each key must correspond to a service spec (below).

The data is signed using the [@joystream/json-signing](../json-signing/README.md)
package.

## Service Info Specifications

Service specifications are JSON Objects, not Arrays. All service specifications
come with their own `version` field which should be intepreted by clients making
use of the information.

Additionally, some services may only provide an `endpoint` value, as defined
here:

* `version`: A numeric version identifier for the service info field.
* `endpoint`: A publicly accessible base URL for a service API.

The `endpoint` should include a scheme and full authority, such that appending
`swagger.json` to the path resolves the OpenAPI definition of the API served
at this endpoint.

The OpenAPI definition must include a top level path component corresponding
to the service name, followed by an API version component. The remainder of the
provided paths are dependent on the specific version of the API provided.

For example, for an endpoint value of `https://user:password@host:port/` the
following must hold:

- `https://user:password@host:port/swagger.json` resolves to the OpenAPI
  definition of the API(s) provided by this endpoint.
- The OpenAPI definitions include paths prefixed by
  `https://user:password@host:port/XXX/vYYY` where
  - `XXX` is the service name, identical to the field name of the service.
  - `YYY` the version identifier for the published service API.

### Discovery Service

Publishes `version` and `endpoint` as above; the `version` field is currently
always `1`.

### Asset Service

Publishes `version` and `endpoint` as above; the `version` field is currently
always `1`.

### Example

```json
{
  "asset": {
    "version": 1,
    "endpoint": "https://foo.bar/asset/v0"
  },
  "discovery": {
    "version": 1,
    "endpoint": "http://quux.io/discovery/v0"
  },
}
```
