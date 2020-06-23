#![warn(missing_docs)]

pub(crate) mod parameters;

use codec::{Decode, Encode};
use rstd::vec::Vec;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use crate::ElectionParameters;
use common::working_group::WorkingGroup;

/// Encodes proposal using its details information.
pub trait ProposalEncoder<T: crate::Trait> {
    /// Encodes proposal using its details information.
    fn encode_proposal(proposal_details: ProposalDetailsOf<T>) -> Vec<u8>;
}

/// _ProposalDetails_ alias for type simplification
pub type ProposalDetailsOf<T> = ProposalDetails<
    crate::BalanceOfMint<T>,
    crate::BalanceOfGovernanceCurrency<T>,
    <T as system::Trait>::BlockNumber,
    <T as system::Trait>::AccountId,
    crate::MemberId<T>,
>;

/// Proposal details provide voters the information required for the perceived voting.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub enum ProposalDetails<MintedBalance, CurrencyBalance, BlockNumber, AccountId, MemberId> {
    /// The text of the `text` proposal
    Text(Vec<u8>),

    /// The wasm code for the `runtime upgrade` proposal
    RuntimeUpgrade(Vec<u8>),

    /// Election parameters for the `set election parameters` proposal
    SetElectionParameters(ElectionParameters<CurrencyBalance, BlockNumber>),

    /// Balance and destination account for the `spending` proposal
    Spending(MintedBalance, AccountId),

    /// New leader memberId and account_id for the `set lead` proposal
    SetLead(Option<(MemberId, AccountId)>),

    /// Balance for the `set content working group mint capacity` proposal
    SetContentWorkingGroupMintCapacity(MintedBalance),

    /// ********** Deprecated during the Nicaea release.
    /// It is kept only for backward compatibility in the Pioneer. **********
    /// AccountId for the `evict storage provider` proposal
    EvictStorageProvider(AccountId),

    /// Validator count for the `set validator count` proposal
    SetValidatorCount(u32),

    /// ********** Deprecated during the Nicaea release.
    /// It is kept only for backward compatibility in the Pioneer. **********
    /// Role parameters for the `set storage role parameters` proposal
    SetStorageRoleParameters(RoleParameters<CurrencyBalance, BlockNumber>),

    /// Add opening for the working group leader position.
    AddWorkingGroupLeaderOpening(AddOpeningParameters<BlockNumber, CurrencyBalance>),
}

impl<MintedBalance, CurrencyBalance, BlockNumber, AccountId, MemberId> Default
    for ProposalDetails<MintedBalance, CurrencyBalance, BlockNumber, AccountId, MemberId>
{
    fn default() -> Self {
        ProposalDetails::Text(b"invalid proposal details".to_vec())
    }
}

/// Parameters for the 'add opening for the leader position' proposal.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Debug)]
pub struct AddOpeningParameters<BlockNumber, Balance> {
    /// Activate opening at block.
    pub activate_at: hiring::ActivateOpeningAt<BlockNumber>,

    /// Opening conditions.
    pub commitment: working_group::OpeningPolicyCommitment<BlockNumber, Balance>,

    /// Opening description.
    pub human_readable_text: Vec<u8>,

    /// Defines working group with the open position.
    pub working_group: WorkingGroup,
}

/// ********** Deprecated during the Nicaea release.
/// It is kept only for backward compatibility in the Pioneer. **********
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Debug)]
pub struct RoleParameters<Balance, BlockNumber> {
    /// Minimum balance required to stake to enter a role.
    pub min_stake: Balance,

    /// Minimum actors to maintain - if role is unstaking
    /// and remaining actors would be less that this value - prevent or punish for unstaking.
    pub min_actors: u32,

    /// The maximum number of spots available to fill for a role.
    pub max_actors: u32,

    /// Fixed amount of tokens paid to actors' primary account.
    pub reward: Balance,

    /// Payouts are made at this block interval.
    pub reward_period: BlockNumber,

    /// Minimum amount of time before being able to unstake.
    pub bonding_period: BlockNumber,

    /// How long tokens remain locked for after unstaking.
    pub unbonding_period: BlockNumber,

    /// Minimum period required to be in service. unbonding before this time is highly penalized
    pub min_service_period: BlockNumber,

    /// "Startup" time allowed for roles that need to sync their infrastructure
    /// with other providers before they are considered in service and punishable for
    /// not delivering required level of service.
    pub startup_grace_period: BlockNumber,

    /// Small fee burned to make a request to enter role.
    pub entry_request_fee: Balance,
}

/// Contains proposal config parameters. Default values are used by migration and genesis config.
pub struct ProposalsConfigParameters {
    /// 'Set validator count' proposal voting period
    pub set_validator_count_proposal_voting_period: u32,

    /// 'Set validator count' proposal grace period
    pub set_validator_count_proposal_grace_period: u32,

    /// 'Runtime upgrade' proposal voting period
    pub runtime_upgrade_proposal_voting_period: u32,

    /// 'Runtime upgrade' proposal grace period
    pub runtime_upgrade_proposal_grace_period: u32,

    /// 'Text' proposal voting period
    pub text_proposal_voting_period: u32,

    /// 'Text' proposal grace period
    pub text_proposal_grace_period: u32,

    /// 'Set election parameters' proposal voting period
    pub set_election_parameters_proposal_voting_period: u32,

    /// 'Set election parameters' proposal grace period
    pub set_election_parameters_proposal_grace_period: u32,

    /// 'Set content working group mint capacity' proposal voting period
    pub set_content_working_group_mint_capacity_proposal_voting_period: u32,

    /// 'Set content working group mint capacity' proposal grace period
    pub set_content_working_group_mint_capacity_proposal_grace_period: u32,

    /// 'Set lead' proposal voting period
    pub set_lead_proposal_voting_period: u32,

    /// 'Set lead' proposal grace period
    pub set_lead_proposal_grace_period: u32,

    /// 'Spending' proposal voting period
    pub spending_proposal_voting_period: u32,

    /// 'Spending' proposal grace period
    pub spending_proposal_grace_period: u32,

    /// 'Add working group opening' proposal voting period
    pub add_working_group_opening_proposal_voting_period: u32,

    /// 'Add working group opening' proposal grace period
    pub add_working_group_opening_proposal_grace_period: u32,
}

impl Default for ProposalsConfigParameters {
    fn default() -> Self {
        ProposalsConfigParameters {
            set_validator_count_proposal_voting_period: 43200u32,
            set_validator_count_proposal_grace_period: 0u32,
            runtime_upgrade_proposal_voting_period: 72000u32,
            runtime_upgrade_proposal_grace_period: 72000u32,
            text_proposal_voting_period: 72000u32,
            text_proposal_grace_period: 0u32,
            set_election_parameters_proposal_voting_period: 72000u32,
            set_election_parameters_proposal_grace_period: 201_601_u32,
            set_content_working_group_mint_capacity_proposal_voting_period: 43200u32,
            set_content_working_group_mint_capacity_proposal_grace_period: 0u32,
            set_lead_proposal_voting_period: 43200u32,
            set_lead_proposal_grace_period: 0u32,
            spending_proposal_voting_period: 72000u32,
            spending_proposal_grace_period: 14400u32,
            add_working_group_opening_proposal_voting_period: 72000u32,
            add_working_group_opening_proposal_grace_period: 14400u32,
        }
    }
}
