import React from 'react';

import { Container, } from 'semantic-ui-react'
import { Controller, View } from '@polkadot/joy-utils/index'
import { ITransport } from '../transport'
import { Opening } from "@joystream/types/hiring"
import {
  Applications, OpeningApplication,
  CurrentRoles, ActiveRole, ActiveRoleWithCTAs,
} from './MyRoles'

type State = {
  applications: OpeningApplication[]
  currentCurationRoles: ActiveRoleWithCTAs[]
  myAddress: string
}

const newEmptyState = (): State => {
  return {
    applications: [],
    currentCurationRoles: [],
    myAddress: "",
  }
}

export class MyRolesController extends Controller<State, ITransport> {
  constructor(transport: ITransport, myAddress: string | undefined, initialState: State = newEmptyState()) {
    super(transport, initialState)

    if (typeof myAddress == "string") {
      this.state.myAddress = myAddress
      this.updateCurationGroupRoles(myAddress)
    }
  }

  protected updateApplications(apps: OpeningApplication[]) {
    this.state.applications = apps
    this.dispatch()
  }

  protected async updateCurationGroupRoles(myAddress: string) {
    const roles = await this.transport.myCurationGroupRoles(myAddress)
    this.state.currentCurationRoles = roles.map(role => ({
      ...role,
      CTAs: [
        {
          title: "Leave role",
          callback: (rationale: string) => { this.leaveCurationRole(role, rationale) },
        }
      ],
    })
    )
    this.dispatch()
  }

  leaveCurationRole(role: ActiveRole, rationale: string) {
    this.transport.leaveCurationRole(this.state.myAddress, role.curatorId.toNumber(), rationale)
  }

  cancelApplication(opening: Opening) {
    // TODO
  }
}

export const MyRolesView = View<MyRolesController, State>(
  (state, controller) => (
    <Container className="my-roles">
      <CurrentRoles currentRoles={state.currentCurationRoles} />
      <Applications applications={state.applications} cancelCallback={controller.cancelApplication} />
    </Container>
  )
)
