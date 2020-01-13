import React from 'react';

import { Controller, memoize, View } from '@polkadot/joy-utils/index'

import { ITransport } from '../transport'

import {
  WorkingGroupOpening,
  OpeningError,
  OpeningView,
} from './Opportunities'

type State = {
  blockTime?: number,
  opportunity?: WorkingGroupOpening,
}

export class OpportunityController extends Controller<State, ITransport> {
  constructor(transport: ITransport, initialState: State = {}) {
    super(transport, initialState)
    this.getBlocktime()
  }

  @memoize()
  async getOpportunity(id: string | undefined) {
    if (typeof id === "undefined") {
      return this.onError("ApplyController: no ID provided in params")
    }

    this.state.opportunity = await this.transport.opening(id)
    this.dispatch()
  }

  async getBlocktime() {
    this.state.blockTime = await this.transport.expectedBlockTime()
    this.dispatch()
  }
}

export const OpportunityView = View<OpportunityController, State>({
  errorComponent: OpeningError,
  render: (state, controller, params) => {
    controller.getOpportunity(params.get("id"))
    return (
      <OpeningView {...state.opportunity!} block_time_in_seconds={state.blockTime!} />
    )
  }
})

