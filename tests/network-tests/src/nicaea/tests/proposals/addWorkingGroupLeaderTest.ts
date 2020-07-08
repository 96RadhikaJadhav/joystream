import { KeyringPair } from '@polkadot/keyring/types';
import { membershipTest } from '../impl/membershipCreation';
import { councilTest } from '../impl/electingCouncil';
import { initConfig } from '../../utils/config';
import { Keyring, WsProvider } from '@polkadot/api';
import BN from 'bn.js';
import { setTestTimeout } from '../../utils/setTestTimeout';
import tap from 'tap';
import { registerJoystreamTypes } from '@nicaea/types';
import { closeApi } from '../impl/closeApi';
import { ApiWrapper, WorkingGroups } from '../../utils/apiWrapper';
import {
  createWorkingGroupLeaderOpening,
  voteForProposal,
  beginWorkingGroupLeaderApplicationReview,
  expectLeadOpeningAdded,
  fillLeaderOpeningProposal,
} from './impl/proposalsModule';
import { applyForOpening } from '../workingGroup/impl/workingGroupModule';

tap.mocha.describe('Set lead proposal scenario', async () => {
  initConfig();
  registerJoystreamTypes();

  const m1KeyPairs: KeyringPair[] = new Array();
  const m2KeyPairs: KeyringPair[] = new Array();
  const leadKeyPair: KeyringPair[] = new Array();

  const keyring = new Keyring({ type: 'sr25519' });
  const N: number = +process.env.MEMBERSHIP_CREATION_N!;
  const paidTerms: number = +process.env.MEMBERSHIP_PAID_TERMS!;
  const nodeUrl: string = process.env.NODE_URL!;
  const sudoUri: string = process.env.SUDO_ACCOUNT_URI!;
  const K: number = +process.env.COUNCIL_ELECTION_K!;
  const greaterStake: BN = new BN(+process.env.COUNCIL_STAKE_GREATER_AMOUNT!);
  const lesserStake: BN = new BN(+process.env.COUNCIL_STAKE_LESSER_AMOUNT!);
  const applicationStake: BN = new BN(process.env.WORKING_GROUP_APPLICATION_STAKE!);
  const roleStake: BN = new BN(process.env.WORKING_GROUP_ROLE_STAKE!);
  const durationInBlocks: number = 60;

  const provider = new WsProvider(nodeUrl);
  const apiWrapper: ApiWrapper = await ApiWrapper.create(provider);
  const sudo: KeyringPair = keyring.addFromUri(sudoUri);

  setTestTimeout(apiWrapper, durationInBlocks);
  membershipTest(apiWrapper, m1KeyPairs, keyring, N, paidTerms, sudoUri);
  membershipTest(apiWrapper, m2KeyPairs, keyring, N, paidTerms, sudoUri);
  membershipTest(apiWrapper, leadKeyPair, keyring, 1, paidTerms, sudoUri);
  councilTest(apiWrapper, m1KeyPairs, m2KeyPairs, keyring, K, sudoUri, greaterStake, lesserStake);

  let createOpeningProposalId: BN;
  let openingId: BN;
  tap.test(
    'Create leader opening',
    async () =>
      (createOpeningProposalId = await createWorkingGroupLeaderOpening(
        apiWrapper,
        m1KeyPairs,
        sudo,
        applicationStake,
        roleStake,
        'Storage'
      ))
  );
  tap.test('Approve add opening proposal', async () => {
    voteForProposal(apiWrapper, m2KeyPairs, sudo, createOpeningProposalId);
    openingId = await expectLeadOpeningAdded(apiWrapper);
  });
  let applicationId: BN;
  tap.test(
    'Apply for lead opening',
    async () =>
      await applyForOpening(
        apiWrapper,
        leadKeyPair,
        sudo,
        applicationStake,
        roleStake,
        new BN(openingId),
        WorkingGroups.storageWorkingGroup,
        false
      )
  );
  let beginReviewProposalId: BN;
  tap.test(
    'Begin leader application review',
    async () =>
      (beginReviewProposalId = await beginWorkingGroupLeaderApplicationReview(
        apiWrapper,
        m1KeyPairs,
        sudo,
        new BN(openingId),
        'Storage'
      ))
  );
  tap.test('Approve begin review proposal', async () =>
    voteForProposal(apiWrapper, m2KeyPairs, sudo, beginReviewProposalId)
  );
  let fillLeaderOpeningProposalId: BN;
  tap.test(
    'Fill leader opening',
    async () =>
      (fillLeaderOpeningProposalId = await fillLeaderOpeningProposal(
        apiWrapper,
        m1KeyPairs,
        leadKeyPair[0].address,
        sudo,
        new BN(openingId),
        'Storage'
      ))
  );
  tap.test('Approve fill leaвer opening', async () =>
    voteForProposal(apiWrapper, m2KeyPairs, sudo, fillLeaderOpeningProposalId)
  );

  closeApi(apiWrapper);
});
