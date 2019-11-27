import React from 'react'
import { number, text, withKnobs } from '@storybook/addon-knobs'
import { Card, Container } from 'semantic-ui-react'

import { u128 } from '@polkadot/types'

import {
  openingClass,
  OpeningBodyApplicationsStatus, OpeningBodyApplicationsStatusProps,
  OpeningBodyReviewInProgress,
  OpeningBodyStakeRequirement, StakeRequirementProps,
  OpeningHeader,
  ApplicationStakeRequirement, RoleStakeRequirement, StakeType,
} from "./Opportunities"
import { tomorrow, yesterday } from "./Opportunities.stories"

import { OpeningStageClassification, OpeningState } from "../classifiers"

import 'semantic-ui-css/semantic.min.css'
import '@polkadot/joy-roles/index.sass'

export default {
  title: 'Roles / Components / Opportunities groups tab / Elements',
  decorators: [withKnobs],
}

type TestProps = {
  _description: string
}

export function OpeningHeaderByState() {
  const stages: OpeningStageClassification[] = [
    {
      uri: "https://some.url/#1",
      state: OpeningState.WaitingToBegin,
      starting_block: 2956498,
      starting_block_hash: "somehash",
      created_time: yesterday(),
    },
    {
      uri: "https://some.url/#1",
      state: OpeningState.AcceptingApplications,
      starting_block: 2956498,
      starting_block_hash: "somehash",
      created_time: yesterday(),
    },
    {
      uri: "https://some.url/#1",
      state: OpeningState.InReview,
      starting_block: 102456,
      starting_block_hash: "somehash",
      created_time: yesterday(),
    },
    {
      uri: "https://some.url/#1",
      state: OpeningState.Complete,
      starting_block: 10345,
      starting_block_hash: "somehash",
      created_time: yesterday(),
    },
    {
      uri: "https://some.url/#1",
      state: OpeningState.Cancelled,
      starting_block: 104,
      starting_block_hash: "somehash",
      created_time: yesterday(),
    },
  ]

  return (
    <Container>
      {stages.map((stage, key) => (
        <Container className={"inner opening " + openingClass(stage.state)} key={key}>
          <Card fluid className="container">
            <Card.Content className="header">
              <OpeningHeader stage={stage} />
            </Card.Content>
          </Card>
        </Container>
      ))}
    </Container>
  )
}

export function OpeningApplicationsStatusByState() {
  const permutations: (OpeningBodyApplicationsStatusProps & TestProps)[] = [
    {
      _description: "No limit, no applications, no stake",
      numberOfApplications: 0,
      maxNumberOfApplications: 0,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "No limit, some applications, no stake",
      numberOfApplications: 15,
      maxNumberOfApplications: 0,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, no applications, no stake",
      numberOfApplications: 0,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, some applications, no stake",
      numberOfApplications: 10,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, full applications, no stake (application impossible)",
      numberOfApplications: 20,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "No limit, no applications, some stake",
      numberOfApplications: 0,
      maxNumberOfApplications: 0,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(10)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "No limit, some applications, some stake",
      numberOfApplications: 15,
      maxNumberOfApplications: 0,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(10)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, no applications, some stake",
      numberOfApplications: 0,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(10)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, some applications, some stake",
      numberOfApplications: 10,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(10)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
    {
      _description: "Limit, full applications, some stake",
      numberOfApplications: 20,
      maxNumberOfApplications: 20,
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(10)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
    },
  ]

  return (
    <Container>
      {permutations.map((permutation, key) => (
        <Container className="outer opening" key={key}>
          <h4>{permutation._description}</h4>
          <Container className="main">
            <OpeningBodyApplicationsStatus {...permutation} />
          </Container>
        </Container>
      ))}
    </Container>
  )
}

export function OpeningApplicationsStakeRequirementByStake() {
  const permutations: (StakeRequirementProps & TestProps)[] = [
    {
      _description: "No stakes required (should be empty)",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: "App stake required; no role stake required",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(500)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: "App stake required >; no role stake required",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(500), StakeType.AtLeast),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: "No app stake required; role stake required",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0)),
      requiredRoleStake: new RoleStakeRequirement(new u128(101)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: "No app stake required; role stake required",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(0), StakeType.AtLeast),
      requiredRoleStake: new RoleStakeRequirement(new u128(102)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: ">= App stake required; role stake required",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(101), StakeType.AtLeast),
      requiredRoleStake: new RoleStakeRequirement(new u128(102)),
      defactoMinimumStake: new u128(0),
      maxNumberOfApplications: 0,
    },
    {
      _description: "App stake required; no role stake required; dynamic minimum > 0",
      requiredApplicationStake: new ApplicationStakeRequirement(new u128(500)),
      requiredRoleStake: new RoleStakeRequirement(new u128(0)),
      defactoMinimumStake: new u128(1000),
      maxNumberOfApplications: 20,
    },
  ]

  return (
    <Container>
      {permutations.map((permutation, key) => (
        <Container className="outer opening" key={key}>
          <h4>{permutation._description}</h4>
          <Card fluid>
            <Card.Content>
              <Container className="main">
                <OpeningBodyStakeRequirement {...permutation} />
              </Container>
            </Card.Content>
          </Card>
        </Container>
      ))}
    </Container>
  )
}

export function ReviewInProgress() {
  const permutations: (OpeningStageClassification & TestProps)[] = [
    {
      _description: "Standard control",
      review_end_time: tomorrow(),
      review_end_block: 1000000,
    },
  ]

  return (
    <Container>
      {permutations.map((permutation, key) => (
        <Container className="outer opening" key={key}>
          <h4>{permutation._description}</h4>
          <Card fluid>
            <Card.Content>
              <Container className="main">
                <OpeningBodyReviewInProgress {...permutation} />
              </Container>
            </Card.Content>
          </Card>
        </Container>
      ))}
    </Container>
  )
}

