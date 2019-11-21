import React, { useState } from 'react'
import { number, withKnobs } from '@storybook/addon-knobs'
import { Card, Container, Message } from 'semantic-ui-react'

import { u128, GenericAccountId } from '@polkadot/types'

import { 
	ConfirmStakesStage, ConfirmStakesStageProps,
    ProgressStepsView, ProgressStepsProps, ProgressSteps,
    SubmitApplicationStage, SubmitApplicationStageProps,
    DoneStage, DoneStageProps, 
    FundSourceSelector,
	StakeRankSelector, StakeRankSelectorProps,
	ConfirmStakes2Up, ConfirmStakes2UpProps,
} from "./apply"
import {
    OpeningBodyApplicationsStatusProps,
    ApplicationStakeRequirement, RoleStakeRequirement,
	StakeType, 
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

export function StakeRankSelectorFragment() {
	const [stake, setStake] = useState<Balance>(new u128(0))

	// List of the minimum stake required to beat each rank
	const slots: Balance[] = []
	for (let i = 0; i < 10; i++) {
		slots.push(new u128((i*100)+10+i+1))
	}

    const props: StakeRankSelectorProps = {
		minStake: new u128(10),
		stake: stake,
		setStake: setStake,
		slots: slots,
		step: new u128(10),
    }

    return (
        <Container className="apply-flow">
            <Card fluid>
				<Message info>
					<StakeRankSelector {...props} />
				</Message>
            </Card>
			<Message warning>
				Slots: {JSON.stringify(slots)}<br />
				Stake: {stake.toString()}
			</Message>
        </Container>
    )
}

export function Select2MinimumStakes() {
	// List of the minimum stake required to beat each rank
	const slots: Balance[] = []
	for (let i = 0; i < 20; i++) {
		slots.push(new u128((i*100)+10+i+1))
	}


	const props: ConfirmStakes2UpProps & TestProps =  {
		_description: "One fixed stake (application), no limit",
		application_stake: new ApplicationStakeRequirement(new u128(1)),
		role_stake: new RoleStakeRequirement(new u128(2)),
		application_max: 0,
		application_count: 0,
		dynamic_minimum: new u128(0),
		step: new u128(5),
        slots: slots,
	}

	return (
        <Container className="apply-flow">
            <Card fluid>
				<Card.Content>
					<ConfirmStakes2Up {...props} />
				</Card.Content>
			</Card>
		</Container>
	)
}

export function StageAConfirmStakes() {
	const permutations:(ConfirmStakesStageProps & TestProps)[] = [
        {
            _description: "One fixed stake (application), no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(0)),
			application_max: 0,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One fixed stake (role), no limit",
			application_stake: new ApplicationStakeRequirement(new u128(0)),
			role_stake: new RoleStakeRequirement(new u128(1213)),
			application_max: 0,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Two fixed stakes, no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(10)),
			application_max: 0,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One fixed stake (application), 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(0)),
			application_max: 20,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One fixed stake (role), 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(456)),
			role_stake: new RoleStakeRequirement(new u128(0)),
			application_max: 20,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Two fixed stakes, 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(10)),
			application_max: 20,
			application_count: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One minimum stake (application), no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(0)),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One minimum stake (role), no limit",
			application_stake: new ApplicationStakeRequirement(new u128(0)),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Two minimum stakes, no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Minimum application stake, fixed role stake, no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(10)),
			application_max: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Minimum role stake, fixed application stake, no limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 0,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One minimum stake (application), 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(0)),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "One minimum stake (role), 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(0)),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Two minimum stakes, 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Minimum application stake, fixed role stake, 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10), StakeType.AtLeast),
			role_stake: new RoleStakeRequirement(new u128(10)),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
        {
            _description: "Minimum role stake, fixed application stake, 20 applicant limit",
			application_stake: new ApplicationStakeRequirement(new u128(10)),
			role_stake: new RoleStakeRequirement(new u128(10), StakeType.AtLeast),
			application_max: 0,
			application_count: 20,
			dynamic_minimum: new u128(0),
			nextTransition: () => {},
        },
    ]

	const keypairs = [
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
	]

	const [stake, setStake] = useState<Balance>(new u128(0))

	// List of the minimum stake required to beat each rank
	const slots: Balance[] = []
	for (let i = 0; i < 20; i++) {
		slots.push(new u128((i*100)+10+i+1))
	}

    const stakeRankSelectorProps: StakeRankSelectorProps = {
		minStake: new u128(10),
		stake: stake,
		setStake: setStake,
		slots: slots,
		step: new u128(10),
    }

    return (
        <Container className="apply-flow">
            {permutations.map((permutation, key) => (
                <Container className="outer" key={key}>
                    <h4>{key}. {permutation._description}</h4>
                    <Card fluid>
                        <ConfirmStakesStage {...permutation} {...stakeRankSelectorProps} keypairs={keypairs} />
                    </Card>
                </Container>
            ))}
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
