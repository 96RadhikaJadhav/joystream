import Transport from "./transport";
import { ApiProps } from "@polkadot/react-api/types";
import { ApiPromise } from "@polkadot/api";

export default class SubstrateTransport extends Transport {
  protected api: ApiPromise;

  constructor(api: ApiProps) {
    super();

    if (!api) {
      throw new Error("Cannot create SubstrateTransport: A Substrate API is required");
    } else if (!api.isApiReady) {
      throw new Error("Cannot create a SubstrateTransport: The Substrate API is not ready yet.");
    }

    this.api = api.api;
  }

  engine() {
    return this.api.query.proposalEngine;
  }

  codex() {
    return this.api.query.codex;
  }

  allProposals() {
    return this.engine();
  }
}
