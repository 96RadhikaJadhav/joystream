import React, { useState } from 'react'
import { number, withKnobs } from '@storybook/addon-knobs'
import { Card, Container, Message } from 'semantic-ui-react'

import { u128, GenericAccountId } from '@polkadot/types'

import { 
    ProgressStepsView, ProgressStepsProps, ProgressSteps,
    SubmitApplicationStage, SubmitApplicationStageProps,
    DoneStage, DoneStageProps, 
    FundSourceSelector,
} from "./apply"
import {
    OpeningBodyApplicationsStatusProps,
    ApplicationStakeRequirement, RoleStakeRequirement,
} from '../tabs/Opportunities'

import { creator } from "../tabs/Opportunities.stories"

import 'semantic-ui-css/semantic.min.css'
import '@polkadot/joy-roles/index.sass'

export default { 
    title: 'Roles / Components / Apply flow / Elements',
    decorators: [withKnobs],
}

const applicationSliderOptions = {
    range: true,
    min: 0,
    max: 20,
    step: 1,
}

const moneySliderOptions = {
    range: true,
    min: 0,
    max: 1000000,
    step: 500,
}

const applications:OpeningBodyApplicationsStatusProps = {
    application_count: number("Applications count", 0, applicationSliderOptions, "Role rationing policy"),
    application_max: number("Application max", 0, applicationSliderOptions, "Role rationing policy"),
    application_stake: new ApplicationStakeRequirement(
                            new u128(number("Application stake", 500, moneySliderOptions, "Role stakes")), 
                       ),
    role_stake: new RoleStakeRequirement(
                            new u128(number("Role stake", 0, moneySliderOptions, "Role stakes")), 
                       ),
}

type TestProps = {
    _description: string
}

export function ProgressIndicator() {

    const permutations:(ProgressStepsProps & TestProps)[] = [
        {
            _description: "Two steps, first active",
            activeStep: ProgressSteps.SubmitApplication,
            hasConfirmStep: false,
        },
        {
            _description: "Three steps, first active",
            activeStep: ProgressSteps.ConfirmStakes,
            hasConfirmStep: true,
        },
        {
            _description: "Three steps, second active",
            activeStep: ProgressSteps.SubmitApplication,
            hasConfirmStep: true,
        },
    ]

    return (
        <Container>
            {permutations.map((permutation, key) => (
                <Container className="outer" key={key}>
                    <h4>{permutation._description}</h4>
                    <Card fluid>
                        <ProgressStepsView {...permutation} />
                    </Card>
                </Container>
            ))}
        </Container>
    )
}

export function FundSourceSelectorFragment() {
    const [address, setAddress] = useState<AccountId>()
    const [passphrase, setPassphrase] = useState("")

    const props = {
        transactionFee: new u128(number("Transaction fee", 500, moneySliderOptions, "Application Tx")), 
        keypairs: [
            {
                shortName: "KP1",
                accountId: new GenericAccountId('5HZ6GtaeyxagLynPryM7ZnmLzoWFePKuDrkb4AT8rT4pU1fp'),
                balance: new u128(23342),
            },
            {
                shortName: "KP2",
                accountId: new GenericAccountId('5DQqNWRFPruFs9YKheVMqxUbqoXeMzAWfVfcJgzuia7NA3D3'),
                balance: new u128(993342),
            },
            {
                shortName: "KP3",
                accountId: new GenericAccountId('5DBaczGTDhcHgwsZzNE5qW15GrQxxdyros4pYkcKrSUovFQ9'),
                balance: new u128(242),
            },
        ],
    }

    return (
        <Container className="apply-flow">
            <Card fluid>
                <Card.Content>
                    <FundSourceSelector {...props} 
                                        addressCallback={setAddress} 
                                        passphraseCallback={setPassphrase} 
                    />
                </Card.Content>
            </Card>
            <Message info>
                <p>Address: {address ? address.toString(): 'not set'}</p>
                <p>Passphrase: {passphrase}</p>
            </Message>
        </Container>
    )
}

export function StageBSubmitApplication() {
    const props: SubmitApplicationStageProps = {
        nextTransition: () => {},
        applications: applications,
        creator: creator,
        transactionFee: new u128(number("Transaction fee", 500, moneySliderOptions, "Application Tx")), 
        transactionDetails: new Map<string, string>([
            ["Extrinsic hash", "0xae6d24d4d55020c645ddfe2e8d0faf93b1c0c9879f9bf2c439fb6514c6d1292e"],
            ["SOmething else", "abc123"],
        ]),
        keypairs: [
            {
                shortName: "KP1",
                accountId: new GenericAccountId('5HZ6GtaeyxagLynPryM7ZnmLzoWFePKuDrkb4AT8rT4pU1fp'),
                balance: new u128(23342),
            },
            {
                shortName: "KP2",
                accountId: new GenericAccountId('5F5SwL7zwfdDN4UifacVrYKQVVYzoNcoDoGzmhVkaPN2ef8F'),
                balance: new u128(993342),
            },
            {
                shortName: "KP3",
                accountId: new GenericAccountId('5HmMiZSGnidr3AhUk7hhZa7wJrvYyKEiT8cneyavA1ALkfJc'),
                balance: new u128(242),
            },
        ],
    }

    return (
        <Container className="apply-flow">
            <Card fluid>
                <Card.Content>
                    <SubmitApplicationStage {...props} />
                </Card.Content>
            </Card>
        </Container>
    )
}

export function StageCDone() {
    const props: DoneStageProps = {
        applications: applications,
        roleKeyName: "NEW_ROLE_KEY",
    }

    return (
        <Container className="apply-flow">
            <Card fluid>
                <DoneStage {...props} />
            </Card>
        </Container>
    )
}
