import { ApiPromise, WsProvider } from '@polkadot/api';
import { Option, Vec, Bytes, u32 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';
import { KeyringPair } from '@polkadot/keyring/types';
import { UserInfo, PaidMembershipTerms, MemberId } from '@nicaea/types/members';
import { Mint, MintId } from '@nicaea/types/mint';
import { Lead, LeadId } from '@nicaea/types/content-working-group';
import { Application, WorkerId, Worker } from '@nicaea/types/working-group';
import { Application as HiringApplication } from '@nicaea/types/hiring';
import { RoleParameters } from '@nicaea/types/roles';
import { Seat } from '@nicaea/types/lib/council';
import { Balance, EventRecord, AccountId, BlockNumber, BalanceOf } from '@polkadot/types/interfaces';
import BN from 'bn.js';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { Sender } from './sender';
import { Utils } from './utils';
import { Stake, StakedState } from '@nicaea/types/stake';
import { RewardRelationship } from '@nicaea/types/recurring-rewards';
import { Opening, ApplicationId } from '@nicaea/types/hiring';

export class ApiWrapper {
  private readonly api: ApiPromise;
  private readonly sender: Sender;

  public static async create(provider: WsProvider): Promise<ApiWrapper> {
    const api = await ApiPromise.create({ provider });
    return new ApiWrapper(api);
  }

  constructor(api: ApiPromise) {
    this.api = api;
    this.sender = new Sender(api);
  }

  public close() {
    this.api.disconnect();
  }

  public async buyMembership(
    account: KeyringPair,
    paidTermsId: number,
    name: string,
    expectFailure = false
  ): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.members.buyMembership(paidTermsId, new UserInfo({ handle: name, avatar_uri: '', about: '' })),
      account,
      expectFailure
    );
  }

  public getMemberIds(address: string): Promise<MemberId[]> {
    return this.api.query.members.memberIdsByControllerAccountId<Vec<MemberId>>(address);
  }

  public getBalance(address: string): Promise<Balance> {
    return this.api.query.balances.freeBalance<Balance>(address);
  }

  public async transferBalance(from: KeyringPair, to: string, amount: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.balances.transfer(to, amount), from);
  }

  public getPaidMembershipTerms(paidTermsId: number): Promise<Option<PaidMembershipTerms>> {
    return this.api.query.members.paidMembershipTermsById<Option<PaidMembershipTerms>>(paidTermsId);
  }

  public getMembershipFee(paidTermsId: number): Promise<BN> {
    return this.getPaidMembershipTerms(paidTermsId).then(terms => terms.unwrap().fee.toBn());
  }

  public async transferBalanceToAccounts(from: KeyringPair, to: KeyringPair[], amount: BN): Promise<void[]> {
    return Promise.all(
      to.map(async keyPair => {
        await this.transferBalance(from, keyPair.address, amount);
      })
    );
  }

  private getBaseTxFee(): BN {
    return this.api.createType('BalanceOf', this.api.consts.transactionPayment.transactionBaseFee).toBn();
  }

  private estimateTxFee(tx: SubmittableExtrinsic<'promise'>): BN {
    const baseFee: BN = this.getBaseTxFee();
    const byteFee: BN = this.api.createType('BalanceOf', this.api.consts.transactionPayment.transactionByteFee).toBn();
    return Utils.calcTxLength(tx).mul(byteFee).add(baseFee);
  }

  public estimateBuyMembershipFee(account: KeyringPair, paidTermsId: number, name: string): BN {
    return this.estimateTxFee(
      this.api.tx.members.buyMembership(paidTermsId, new UserInfo({ handle: name, avatar_uri: '', about: '' }))
    );
  }

  public estimateApplyForCouncilFee(amount: BN): BN {
    return this.estimateTxFee(this.api.tx.councilElection.apply(amount));
  }

  public estimateVoteForCouncilFee(nominee: string, salt: string, stake: BN): BN {
    const hashedVote: string = Utils.hashVote(nominee, salt);
    return this.estimateTxFee(this.api.tx.councilElection.vote(hashedVote, stake));
  }

  public estimateRevealVoteFee(nominee: string, salt: string): BN {
    const hashedVote: string = Utils.hashVote(nominee, salt);
    return this.estimateTxFee(this.api.tx.councilElection.reveal(hashedVote, nominee, salt));
  }

  public estimateProposeRuntimeUpgradeFee(stake: BN, name: string, description: string, runtime: Bytes | string): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createRuntimeUpgradeProposal(stake, name, description, stake, runtime)
    );
  }

  public estimateProposeTextFee(stake: BN, name: string, description: string, text: string): BN {
    return this.estimateTxFee(this.api.tx.proposalsCodex.createTextProposal(stake, name, description, stake, text));
  }

  public estimateProposeSpendingFee(
    title: string,
    description: string,
    stake: BN,
    balance: BN,
    destination: string
  ): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSpendingProposal(stake, title, description, stake, balance, destination)
    );
  }

  public estimateProposeWorkingGroupMintCapacityFee(title: string, description: string, stake: BN, balance: BN): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSetContentWorkingGroupMintCapacityProposal(
        stake,
        title,
        description,
        stake,
        balance
      )
    );
  }

  public estimateProposeValidatorCountFee(title: string, description: string, stake: BN): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSetValidatorCountProposal(stake, title, description, stake, stake)
    );
  }

  public estimateProposeLeadFee(title: string, description: string, stake: BN, address: string): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSetLeadProposal(stake, title, description, stake, { stake, address })
    );
  }

  public estimateProposeEvictStorageProviderFee(title: string, description: string, stake: BN, address: string): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createEvictStorageProviderProposal(stake, title, description, stake, address)
    );
  }

  public estimateProposeStorageRoleParametersFee(
    title: string,
    description: string,
    stake: BN,
    minStake: BN,
    minActors: BN,
    maxActors: BN,
    reward: BN,
    rewardPeriod: BN,
    bondingPeriod: BN,
    unbondingPeriod: BN,
    minServicePeriod: BN,
    startupGracePeriod: BN,
    entryRequestFee: BN
  ): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSetStorageRoleParametersProposal(stake, title, description, stake, [
        minStake,
        minActors,
        maxActors,
        reward,
        rewardPeriod,
        bondingPeriod,
        unbondingPeriod,
        minServicePeriod,
        startupGracePeriod,
        entryRequestFee,
      ])
    );
  }

  public estimateProposeElectionParametersFee(
    title: string,
    description: string,
    stake: BN,
    announcingPeriod: BN,
    votingPeriod: BN,
    revealingPeriod: BN,
    councilSize: BN,
    candidacyLimit: BN,
    newTermDuration: BN,
    minCouncilStake: BN,
    minVotingStake: BN
  ): BN {
    return this.estimateTxFee(
      this.api.tx.proposalsCodex.createSetElectionParametersProposal(stake, title, description, stake, [
        announcingPeriod,
        votingPeriod,
        revealingPeriod,
        councilSize,
        candidacyLimit,
        newTermDuration,
        minCouncilStake,
        minVotingStake,
      ])
    );
  }

  public estimateVoteForProposalFee(): BN {
    return this.estimateTxFee(this.api.tx.proposalsEngine.vote(0, 0, 'Approve'));
  }

  public estimateAddOpeningFee(): BN {
    return this.estimateTxFee(
      this.api.tx.storageWorkingGroup.addOpening(
        'CurrentBlock',
        {
          application_rationing_policy: { max_active_applicants: '32' },
          max_review_period_length: 32,
          application_staking_policy: {
            amount: 0,
            amount_mode: 'AtLeast',
            crowded_out_unstaking_period_length: 0,
            review_period_expired_unstaking_period_length: 0,
          },
          role_staking_policy: {
            amount: 0,
            amount_mode: 'AtLeast',
            crowded_out_unstaking_period_length: 0,
            review_period_expired_unstaking_period_length: 0,
          },
          role_slashing_terms: {
            Slashable: {
              max_count: 0,
              max_percent_pts_per_time: 0,
            },
          },
          fill_opening_successful_applicant_application_stake_unstaking_period: 0,
          fill_opening_failed_applicant_application_stake_unstaking_period: 0,
          fill_opening_failed_applicant_role_stake_unstaking_period: 0,
          terminate_curator_application_stake_unstaking_period: 0,
          terminate_curator_role_stake_unstaking_period: 0,
          exit_curator_role_application_stake_unstaking_period: 0,
          exit_curator_role_stake_unstaking_period: 0,
        },
        'Opening readable text',
        'Worker'
      )
    );
  }

  public estimateAcceptApplicationsFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.acceptApplications(0));
  }

  public estimateApplyOnOpeningFee(account: KeyringPair): BN {
    return this.estimateTxFee(
      this.api.tx.storageWorkingGroup.applyOnOpening(
        0,
        0,
        account.address,
        0,
        0,
        'Some testing text used for estimation purposes which is longer than text expected during the test'
      )
    );
  }

  public estimateBeginApplicantReviewFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.beginApplicantReview(0));
  }

  public estimateFillOpeningFee(): BN {
    return this.estimateTxFee(
      this.api.tx.storageWorkingGroup.fillOpening(0, [0], {
        amount_per_payout: 0,
        next_payment_at_block: 0,
        payout_interval: 0,
      })
    );
  }

  public estimateIncreaseStakeFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.increaseStake(0, 0));
  }

  public estimateDecreaseStakeFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.decreaseStake(0, 0));
  }

  public estimateUpdateRoleAccountFee(address: string): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.updateRoleAccount(0, address));
  }

  public estimateUpdateRewardAccountFee(address: string): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.updateRewardAccount(0, address));
  }

  public estimateLeaveRoleFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.leaveRole(0, 'Long justification text'));
  }

  public estimateWithdrawApplicationFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.withdrawApplication(0));
  }

  public estimateTerminateApplicationFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.terminateApplication(0));
  }

  public estimateSlashStakeFee(): BN {
    return this.estimateTxFee(this.api.tx.storageWorkingGroup.slashStake(0, 0));
  }

  public estimateTerminateRoleFee(): BN {
    return this.estimateTxFee(
      this.api.tx.storageWorkingGroup.terminateRole(
        0,
        'Long justification text explaining why the worker role will be terminated'
      )
    );
  }

  private applyForCouncilElection(account: KeyringPair, amount: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.councilElection.apply(amount), account, false);
  }

  public batchApplyForCouncilElection(accounts: KeyringPair[], amount: BN): Promise<void[]> {
    return Promise.all(
      accounts.map(async keyPair => {
        await this.applyForCouncilElection(keyPair, amount);
      })
    );
  }

  public async getCouncilElectionStake(address: string): Promise<BN> {
    // TODO alter then `applicantStake` type will be introduced
    return this.api.query.councilElection.applicantStakes(address).then(stake => {
      const parsed = JSON.parse(stake.toString());
      return new BN(parsed.new);
    });
  }

  private voteForCouncilMember(account: KeyringPair, nominee: string, salt: string, stake: BN): Promise<void> {
    const hashedVote: string = Utils.hashVote(nominee, salt);
    return this.sender.signAndSend(this.api.tx.councilElection.vote(hashedVote, stake), account, false);
  }

  public batchVoteForCouncilMember(
    accounts: KeyringPair[],
    nominees: KeyringPair[],
    salt: string[],
    stake: BN
  ): Promise<void[]> {
    return Promise.all(
      accounts.map(async (keyPair, index) => {
        await this.voteForCouncilMember(keyPair, nominees[index].address, salt[index], stake);
      })
    );
  }

  private revealVote(account: KeyringPair, commitment: string, nominee: string, salt: string): Promise<void> {
    return this.sender.signAndSend(this.api.tx.councilElection.reveal(commitment, nominee, salt), account, false);
  }

  public batchRevealVote(accounts: KeyringPair[], nominees: KeyringPair[], salt: string[]): Promise<void[]> {
    return Promise.all(
      accounts.map(async (keyPair, index) => {
        const commitment = Utils.hashVote(nominees[index].address, salt[index]);
        await this.revealVote(keyPair, commitment, nominees[index].address, salt[index]);
      })
    );
  }

  // TODO consider using configurable genesis instead
  public sudoStartAnnouncingPerion(sudo: KeyringPair, endsAtBlock: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.councilElection.setStageAnnouncing(endsAtBlock)),
      sudo,
      false
    );
  }

  public sudoStartVotingPerion(sudo: KeyringPair, endsAtBlock: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.councilElection.setStageVoting(endsAtBlock)),
      sudo,
      false
    );
  }

  public sudoStartRevealingPerion(sudo: KeyringPair, endsAtBlock: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.councilElection.setStageRevealing(endsAtBlock)),
      sudo,
      false
    );
  }

  public sudoSetCouncilMintCapacity(sudo: KeyringPair, capacity: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.council.setCouncilMintCapacity(capacity)),
      sudo,
      false
    );
  }

  public sudoSetWorkingGroupMintCapacity(sudo: KeyringPair, capacity: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.storageWorkingGroup.setMintCapacity(capacity)),
      sudo,
      false
    );
  }

  public getBestBlock(): Promise<BN> {
    return this.api.derive.chain.bestNumber();
  }

  public getCouncil(): Promise<Seat[]> {
    return this.api.query.council.activeCouncil<Vec<Codec>>().then(seats => {
      return (seats as unknown) as Seat[];
    });
  }

  public getRuntime(): Promise<Bytes> {
    return this.api.query.substrate.code<Bytes>();
  }

  public async proposeRuntime(
    account: KeyringPair,
    stake: BN,
    name: string,
    description: string,
    runtime: Bytes | string
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createRuntimeUpgradeProposal(memberId, name, description, stake, runtime),
      account,
      false
    );
  }

  public async proposeText(
    account: KeyringPair,
    stake: BN,
    name: string,
    description: string,
    text: string
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createTextProposal(memberId, name, description, stake, text),
      account,
      false
    );
  }

  public async proposeSpending(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    balance: BN,
    destination: string
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSpendingProposal(memberId, title, description, stake, balance, destination),
      account,
      false
    );
  }

  public async proposeWorkingGroupMintCapacity(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    balance: BN
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSetContentWorkingGroupMintCapacityProposal(
        memberId,
        title,
        description,
        stake,
        balance
      ),
      account,
      false
    );
  }

  public async proposeValidatorCount(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    validatorCount: BN
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSetValidatorCountProposal(memberId, title, description, stake, validatorCount),
      account,
      false
    );
  }

  public async proposeLead(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    leadAccount: KeyringPair
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    const leadMemberId: BN = (await this.getMemberIds(leadAccount.address))[0].toBn();
    const addressString: string = leadAccount.address;
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSetLeadProposal(memberId, title, description, stake, [
        leadMemberId,
        addressString,
      ]),
      account,
      false
    );
  }

  public async proposeEvictStorageProvider(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    storageProvider: string
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createEvictStorageProviderProposal(
        memberId,
        title,
        description,
        stake,
        storageProvider
      ),
      account,
      false
    );
  }

  public async proposeStorageRoleParameters(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    minStake: BN,
    minActors: BN,
    maxActors: BN,
    reward: BN,
    rewardPeriod: BN,
    bondingPeriod: BN,
    unbondingPeriod: BN,
    minServicePeriod: BN,
    startupGracePeriod: BN,
    entryRequestFee: BN
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSetStorageRoleParametersProposal(memberId, title, description, stake, [
        minStake,
        minActors,
        maxActors,
        reward,
        rewardPeriod,
        bondingPeriod,
        unbondingPeriod,
        minServicePeriod,
        startupGracePeriod,
        entryRequestFee,
      ]),
      account,
      false
    );
  }

  public async proposeElectionParameters(
    account: KeyringPair,
    title: string,
    description: string,
    stake: BN,
    announcingPeriod: BN,
    votingPeriod: BN,
    revealingPeriod: BN,
    councilSize: BN,
    candidacyLimit: BN,
    newTermDuration: BN,
    minCouncilStake: BN,
    minVotingStake: BN
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.proposalsCodex.createSetElectionParametersProposal(memberId, title, description, stake, [
        announcingPeriod,
        votingPeriod,
        revealingPeriod,
        councilSize,
        candidacyLimit,
        newTermDuration,
        minCouncilStake,
        minVotingStake,
      ]),
      account,
      false
    );
  }

  public approveProposal(account: KeyringPair, memberId: BN, proposal: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.proposalsEngine.vote(memberId, proposal, 'Approve'), account, false);
  }

  public batchApproveProposal(council: KeyringPair[], proposal: BN): Promise<void[]> {
    return Promise.all(
      council.map(async keyPair => {
        const memberId: BN = (await this.getMemberIds(keyPair.address))[0].toBn();
        await this.approveProposal(keyPair, memberId, proposal);
      })
    );
  }

  public getBlockDuration(): BN {
    return this.api.createType('Moment', this.api.consts.babe.expectedBlockTime).toBn();
  }

  public expectProposalCreated(): Promise<BN> {
    return new Promise(async resolve => {
      await this.api.query.system.events<Vec<EventRecord>>(events => {
        events.forEach(record => {
          if (record.event.method && record.event.method.toString() === 'ProposalCreated') {
            resolve(new BN(record.event.data[1].toString()));
          }
        });
      });
    });
  }

  public expectRuntimeUpgraded(): Promise<void> {
    return new Promise(async resolve => {
      await this.api.query.system.events<Vec<EventRecord>>(events => {
        events.forEach(record => {
          if (record.event.method.toString() === 'RuntimeUpdated') {
            resolve();
          }
        });
      });
    });
  }

  public expectProposalFinalized(): Promise<void> {
    return new Promise(async resolve => {
      await this.api.query.system.events<Vec<EventRecord>>(events => {
        events.forEach(record => {
          if (
            record.event.method &&
            record.event.method.toString() === 'ProposalStatusUpdated' &&
            record.event.data[1].toString().includes('Executed')
          ) {
            resolve();
          }
        });
      });
    });
  }

  public getTotalIssuance(): Promise<BN> {
    return this.api.query.balances.totalIssuance<Balance>();
  }

  public async getRequiredProposalStake(numerator: number, denominator: number): Promise<BN> {
    const issuance: number = await (await this.getTotalIssuance()).toNumber();
    const stake = (issuance * numerator) / denominator;
    return new BN(stake.toFixed(0));
  }

  public getProposalCount(): Promise<BN> {
    return this.api.query.proposalsEngine.proposalCount<u32>();
  }

  public async getWorkingGroupMintCapacity(): Promise<BN> {
    const mintId: MintId = await this.api.query.contentWorkingGroup.mint<MintId>();
    const mintCodec = await this.api.query.minting.mints<Codec[]>(mintId);
    const mint: Mint = (mintCodec[0] as unknown) as Mint;
    return mint.getField<Balance>('capacity');
  }

  public getValidatorCount(): Promise<BN> {
    return this.api.query.staking.validatorCount<u32>();
  }

  public async getCurrentLeadAddress(): Promise<string> {
    const leadId: Option<LeadId> = await this.api.query.contentWorkingGroup.currentLeadId<Option<LeadId>>();
    const leadCodec = await this.api.query.contentWorkingGroup.leadById<Codec[]>(leadId.unwrap());
    const lead = (leadCodec[0] as unknown) as Lead;
    return lead.role_account.toString();
  }

  public async createStorageProvider(account: KeyringPair): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0].toBn();
    await this.sender.signAndSend(this.api.tx.actors.roleEntryRequest('StorageProvider', memberId), account, false);
    await this.sender.signAndSend(this.api.tx.actors.stake('StorageProvider', account.address), account, false);
    return;
  }

  public async isStorageProvider(address: string): Promise<boolean> {
    const storageProviders: Vec<AccountId> = await this.api.query.actors.accountIdsByRole<Vec<AccountId>>(
      'StorageProvider'
    );
    return storageProviders.map(accountId => accountId.toString()).includes(address);
  }

  public async sudoSetLead(sudo: KeyringPair, lead: KeyringPair): Promise<void> {
    const leadMemberId: BN = (await this.getMemberIds(lead.address))[0].toBn();
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.storageWorkingGroup.setLead(leadMemberId, lead.address)),
      sudo,
      false
    );
  }

  public async sudoUnsetLead(sudo: KeyringPair): Promise<void> {
    return this.sender.signAndSend(this.api.tx.sudo.sudo(this.api.tx.storageWorkingGroup.unsetLead()), sudo, false);
  }

  public async addOpening(
    activateAtBlock: BN | undefined,
    account: KeyringPair,
    maxActiveApplicants: BN,
    maxReviewPeriodLength: BN,
    applicationStakingPolicyAmount: BN,
    applicationCrowdedOutUnstakingPeriodLength: BN,
    applicationExpiredUnstakingPeriodLength: BN,
    roleStakingPolicyAmount: BN,
    roleCrowdedOutUnstakingPeriodLength: BN,
    roleExpiredUnstakingPeriodLength: BN,
    slashableMaxCount: BN,
    slashableMaxPercentPtsPerTime: BN,
    successfulApplicantApplicationStakeUnstakingPeriod: BN,
    failedApplicantApplicationStakeUnstakingPeriod: BN,
    failedApplicantRoleStakeUnstakingPeriod: BN,
    terminateCuratorApplicationStakeUnstakingPeriod: BN,
    terminateCuratorRoleStakeUnstakingPeriod: BN,
    exitCuratorRoleApplicationStakeUnstakingPeriod: BN,
    exitCuratorRoleStakeUnstakingPeriod: BN,
    text: string,
    openingType: string
  ): Promise<void> {
    const activateAt = activateAtBlock == undefined ? 'CurrentBlock' : { ExactBlock: activateAtBlock };
    const commitment = {
      application_rationing_policy: { max_active_applicants: maxActiveApplicants },
      max_review_period_length: maxReviewPeriodLength,
      application_staking_policy: {
        amount: applicationStakingPolicyAmount,
        amount_mode: 'AtLeast',
        crowded_out_unstaking_period_length: applicationCrowdedOutUnstakingPeriodLength,
        review_period_expired_unstaking_period_length: applicationExpiredUnstakingPeriodLength,
      },
      role_staking_policy: {
        amount: roleStakingPolicyAmount,
        amount_mode: 'AtLeast',
        crowded_out_unstaking_period_length: roleCrowdedOutUnstakingPeriodLength,
        review_period_expired_unstaking_period_length: roleExpiredUnstakingPeriodLength,
      },
      role_slashing_terms: {
        Slashable: {
          max_count: slashableMaxCount,
          max_percent_pts_per_time: slashableMaxPercentPtsPerTime,
        },
      },
      fill_opening_successful_applicant_application_stake_unstaking_period: successfulApplicantApplicationStakeUnstakingPeriod,
      fill_opening_failed_applicant_application_stake_unstaking_period: failedApplicantApplicationStakeUnstakingPeriod,
      fill_opening_failed_applicant_role_stake_unstaking_period: failedApplicantRoleStakeUnstakingPeriod,
      terminate_curator_application_stake_unstaking_period: terminateCuratorApplicationStakeUnstakingPeriod,
      terminate_curator_role_stake_unstaking_period: terminateCuratorRoleStakeUnstakingPeriod,
      exit_curator_role_application_stake_unstaking_period: exitCuratorRoleApplicationStakeUnstakingPeriod,
      exit_curator_role_stake_unstaking_period: exitCuratorRoleStakeUnstakingPeriod,
    };
    await this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.addOpening(activateAt, commitment, text, openingType),
      account,
      false
    );
  }

  public async sudoAddOpening(
    activateAtBlock: BN | undefined,
    sudo: KeyringPair,
    maxActiveApplicants: BN,
    maxReviewPeriodLength: BN,
    applicationStakingPolicyAmount: BN,
    applicationCrowdedOutUnstakingPeriodLength: BN,
    applicationExpiredUnstakingPeriodLength: BN,
    roleStakingPolicyAmount: BN,
    roleCrowdedOutUnstakingPeriodLength: BN,
    roleExpiredUnstakingPeriodLength: BN,
    slashableMaxCount: BN,
    slashableMaxPercentPtsPerTime: BN,
    successfulApplicantApplicationStakeUnstakingPeriod: BN,
    failedApplicantApplicationStakeUnstakingPeriod: BN,
    failedApplicantRoleStakeUnstakingPeriod: BN,
    terminateCuratorApplicationStakeUnstakingPeriod: BN,
    terminateCuratorRoleStakeUnstakingPeriod: BN,
    exitCuratorRoleApplicationStakeUnstakingPeriod: BN,
    exitCuratorRoleStakeUnstakingPeriod: BN,
    text: string,
    openingType: string
  ): Promise<void> {
    const activateAt = activateAtBlock == undefined ? 'CurrentBlock' : { ExactBlock: activateAtBlock };
    const commitment = {
      application_rationing_policy: { max_active_applicants: maxActiveApplicants },
      max_review_period_length: maxReviewPeriodLength,
      application_staking_policy: {
        amount: applicationStakingPolicyAmount,
        amount_mode: 'AtLeast',
        crowded_out_unstaking_period_length: applicationCrowdedOutUnstakingPeriodLength,
        review_period_expired_unstaking_period_length: applicationExpiredUnstakingPeriodLength,
      },
      role_staking_policy: {
        amount: roleStakingPolicyAmount,
        amount_mode: 'AtLeast',
        crowded_out_unstaking_period_length: roleCrowdedOutUnstakingPeriodLength,
        review_period_expired_unstaking_period_length: roleExpiredUnstakingPeriodLength,
      },
      role_slashing_terms: {
        Slashable: {
          max_count: slashableMaxCount,
          max_percent_pts_per_time: slashableMaxPercentPtsPerTime,
        },
      },
      fill_opening_successful_applicant_application_stake_unstaking_period: successfulApplicantApplicationStakeUnstakingPeriod,
      fill_opening_failed_applicant_application_stake_unstaking_period: failedApplicantApplicationStakeUnstakingPeriod,
      fill_opening_failed_applicant_role_stake_unstaking_period: failedApplicantRoleStakeUnstakingPeriod,
      terminate_curator_application_stake_unstaking_period: terminateCuratorApplicationStakeUnstakingPeriod,
      terminate_curator_role_stake_unstaking_period: terminateCuratorRoleStakeUnstakingPeriod,
      exit_curator_role_application_stake_unstaking_period: exitCuratorRoleApplicationStakeUnstakingPeriod,
      exit_curator_role_stake_unstaking_period: exitCuratorRoleStakeUnstakingPeriod,
    };
    await this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.storageWorkingGroup.addOpening(activateAt, commitment, text, openingType)),
      sudo,
      false
    );
  }

  public async acceptApplications(account: KeyringPair, openingId: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.acceptApplications(openingId), account, false);
  }

  public async beginApplicantReview(account: KeyringPair, openingId: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.beginApplicantReview(openingId), account, false);
  }

  public async sudoBeginApplicantReview(sudo: KeyringPair, openingId: BN): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(this.api.tx.storageWorkingGroup.beginApplicantReview(openingId)),
      sudo,
      false
    );
  }

  public async applyOnOpening(
    account: KeyringPair,
    openingId: BN,
    roleStake: BN,
    applicantStake: BN,
    text: string,
    expectFailure: boolean
  ): Promise<void> {
    const memberId: BN = (await this.getMemberIds(account.address))[0];
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.applyOnOpening(
        memberId,
        openingId,
        account.address,
        roleStake,
        applicantStake,
        text
      ),
      account,
      expectFailure
    );
  }

  public async batchApplyOnOpening(
    accounts: KeyringPair[],
    openingId: BN,
    roleStake: BN,
    applicantStake: BN,
    text: string,
    expectFailure: boolean
  ): Promise<void[]> {
    return Promise.all(
      accounts.map(async keyPair => {
        await this.applyOnOpening(keyPair, openingId, roleStake, applicantStake, text, expectFailure);
      })
    );
  }

  public async fillOpening(
    account: KeyringPair,
    openingId: BN,
    applicationId: BN[],
    amountPerPayout: BN,
    nextPaymentBlock: BN,
    payoutInterval: BN
  ): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.fillOpening(openingId, applicationId, {
        amount_per_payout: amountPerPayout,
        next_payment_at_block: nextPaymentBlock,
        payout_interval: payoutInterval,
      }),
      account,
      false
    );
  }

  public async sudoFillOpening(
    sudo: KeyringPair,
    openingId: BN,
    applicationId: BN[],
    amountPerPayout: BN,
    nextPaymentBlock: BN,
    payoutInterval: BN
  ): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.sudo.sudo(
        this.api.tx.storageWorkingGroup.fillOpening(openingId, applicationId, {
          amount_per_payout: amountPerPayout,
          next_payment_at_block: nextPaymentBlock,
          payout_interval: payoutInterval,
        })
      ),
      sudo,
      false
    );
  }

  public async increaseStake(account: KeyringPair, workerId: BN, stake: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.increaseStake(workerId, stake), account, false);
  }

  public async decreaseStake(account: KeyringPair, workerId: BN, stake: BN, expectFailure: boolean): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.decreaseStake(workerId, stake),
      account,
      expectFailure
    );
  }

  public async slashStake(account: KeyringPair, workerId: BN, stake: BN, expectFailure: boolean): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.slashStake(workerId, stake), account, expectFailure);
  }

  public async updateRoleAccount(account: KeyringPair, workerId: BN, newRoleAccount: string): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.updateRoleAccount(workerId, newRoleAccount),
      account,
      false
    );
  }

  public async updateRewardAccount(account: KeyringPair, workerId: BN, newRewardAccount: string): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.updateRewardAccount(workerId, newRewardAccount),
      account,
      false
    );
  }

  public async withdrawApplication(account: KeyringPair, workerId: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.withdrawApplication(workerId), account, false);
  }

  public async batchWithdrawApplication(accounts: KeyringPair[]): Promise<void[]> {
    return Promise.all(
      accounts.map(async keyPair => {
        const applicationIds: BN[] = await this.getApplicationsIdsByRoleAccount(keyPair.address);
        await this.withdrawApplication(keyPair, applicationIds[0]);
      })
    );
  }

  public async terminateApplication(account: KeyringPair, applicationId: BN): Promise<void> {
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.terminateApplication(applicationId), account, false);
  }

  public async batchTerminateApplication(account: KeyringPair, roleAccounts: KeyringPair[]): Promise<void[]> {
    return Promise.all(
      roleAccounts.map(async keyPair => {
        const applicationIds: BN[] = await this.getActiveApplicationsIdsByRoleAccount(keyPair.address);
        await this.terminateApplication(account, applicationIds[0]);
      })
    );
  }

  public async terminateRole(
    account: KeyringPair,
    applicationId: BN,
    text: string,
    expectFailure: boolean
  ): Promise<void> {
    return this.sender.signAndSend(
      this.api.tx.storageWorkingGroup.terminateRole(applicationId, text),
      account,
      expectFailure
    );
  }

  public async leaveRole(account: KeyringPair, text: string, expectFailure: boolean): Promise<void> {
    const workerId: BN = await this.getWorkerIdByRoleAccount(account.address);
    return this.sender.signAndSend(this.api.tx.storageWorkingGroup.leaveRole(workerId, text), account, expectFailure);
  }

  public async batchLeaveRole(roleAccounts: KeyringPair[], text: string, expectFailure: boolean): Promise<void[]> {
    return Promise.all(
      roleAccounts.map(async keyPair => {
        await this.leaveRole(keyPair, text, expectFailure);
      })
    );
  }

  public async getStorageRoleParameters(): Promise<RoleParameters> {
    return (await this.api.query.actors.parameters<Option<RoleParameters>>('StorageProvider')).unwrap();
  }

  public async getAnnouncingPeriod(): Promise<BN> {
    return this.api.query.councilElection.announcingPeriod<BlockNumber>();
  }

  public async getVotingPeriod(): Promise<BN> {
    return this.api.query.councilElection.votingPeriod<BlockNumber>();
  }

  public async getRevealingPeriod(): Promise<BN> {
    return this.api.query.councilElection.revealingPeriod<BlockNumber>();
  }

  public async getCouncilSize(): Promise<BN> {
    return this.api.query.councilElection.councilSize<u32>();
  }

  public async getCandidacyLimit(): Promise<BN> {
    return this.api.query.councilElection.candidacyLimit<u32>();
  }

  public async getNewTermDuration(): Promise<BN> {
    return this.api.query.councilElection.newTermDuration<BlockNumber>();
  }

  public async getMinCouncilStake(): Promise<BN> {
    return this.api.query.councilElection.minCouncilStake<BalanceOf>();
  }

  public async getMinVotingStake(): Promise<BN> {
    return this.api.query.councilElection.minVotingStake<BalanceOf>();
  }

  public async getNextOpeningId(): Promise<BN> {
    return this.api.query.storageWorkingGroup.nextOpeningId<u32>();
  }

  public async getNextApplicationId(): Promise<BN> {
    return this.api.query.storageWorkingGroup.nextApplicationId<u32>();
  }

  public async getOpening(id: BN): Promise<Opening> {
    return ((await this.api.query.storageWorkingGroup.openingById<Codec[]>(id))[0] as unknown) as Opening;
  }

  public async getWorkers(): Promise<Worker[]> {
    return ((await this.api.query.storageWorkingGroup.workerById<Codec[]>())[1] as unknown) as Worker[];
  }

  public async getWorker(id: BN): Promise<Worker> {
    return ((await this.api.query.storageWorkingGroup.workerById<Codec[]>(id))[0] as unknown) as Worker;
  }

  public async getWorkerIdByRoleAccount(address: string): Promise<BN> {
    const workersAndIds = await this.api.query.storageWorkingGroup.workerById<Codec[]>();
    const workers: Worker[] = (workersAndIds[1] as unknown) as Worker[];
    const ids: WorkerId[] = (workersAndIds[0] as unknown) as WorkerId[];
    let index: number;
    workers.forEach((worker, i) => {
      if (worker.role_account_id.toString() === address) index = i;
    });
    return ids[index!];
  }

  public async getApplicationsIdsByRoleAccount(address: string): Promise<BN[]> {
    const applicationsAndIds = await this.api.query.storageWorkingGroup.applicationById<Codec[]>();
    const applications: Application[] = (applicationsAndIds[1] as unknown) as Application[];
    const ids: ApplicationId[] = (applicationsAndIds[0] as unknown) as ApplicationId[];
    return applications
      .map((application, index) => (application.role_account_id.toString() === address ? ids[index] : undefined))
      .filter(index => index !== undefined) as BN[];
  }

  public async getApplicationById(id: BN): Promise<HiringApplication> {
    return ((await this.api.query.hiring.applicationById<Codec[]>(id))[0] as unknown) as HiringApplication;
  }

  public async getActiveApplicationsIdsByRoleAccount(address: string): Promise<BN[]> {
    const applicationsAndIds = await this.api.query.storageWorkingGroup.applicationById<Codec[]>();
    const applications: Application[] = (applicationsAndIds[1] as unknown) as Application[];
    const ids: ApplicationId[] = (applicationsAndIds[0] as unknown) as ApplicationId[];
    return (
      await Promise.all(
        applications.map(async (application, index) => {
          if (
            application.role_account_id.toString() === address &&
            (await this.getApplicationById(application.application_id)).stage.type === 'Active'
          ) {
            return ids[index];
          } else {
            return undefined;
          }
        })
      )
    ).filter(index => index !== undefined) as BN[];
  }

  public async getStake(id: BN): Promise<Stake> {
    return ((await this.api.query.stake.stakes<Codec[]>(id))[0] as unknown) as Stake;
  }

  public async getWorkerStakeAmount(workerId: BN): Promise<BN> {
    let stakeId: BN = (await this.getWorker(workerId)).role_stake_profile.unwrap().stake_id;
    return (((await this.getStake(stakeId)).staking_status.value as unknown) as StakedState).staked_amount;
  }

  public async getRewardRelationship(id: BN): Promise<RewardRelationship> {
    return ((
      await this.api.query.recurringRewards.rewardRelationships<Codec[]>(id)
    )[0] as unknown) as RewardRelationship;
  }

  public async getWorkerRewardAccount(workerId: BN): Promise<string> {
    let rewardRelationshipId: BN = (await this.getWorker(workerId)).reward_relationship.unwrap();
    return (await this.getRewardRelationship(rewardRelationshipId)).getField('account').toString();
  }
}
