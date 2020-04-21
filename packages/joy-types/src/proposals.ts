import { Text, u32, Enum, getTypeRegistry, GenericAccountId, u8, Vec, Option } from "@polkadot/types";
import { BlockNumber, Balance } from "@polkadot/types/interfaces";
import { MemberId } from "./members";
import { StakeId } from "./stake";
import AccountId from "@polkadot/types/primitive/Generic/AccountId";
import { JoyStruct } from "./JoyStruct";

export type VotingResults = {
  abstensions: u32;
  approvals: u32;
  rejections: u32;
  slashes: u32;
};

export type ProposalParameters = {
  // During this period, votes can be accepted
  votingPeriod: BlockNumber;

  /* A pause before execution of the approved proposal. Zero means approved proposal would be
     executed immediately. */
  gracePeriod: BlockNumber;

  // Quorum percentage of approving voters required to pass the proposal.
  approvalQuorumPercentage: u32;

  // Approval votes percentage threshold to pass the proposal.
  approvalThresholdPercentage: u32;

  // Quorum percentage of voters required to slash the proposal.
  slashingQuorumPercentage: u32;

  // Slashing votes percentage threshold to slash the proposal.
  slashingThresholdPercentage: u32;

  // Proposal stake
  requiredStake: Balance;
};

export type Proposal = {
  parameters: ProposalParameters;
  proposerId: MemberId;
  title: Text;
  description: Text;
  createdAt: BlockNumber;
  status: ProposalStatus;
  votingResults: VotingResults;
};

export const IProposalStatus: { [key: string]: string } = {
  Active: "Active",
  Canceled: "Canceled",
  Expired: "Expired",
  Approved: "Approved",
  Rejected: "Rejected",
  Vetoed: "Vetoed",
  PendingExecution: "PendingExecution",
  Executed: "Executed",
  ExecutionFailed: "ExecutionFailed",
  Finalized: "Finalized",
  Slashed: "Slashed",
};

export type IActiveStake = {
  stake_id: StakeId;
  source_account_id: AccountId;
};
export class ActiveStake extends JoyStruct<IActiveStake> {
  constructor(value?: IActiveStake) {
    super(
      {
        stake_id: u32,
        source_account_id: GenericAccountId,
      },
      value
    );
  }
}

export type ProposalDecisionStatuses = "Canceled" | "Vetoed" | "Rejected" | "Slashed" | "Expired" | "Approved";

export class ProposalDecisionStatus extends Enum {
  constructor(value?: any, index?: number) {
    super(["Canceled", "Vetoed", "Rejected", "Slashed", "Expired", "Approved"], value, index);
  }
}

export type IFinalizationData = {
  proposal_status: ProposalDecisionStatus;
  finalized_at: BlockNumber;
  encoded_unstaking_error_due_to_broken_runtime: Option<Vec<u8>>;
  stake_data_after_unstaking_error: Option<ActiveStake>;
};

export class FinalizationData extends JoyStruct<IFinalizationData> {
  constructor(value?: IFinalizationData) {
    super(
      {
        proposal_status: ProposalDecisionStatus,
        finalized_at: u32,
        encoded_unstaking_error_due_to_broken_runtime: Option.with(Vec.with(u8)),
        stake_data_after_unstaking_error: Option.with(ActiveStake),
      },
      value
    );
  }
}

export class Active extends ActiveStake {}
export class Finalized extends FinalizationData {}

export class ProposalStatus extends Enum {
  constructor(value?: any) {
    super(
      {
        Active,
        Finalized,
      },
      value
    );
  }
}

export const VoteKinds: { [key: string]: string } = {
  Abstain: "Abstain",
  Approve: "Approve",
  Reject: "Reject",
  Slash: "Slash",
};

export class VoteKind extends Enum {
  constructor(value?: any) {
    super(["Abstain", "Approve", "Reject", "Slash"], value);
  }
}

export type ProposalVotes = [MemberId, VoteKind][];

export class ProposalId extends u32 {}

// const proposalTypes = {
//   VoteKind,
//   ProposalStatus
// };

// export default proposalTypes;
export function registerProposalTypes() {
  try {
    getTypeRegistry().register({
      ProposalId,
      ProposalStatus,
      VoteKind,
    });
  } catch (err) {
    console.error("Failed to register custom types of proposals module", err);
  }
}
