import React from 'react';

import { Controller, View } from '@polkadot/joy-utils/index'

import { ITransport } from '../transport'

import {
  WorkingGroupOpening,
  OpeningsView,
} from './Opportunities'

type State = {
  blockTime?: number,
  opportunities?: Array<WorkingGroupOpening>,
}

export class OpportunitiesController extends Controller<State, ITransport> {
  constructor(transport: ITransport, initialState: State = {}) {
    super(transport, initialState)
    this.getOpportunities()
    this.getBlocktime()
  }

  getOpportunities() {
    this.transport.currentOpportunities().then(value => {
      this.state.opportunities = value
      this.dispatch()
    })
  }

  getBlocktime() {
    this.transport.expectedBlockTime().then(value => {
      this.state.blockTime = value
      this.dispatch()
    })
  }
}

export const OpportunitiesView = View<OpportunitiesController, State>(
  (state) => (
    <OpeningsView
      openings={state.opportunities}
      block_time_in_seconds={state.blockTime}
    />
  )
)
