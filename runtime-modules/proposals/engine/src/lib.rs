//! # Proposals engine module
//! Proposals `engine` module for the Joystream platform. Version 3.
//! The main component of the proposals system. Provides methods and extrinsics to create and
//! vote for proposals, inspired by Parity **Democracy module**.
//!
//! ## Overview
//! Proposals `engine` module provides an abstract mechanism to work with proposals: creation, voting,
//! execution, canceling, etc. Proposal execution demands serialized _Dispatchable_ proposal code.
//! It could be any _Dispatchable_ + _Parameter_ type, but most likely, it would be serialized (via
//! Parity _codec_ crate) extrisic call. A proposal stage can be described by its [status](./enum.ProposalStatus.html).
//!
//! ## Proposal lifecycle
//! When a proposal passes [checks](./struct.Module.html#method.ensure_create_proposal_parameters_are_valid)
//! for its [parameters](./struct.ProposalParameters.html) - it can be [created](./struct.Module.html#method.create_proposal).
//! The newly created proposal has _Active_ status. The proposal can be voted on or canceled during its
//! _voting period_. Votes can be [different](./enum.VoteKind.html). When the proposal gets enough votes
//! to be slashed or approved or _voting period_ ends - the proposal becomes _Finalized_. If the proposal
//! got approved and _grace period_ passed - the  `engine` module tries to execute the proposal.
//! The final [approved status](./enum.ApprovedProposalStatus.html) of the proposal defines
//! an overall proposal outcome.
//!
//! ### Notes
//!
//! - The proposal can be [vetoed](./struct.Module.html#method.veto_proposal)
//! anytime before the proposal execution by the _sudo_.
//! - When the proposal is created with some stake - refunding on proposal finalization with
//! different statuses should be accomplished from the external handler from the _stake module_
//! (_StakingEventsHandler_). Such a handler should call
//! [refund_proposal_stake](./struct.Module.html#method.refund_proposal_stake) callback function.
//! - If the _council_ got reelected during the proposal _voting period_ the external handler calls
//! [reset_active_proposals](./trait.Module.html#method.reset_active_proposals) function and
//! all voting results get cleared.
//!
//! ### Important abstract types to be implemented
//! Proposals `engine` module has several abstractions to be implemented in order to work correctly.
//! - _VoterOriginValidator_ - ensure valid voter identity. Voters should have permissions to vote:
//! they should be council members.
//! - [VotersParameters](./trait.VotersParameters.html) - defines total voter number, which is
//! the council size
//! - _ProposerOriginValidator_ - ensure valid proposer identity. Proposers should have permissions
//! to create a proposal: they should be members of the Joystream.
//! - [StakeHandlerProvider](./trait.StakeHandlerProvider.html) - defines an interface for the staking.
//!
//! A full list of the abstractions can be found [here](./trait.Trait.html).
//!
//! ### Supported extrinsics
//! - [vote](./struct.Module.html#method.vote) - registers a vote for the proposal
//! - [cancel_proposal](./struct.Module.html#method.cancel_proposal) - cancels the proposal (can be canceled only by owner)
//! - [veto_proposal](./struct.Module.html#method.veto_proposal) - vetoes the proposal
//!
//! ### Public API
//! - [create_proposal](./struct.Module.html#method.create_proposal) - creates proposal using provided parameters
//! - [ensure_create_proposal_parameters_are_valid](./struct.Module.html#method.ensure_create_proposal_parameters_are_valid) - ensures that we can create the proposal
//! - [refund_proposal_stake](./struct.Module.html#method.refund_proposal_stake) - a callback for _StakingHandlerEvents_
//! - [reset_active_proposals](./trait.Module.html#method.reset_active_proposals) - resets voting results for active proposals
//!
//! ## Usage
//!
//! ```
//! use frame_support::{decl_module, print};
//! use system::ensure_signed;
//! use codec::Encode;
//! use pallet_proposals_engine::{self as engine, ProposalParameters, ProposalCreationParameters};
//!
//! pub trait Trait: engine::Trait + membership::Trait {}
//!
//! decl_module! {
//!     pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//!         #[weight = 10_000_000]
//!         fn executable_proposal(origin) {
//!             print("executed!");
//!         }
//!
//!         #[weight = 10_000_000]
//!         pub fn create_spending_proposal(
//!             origin,
//!             proposer_id: T::MemberId,
//!         ) {
//!             let account_id = ensure_signed(origin)?;
//!             let parameters = ProposalParameters::default();
//!             let title = b"Spending proposal".to_vec();
//!             let description = b"We need to spend some tokens to support the working group lead."
//!                 .to_vec();
//!             let encoded_proposal_code = <Call<T>>::executable_proposal().encode();
//!
//!             <engine::Module<T>>::ensure_create_proposal_parameters_are_valid(
//!                 &parameters,
//!                 &title,
//!                 &description,
//!                 None,
//!                 None,
//!             )?;
//!
//!             let creation_parameters = ProposalCreationParameters {
//!                 account_id,
//!                 proposer_id,
//!                 proposal_parameters : parameters,
//!                 title,
//!                 description,
//!                 stake: None,
//!                 encoded_dispatchable_call_code: encoded_proposal_code,
//!                 exact_execution_block: None,
//!             };
//!
//!             <engine::Module<T>>::create_proposal(creation_parameters)?;
//!         }
//!     }
//! }
//! # fn main() {}
//! ```

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Do not delete! Cannot be uncommented by default, because of Parity decl_module! issue.
//#![warn(missing_docs)]

use types::{ApprovedProposalData, FinalizedProposalData, MemberId};

pub use types::{
    ActiveStake, ApprovedProposalStatus, BalanceOf, FinalizationData, Proposal,
    ProposalCodeDecoder, ProposalCreationParameters, ProposalDecisionStatus, ProposalExecutable,
    ProposalParameters, ProposalStatus, Stake, StakingHandler, VoteKind, VotersParameters,
    VotingResults,
};

pub(crate) mod types;

#[cfg(test)]
mod tests;

use codec::Decode;
use frame_support::dispatch::{DispatchError, DispatchResult, UnfilteredDispatchable};
use frame_support::storage::IterableStorageMap;
use frame_support::traits::{Currency, Get, LockIdentifier, LockableCurrency, WithdrawReasons};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure, print, Parameter, StorageDoubleMap,
};
use sp_arithmetic::traits::Zero;
use sp_std::vec::Vec;
use system::{ensure_root, RawOrigin};

use common::origin::ActorOriginValidator;
use frame_support::sp_std::marker::PhantomData;

/// Proposals engine trait.
pub trait Trait:
    system::Trait + pallet_timestamp::Trait + membership::Trait + balances::Trait
{
    /// Engine event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Validates proposer id and origin combination
    type ProposerOriginValidator: ActorOriginValidator<
        Self::Origin,
        MemberId<Self>,
        Self::AccountId,
    >;

    /// Validates voter id and origin combination
    type VoterOriginValidator: ActorOriginValidator<Self::Origin, MemberId<Self>, Self::AccountId>;

    /// Provides data for voting. Defines maximum voters count for the proposal.
    type TotalVotersCounter: VotersParameters;

    /// Proposal Id type
    type ProposalId: From<u32> + Parameter + Default + Copy;

    /// Provides stake logic implementation.
    type StakingHandler: StakingHandler<Self>;

    /// The fee is applied when cancel the proposal. A fee would be slashed (burned).
    type CancellationFee: Get<BalanceOf<Self>>;

    /// The fee is applied when the proposal gets rejected. A fee would be slashed (burned).
    type RejectionFee: Get<BalanceOf<Self>>;

    /// Defines max allowed proposal title length.
    type TitleMaxLength: Get<u32>;

    /// Defines max allowed proposal description length.
    type DescriptionMaxLength: Get<u32>;

    /// Defines max simultaneous active proposals number.
    type MaxActiveProposalLimit: Get<u32>;

    /// Proposals executable code. Can be instantiated by external module Call enum members.
    type DispatchableCallCode: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + Default;

    /// Proposal state change observer.
    type ProposalObserver: ProposalObserver<Self>;
}

/// Proposal state change observer.
pub trait ProposalObserver<T: Trait> {
    /// Should be called on proposal removing.
    fn proposal_removed(proposal_id: &T::ProposalId);
}

/// Nesting implementation.
impl<T: Trait, X: ProposalObserver<T>, Y: ProposalObserver<T>> ProposalObserver<T> for (X, Y) {
    fn proposal_removed(proposal_id: &<T as Trait>::ProposalId) {
        X::proposal_removed(proposal_id);
        Y::proposal_removed(proposal_id);
    }
}

decl_event!(
    /// Proposals engine events
    pub enum Event<T>
    where
        <T as Trait>::ProposalId,
        MemberId = MemberId<T>,
        <T as system::Trait>::BlockNumber,
        <T as system::Trait>::AccountId,
    {
        /// Emits on proposal creation.
        /// Params:
        /// - Member id of a proposer.
        /// - Id of a newly created proposal after it was saved in storage.
        ProposalCreated(MemberId, ProposalId),

        /// Emits on proposal status change.
        /// Params:
        /// - Id of a updated proposal.
        /// - New proposal status
        ProposalStatusUpdated(ProposalId, ProposalStatus<BlockNumber, AccountId>),

        /// Emits on voting for the proposal
        /// Params:
        /// - Voter - member id of a voter.
        /// - Id of a proposal.
        /// - Kind of vote.
        Voted(MemberId, ProposalId, VoteKind),
    }
);

decl_error! {
    /// Engine module predefined errors
    pub enum Error for Module<T: Trait>{
        /// Proposal cannot have an empty title"
        EmptyTitleProvided,

        /// Proposal cannot have an empty body
        EmptyDescriptionProvided,

        /// Title is too long
        TitleIsTooLong,

        /// Description is too long
        DescriptionIsTooLong,

        /// The proposal does not exist
        ProposalNotFound,

        /// Proposal is finalized already
        ProposalFinalized,

        /// The proposal have been already voted on
        AlreadyVoted,

        /// Not an author
        NotAuthor,

        /// Max active proposals number exceeded
        MaxActiveProposalNumberExceeded,

        /// Stake cannot be empty with this proposal
        EmptyStake,

        /// Stake should be empty for this proposal
        StakeShouldBeEmpty,

        /// Stake differs from the proposal requirements
        StakeDiffersFromRequired,

        /// Approval threshold cannot be zero
        InvalidParameterApprovalThreshold,

        /// Slashing threshold cannot be zero
        InvalidParameterSlashingThreshold,

        /// Require root origin in extrinsics
        RequireRootOrigin,

        /// Disallow to cancel the proposal if there are any votes on it.
        ProposalHasVotes,

        /// Exact execution block cannot be zero.
        ZeroExactExecutionBlock,

        /// Exact execution block cannot be less than current_block.
        InvalidExactExecutionBlock,

        /// There is not enough balance for a stake.
        InsufficientBalanceForStake,
    }
}

// Storage for the proposals engine module
decl_storage! {
    pub trait Store for Module<T: Trait> as ProposalEngine{
        /// Map proposal by its id.
        pub Proposals get(fn proposals): map hasher(blake2_128_concat)
            T::ProposalId => ProposalOf<T>;

        /// Count of all proposals that have been created.
        pub ProposalCount get(fn proposal_count): u32;

        /// Map proposal executable code by proposal id.
        pub DispatchableCallCode get(fn proposal_codes): map hasher(blake2_128_concat)
            T::ProposalId =>  Vec<u8>;

        /// Count of active proposals.
        pub ActiveProposalCount get(fn active_proposal_count): u32;

        /// Ids of proposals that are open for voting (have not been finalized yet).
        pub ActiveProposalIds get(fn active_proposal_ids): map hasher(blake2_128_concat)
            T::ProposalId=> ();

        /// Ids of proposals that were approved and theirs grace period was not expired.
        pub PendingExecutionProposalIds get(fn pending_proposal_ids): map hasher(blake2_128_concat)
            T::ProposalId=> ();

        /// Double map for preventing duplicate votes. Should be cleaned after usage.
        pub VoteExistsByProposalByVoter get(fn vote_by_proposal_by_voter):
            double_map hasher(blake2_128_concat)  T::ProposalId, hasher(blake2_128_concat) MemberId<T> => VoteKind;
    }
}

decl_module! {
    /// 'Proposal engine' substrate module
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// Predefined errors
        type Error = Error<T>;

        /// Emits an event. Default substrate implementation.
        fn deposit_event() = default;

        /// Exports const - the fee is applied when cancel the proposal. A fee would be slashed (burned).
        const CancellationFee: BalanceOf<T> = T::CancellationFee::get();

        /// Exports const -  the fee is applied when the proposal gets rejected. A fee would be slashed (burned).
        const RejectionFee: BalanceOf<T> = T::RejectionFee::get();

        /// Exports const -  max allowed proposal title length.
        const TitleMaxLength: u32 = T::TitleMaxLength::get();

        /// Exports const -  max allowed proposal description length.
        const DescriptionMaxLength: u32 = T::DescriptionMaxLength::get();

        /// Exports const -  max simultaneous active proposals number.
        const MaxActiveProposalLimit: u32 = T::MaxActiveProposalLimit::get();

        /// Vote extrinsic. Conditions:  origin must allow votes.
        #[weight = 10_000_000] // TODO: adjust weight
        pub fn vote(origin, voter_id: MemberId<T>, proposal_id: T::ProposalId, vote: VoteKind)  {
            T::VoterOriginValidator::ensure_actor_origin(
                origin,
                voter_id,
            )?;

            ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::ProposalNotFound);
            let mut proposal = Self::proposals(proposal_id);

            ensure!(matches!(proposal.status, ProposalStatus::Active{..}), Error::<T>::ProposalFinalized);

            let did_not_vote_before = !<VoteExistsByProposalByVoter<T>>::contains_key(
                proposal_id,
                voter_id,
            );

            ensure!(did_not_vote_before, Error::<T>::AlreadyVoted);

            proposal.voting_results.add_vote(vote.clone());

            // mutation

            <Proposals<T>>::insert(proposal_id, proposal);
            <VoteExistsByProposalByVoter<T>>::insert(proposal_id, voter_id, vote.clone());
            Self::deposit_event(RawEvent::Voted(voter_id, proposal_id, vote));
        }

        /// Cancel a proposal by its original proposer.
        #[weight = 10_000_000] // TODO: adjust weight
        pub fn cancel_proposal(origin, proposer_id: MemberId<T>, proposal_id: T::ProposalId) {
            T::ProposerOriginValidator::ensure_actor_origin(
                origin,
                proposer_id,
            )?;

            ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::ProposalNotFound);
            let proposal = Self::proposals(proposal_id);

            ensure!(proposer_id == proposal.proposer_id, Error::<T>::NotAuthor);
            ensure!(matches!(proposal.status, ProposalStatus::Active{..}), Error::<T>::ProposalFinalized);
            ensure!(proposal.voting_results.no_votes_yet(), Error::<T>::ProposalHasVotes);

            // mutation

            Self::finalize_proposal(proposal_id, ProposalDecisionStatus::Canceled);
        }

        /// Veto a proposal. Must be root.
        #[weight = 10_000_000] // TODO: adjust weight
        pub fn veto_proposal(origin, proposal_id: T::ProposalId) {
            ensure_root(origin)?;

            ensure!(<Proposals<T>>::contains_key(proposal_id), Error::<T>::ProposalNotFound);
            let proposal = Self::proposals(proposal_id);

            // mutation

            if <PendingExecutionProposalIds<T>>::contains_key(proposal_id) {
                Self::veto_pending_execution_proposal(proposal_id, proposal);
            } else {
                ensure!(matches!(proposal.status, ProposalStatus::Active{..}), Error::<T>::ProposalFinalized);
                Self::finalize_proposal(proposal_id, ProposalDecisionStatus::Vetoed);
            }
        }

        /// Block finalization. Perform voting period check, vote result tally, approved proposals
        /// grace period checks, and proposal execution.
        fn on_finalize(_n: T::BlockNumber) {
            let finalized_proposals = Self::get_finalized_proposals();

            // mutation

            // Check vote results. Approved proposals with zero grace period will be
            // transitioned to the PendingExecution status.
            for  proposal_data in finalized_proposals {
                <Proposals<T>>::insert(proposal_data.proposal_id, proposal_data.proposal);
                Self::finalize_proposal(proposal_data.proposal_id, proposal_data.status);
            }

            let executable_proposals =
                Self::get_approved_proposal_ready_for_execution();

            // Execute approved proposals with expired grace period
            for approved_proposal in executable_proposals {
                Self::execute_proposal(approved_proposal);
            }
        }
    }
}

impl<T: Trait> Module<T> {
    /// Create proposal. Requires 'proposal origin' membership.
    pub fn create_proposal(
        creation_params: ProposalCreationParameters<
            T::BlockNumber,
            BalanceOf<T>,
            MemberId<T>,
            T::AccountId,
        >,
    ) -> Result<T::ProposalId, DispatchError> {
        Self::ensure_create_proposal_parameters_are_valid(
            &creation_params.proposal_parameters,
            &creation_params.title,
            &creation_params.description,
            creation_params.stake.clone(),
            creation_params.exact_execution_block,
        )?;

        // checks passed
        // mutation

        let next_proposal_count_value = Self::proposal_count() + 1;
        let new_proposal_id = next_proposal_count_value;
        let proposal_id = T::ProposalId::from(new_proposal_id);

        let stake_data = if let Some(stake) = creation_params.stake {
            ensure!(
                T::StakingHandler::is_enough_balance_for_stake(&stake.account_id, stake.balance),
                Error::<T>::InsufficientBalanceForStake
            );

            T::StakingHandler::lock(&stake.account_id, stake.balance);

            Some(ActiveStake {
                source_account_id: stake.account_id,
            })
        } else {
            None
        };

        let new_proposal = Proposal {
            created_at: Self::current_block(),
            parameters: creation_params.proposal_parameters,
            title: creation_params.title,
            description: creation_params.description,
            proposer_id: creation_params.proposer_id,
            status: ProposalStatus::Active(stake_data),
            voting_results: VotingResults::default(),
            exact_execution_block: creation_params.exact_execution_block,
        };

        <Proposals<T>>::insert(proposal_id, new_proposal);
        <DispatchableCallCode<T>>::insert(
            proposal_id,
            creation_params.encoded_dispatchable_call_code,
        );
        <ActiveProposalIds<T>>::insert(proposal_id, ());
        ProposalCount::put(next_proposal_count_value);
        Self::increase_active_proposal_counter();

        Self::deposit_event(RawEvent::ProposalCreated(
            creation_params.proposer_id,
            proposal_id,
        ));

        Ok(proposal_id)
    }

    /// Performs all checks for the proposal creation:
    /// - title, body lengths
    /// - max active proposal
    /// - provided parameters: approval_threshold_percentage and slashing_threshold_percentage > 0
    /// - provided stake balance and parameters.required_stake are valid
    pub fn ensure_create_proposal_parameters_are_valid(
        parameters: &ProposalParameters<T::BlockNumber, BalanceOf<T>>,
        title: &[u8],
        description: &[u8],
        stake: Option<Stake<T::AccountId, BalanceOf<T>>>,
        exact_execution_block: Option<T::BlockNumber>,
    ) -> DispatchResult {
        ensure!(!title.is_empty(), Error::<T>::EmptyTitleProvided);
        ensure!(
            title.len() as u32 <= T::TitleMaxLength::get(),
            Error::<T>::TitleIsTooLong
        );

        ensure!(
            !description.is_empty(),
            Error::<T>::EmptyDescriptionProvided
        );
        ensure!(
            description.len() as u32 <= T::DescriptionMaxLength::get(),
            Error::<T>::DescriptionIsTooLong
        );

        ensure!(
            (Self::active_proposal_count()) < T::MaxActiveProposalLimit::get(),
            Error::<T>::MaxActiveProposalNumberExceeded
        );

        ensure!(
            parameters.approval_threshold_percentage > 0,
            Error::<T>::InvalidParameterApprovalThreshold
        );

        ensure!(
            parameters.slashing_threshold_percentage > 0,
            Error::<T>::InvalidParameterSlashingThreshold
        );

        // check stake parameters
        if let Some(required_stake) = parameters.required_stake {
            if let Some(stake) = stake.clone() {
                ensure!(
                    required_stake == stake.balance,
                    Error::<T>::StakeDiffersFromRequired
                );
            } else {
                return Err(Error::<T>::EmptyStake.into());
            }
        }

        if stake.is_some() && parameters.required_stake.is_none() {
            return Err(Error::<T>::StakeShouldBeEmpty.into());
        }

        if let Some(execution_block) = exact_execution_block {
            if execution_block == Zero::zero() {
                return Err(Error::<T>::ZeroExactExecutionBlock.into());
            }

            let now = Self::current_block();
            if execution_block < now + parameters.grace_period + parameters.voting_period {
                return Err(Error::<T>::InvalidExactExecutionBlock.into());
            }
        }

        Ok(())
    }

    /// Resets voting results for active proposals.
    /// Possible application includes new council elections.
    pub fn reset_active_proposals() {
        <ActiveProposalIds<T>>::iter().for_each(|(proposal_id, _)| {
            <Proposals<T>>::mutate(proposal_id, |proposal| {
                proposal.reset_proposal();
                <VoteExistsByProposalByVoter<T>>::remove_prefix(&proposal_id);
            });
        });
    }
}

impl<T: Trait> Module<T> {
    // Wrapper-function over system::block_number()
    fn current_block() -> T::BlockNumber {
        <system::Module<T>>::block_number()
    }

    // Enumerates through active proposals. Tally Voting results.
    // Returns proposals with finalized status and id
    fn get_finalized_proposals() -> Vec<FinalizedProposal<T>> {
        // Enumerate active proposals id and gather finalization data.
        // Skip proposals with unfinished voting.
        <ActiveProposalIds<T>>::iter()
            .filter_map(|(proposal_id, _)| {
                // load current proposal
                let proposal = Self::proposals(proposal_id);

                // Calculates votes, takes in account voting period expiration.
                // If voting process is in progress, then decision status is None.
                let decision_status = proposal.define_proposal_decision_status(
                    T::TotalVotersCounter::total_voters_count(),
                    Self::current_block(),
                );

                // map to FinalizedProposalData if decision for the proposal is made or return None
                decision_status.map(|status| FinalizedProposalData {
                    proposal_id,
                    proposal,
                    status,
                    finalized_at: Self::current_block(),
                })
            })
            .collect() // compose output vector
    }

    // Veto approved proposal during its grace period. Saves a new proposal status and removes
    // proposal id from the 'PendingExecutionProposalIds'
    fn veto_pending_execution_proposal(proposal_id: T::ProposalId, proposal: ProposalOf<T>) {
        <PendingExecutionProposalIds<T>>::remove(proposal_id);

        let vetoed_proposal_status =
            ProposalStatus::finalized(ProposalDecisionStatus::Vetoed, Self::current_block());

        <Proposals<T>>::insert(
            proposal_id,
            Proposal {
                status: vetoed_proposal_status,
                ..proposal
            },
        );
    }

    // Executes approved proposal code
    fn execute_proposal(approved_proposal: ApprovedProposal<T>) {
        let proposal_code = Self::proposal_codes(approved_proposal.proposal_id);

        let proposal_code_result = T::DispatchableCallCode::decode(&mut &proposal_code[..]);

        let approved_proposal_status = match proposal_code_result {
            Ok(proposal_code) => {
                if let Err(dispatch_error) =
                    proposal_code.dispatch_bypass_filter(T::Origin::from(RawOrigin::Root))
                {
                    ApprovedProposalStatus::failed_execution(Self::parse_dispatch_error(
                        dispatch_error.error,
                    ))
                } else {
                    ApprovedProposalStatus::Executed
                }
            }
            Err(error) => ApprovedProposalStatus::failed_execution(error.what()),
        };

        let proposal_execution_status = ProposalStatus::Finalized(FinalizationData {
            proposal_status: ProposalDecisionStatus::Approved(approved_proposal_status),
            finalized_at: approved_proposal.finalisation_status_data.finalized_at,
        });

        Self::deposit_event(RawEvent::ProposalStatusUpdated(
            approved_proposal.proposal_id,
            proposal_execution_status,
        ));

        Self::remove_proposal_data(&approved_proposal.proposal_id);
    }

    // Performs all actions on proposal finalization:
    // - clean active proposal cache
    // - update proposal status fields (status, finalized_at)
    // - add to pending execution proposal cache if approved
    // - slash and unstake proposal stake if stake exists
    // - decrease active proposal counter
    // - fire an event
    // It prints an error message in case of an attempt to finalize the non-active proposal.
    fn finalize_proposal(proposal_id: T::ProposalId, decision_status: ProposalDecisionStatus) {
        Self::decrease_active_proposal_counter();
        <ActiveProposalIds<T>>::remove(&proposal_id.clone());

        let mut proposal = Self::proposals(proposal_id);

        if let ProposalStatus::Active(active_stake) = proposal.status.clone() {
            let mut clean_finilized_proposal = true;
            if let ProposalDecisionStatus::Approved { .. } = decision_status {
                <PendingExecutionProposalIds<T>>::insert(proposal_id, ());

                clean_finilized_proposal = false; // keep pending execution proposal
            }

            // deal with stakes if necessary
            let slash_balance =
                Self::calculate_slash_balance(&decision_status, &proposal.parameters);
            Self::slash_and_unstake(active_stake, slash_balance);

            // create finalized proposal status with error if any
            let new_proposal_status =
                ProposalStatus::finalized(decision_status, Self::current_block());

            if clean_finilized_proposal {
                Self::remove_proposal_data(&proposal_id);
            } else {
                proposal.status = new_proposal_status.clone();
                <Proposals<T>>::insert(proposal_id, proposal);
            }

            Self::deposit_event(RawEvent::ProposalStatusUpdated(
                proposal_id,
                new_proposal_status,
            ));
        } else {
            print("Broken invariant: proposal cannot be non-active during the finalisation");
        }
    }

    // Slashes the stake and perform unstake only in case of existing stake
    fn slash_and_unstake(
        current_stake_data: Option<ActiveStake<T::AccountId>>,
        slash_balance: BalanceOf<T>,
    ) {
        // only if stake exists
        if let Some(stake_data) = current_stake_data {
            if !slash_balance.is_zero() {
                T::StakingHandler::slash(&stake_data.source_account_id, Some(slash_balance));
            }

            T::StakingHandler::unlock(&stake_data.source_account_id);
        }
    }

    // Calculates required slash based on finalization ProposalDecisionStatus and proposal parameters.
    // Method visibility allows testing.
    pub(crate) fn calculate_slash_balance(
        decision_status: &ProposalDecisionStatus,
        proposal_parameters: &ProposalParameters<T::BlockNumber, BalanceOf<T>>,
    ) -> BalanceOf<T> {
        match decision_status {
            ProposalDecisionStatus::Rejected | ProposalDecisionStatus::Expired => {
                T::RejectionFee::get()
            }
            ProposalDecisionStatus::Approved { .. } | ProposalDecisionStatus::Vetoed => {
                BalanceOf::<T>::zero()
            }
            ProposalDecisionStatus::Canceled => T::CancellationFee::get(),
            ProposalDecisionStatus::Slashed => proposal_parameters
                .required_stake
                .clone()
                .unwrap_or_else(BalanceOf::<T>::zero), // stake if set or zero
        }
    }

    // Enumerates approved proposals and checks their grace period expiration and
    // exact execution block.
    fn get_approved_proposal_ready_for_execution() -> Vec<ApprovedProposal<T>> {
        <PendingExecutionProposalIds<T>>::iter()
            .filter_map(|(proposal_id, _)| {
                let proposal = Self::proposals(proposal_id);

                let now = Self::current_block();
                if proposal.is_ready_for_execution(now) {
                    // this should be true, because it was tested inside is_grace_period_expired()
                    if let ProposalStatus::Finalized(finalisation_data) = proposal.status.clone() {
                        Some(ApprovedProposalData {
                            proposal_id,
                            proposal,
                            finalisation_status_data: finalisation_data,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    // Increases active proposal counter.
    fn increase_active_proposal_counter() {
        let next_active_proposal_count_value = Self::active_proposal_count() + 1;
        ActiveProposalCount::put(next_active_proposal_count_value);
    }

    // Decreases active proposal counter down to zero. Decreasing below zero has no effect.
    fn decrease_active_proposal_counter() {
        let current_active_proposal_counter = Self::active_proposal_count();

        if current_active_proposal_counter > 0 {
            let next_active_proposal_count_value = current_active_proposal_counter - 1;
            ActiveProposalCount::put(next_active_proposal_count_value);
        };
    }

    // Parse dispatchable execution result.
    fn parse_dispatch_error(error: DispatchError) -> &'static str {
        match error {
            DispatchError::BadOrigin => error.into(),
            DispatchError::Other(msg) => msg,
            DispatchError::CannotLookup => error.into(),
            DispatchError::Module {
                index: _,
                error: _,
                message: msg,
            } => msg.unwrap_or("Dispatch error."),
        }
    }

    // Clean proposal data. Remove proposal, votes from the storage.
    fn remove_proposal_data(proposal_id: &T::ProposalId) {
        <Proposals<T>>::remove(proposal_id);
        <DispatchableCallCode<T>>::remove(proposal_id);
        <PendingExecutionProposalIds<T>>::remove(proposal_id);
        <VoteExistsByProposalByVoter<T>>::remove_prefix(&proposal_id);

        T::ProposalObserver::proposal_removed(proposal_id);
    }
}

// Simplification of the 'FinalizedProposalData' type
type FinalizedProposal<T> = FinalizedProposalData<
    <T as Trait>::ProposalId,
    <T as system::Trait>::BlockNumber,
    MemberId<T>,
    BalanceOf<T>,
    <T as system::Trait>::AccountId,
>;

// Simplification of the 'ApprovedProposalData' type
type ApprovedProposal<T> = ApprovedProposalData<
    <T as Trait>::ProposalId,
    <T as system::Trait>::BlockNumber,
    MemberId<T>,
    BalanceOf<T>,
    <T as system::Trait>::AccountId,
>;

// Simplification of the 'Proposal' type
type ProposalOf<T> = Proposal<
    <T as system::Trait>::BlockNumber,
    MemberId<T>,
    BalanceOf<T>,
    <T as system::Trait>::AccountId,
>;

pub struct StakingManager<T: Trait, LockId: Get<LockIdentifier>> {
    trait_marker: PhantomData<T>,
    lock_id_marker: PhantomData<LockId>,
}

impl<T: Trait, LockId: Get<LockIdentifier>> StakingHandler<T> for StakingManager<T, LockId> {
    fn lock(account_id: &T::AccountId, amount: BalanceOf<T>) {
        <balances::Module<T>>::set_lock(LockId::get(), &account_id, amount, WithdrawReasons::all())
    }

    fn unlock(account_id: &T::AccountId) {
        T::Currency::remove_lock(LockId::get(), &account_id);
    }

    fn slash(account_id: &T::AccountId, amount: Option<BalanceOf<T>>) -> BalanceOf<T> {
        let locks = <balances::Module<T>>::locks(&account_id);

        let existing_lock = locks.iter().find(|lock| lock.id == LockId::get());

        let mut actually_slashed_balance = Default::default();
        if let Some(existing_lock) = existing_lock {
            Self::unlock(&account_id);

            let mut slashable_amount = existing_lock.amount;
            if let Some(amount) = amount {
                if existing_lock.amount > amount {
                    let new_amount = existing_lock.amount - amount;
                    Self::lock(&account_id, new_amount);

                    slashable_amount = amount;
                }
            }

            let _ = <balances::Module<T>>::slash(&account_id, slashable_amount);

            actually_slashed_balance = slashable_amount
        }

        actually_slashed_balance
    }

    fn set_stake(account_id: &T::AccountId, new_stake: BalanceOf<T>) -> DispatchResult {
        let current_stake = Self::current_stake(account_id);

        //Unlock previous stake if its not zero.
        if current_stake > Zero::zero() {
            Self::unlock(account_id);
        }

        if !Self::is_enough_balance_for_stake(account_id, new_stake) {
            //Restore previous stake if its not zero.
            if current_stake > Zero::zero() {
                Self::lock(account_id, current_stake);
            }
            return Err(DispatchError::Other("Not enough balance for a new stake."));
        }

        Self::lock(account_id, new_stake);

        Ok(())
    }

    fn is_member_staking_account(_member_id: &MemberId<T>, _account_id: &T::AccountId) -> bool {
        true
    }

    fn is_account_free_of_conflicting_stakes(account_id: &T::AccountId) -> bool {
        let locks = <balances::Module<T>>::locks(&account_id);

        let existing_lock = locks.iter().find(|lock| lock.id == LockId::get());

        existing_lock.is_none()
    }

    fn is_enough_balance_for_stake(account_id: &T::AccountId, amount: BalanceOf<T>) -> bool {
        <balances::Module<T>>::usable_balance(account_id) >= amount
    }

    fn current_stake(account_id: &T::AccountId) -> BalanceOf<T> {
        let locks = <balances::Module<T>>::locks(&account_id);

        let existing_lock = locks.iter().find(|lock| lock.id == LockId::get());

        existing_lock.map_or(Zero::zero(), |lock| lock.amount)
    }
}
