// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod constraints;
#[cfg(test)]
mod tests;
mod types;
#[macro_use]
mod errors;

use rstd::collections::btree_map::BTreeMap;
use rstd::collections::btree_set::BTreeSet;
use rstd::prelude::*;
use rstd::vec::Vec;
use sr_primitives::traits::{EnsureOrigin, One, Zero};
use srml_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};
use srml_support::{decl_event, decl_module, decl_storage, dispatch, ensure};
use system::{ensure_root, ensure_signed, RawOrigin};

use constraints::InputValidationLengthConstraint;
use errors::bureaucracy_errors::*;
use errors::WrappedError;
use types::{
    Lead, OpeningPolicyCommitment, Worker, WorkerApplication, WorkerOpening, WorkerRoleStakeProfile,
};

//TODO: docs
//TODO: migrate to decl_error

/// Alias for the _Lead_ type
pub type LeadOf<T> =
    Lead<<T as membership::members::Trait>::MemberId, <T as system::Trait>::AccountId>;

/*
+ add_worker_opening
+ accept_worker_applications
+ begin_worker_applicant_review
+ fill_worker_opening
+ withdraw_worker_application
+ terminate_worker_application
+ apply_on_worker_opening
*/

/// Workaround for BTreeSet type
pub type WorkerApplicationIdSet<T> = BTreeSet<WorkerApplicationId<T>>;

/// Type for the identifier for an opening for a worker.
pub type WorkerOpeningId<T> = <T as hiring::Trait>::OpeningId;

/// Type for the identifier for an application as a worker.
pub type WorkerApplicationId<T> = <T as hiring::Trait>::ApplicationId;

/// Balance type of runtime
pub type BalanceOf<T> =
    <<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// Balance type of runtime
pub type CurrencyOf<T> = <T as stake::Trait>::Currency;

/// Negative imbalance of runtime.
pub type NegativeImbalance<T> =
    <<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// Alias for the worker application id to the worker id dictionary
pub type WorkerApplicationIdToWorkerIdMap<T> = BTreeMap<WorkerApplicationId<T>, WorkerId<T>>;

/// Type identifier for worker role, which must be same as membership actor identifier
pub type WorkerId<T> = <T as membership::members::Trait>::ActorId;

// Type simplification
type WorkerOpeningInfo<T> = (
    WorkerOpening<
        <T as hiring::Trait>::OpeningId,
        <T as system::Trait>::BlockNumber,
        BalanceOf<T>,
        WorkerApplicationId<T>,
    >,
    hiring::Opening<
        BalanceOf<T>,
        <T as system::Trait>::BlockNumber,
        <T as hiring::Trait>::ApplicationId,
    >,
);

// Type simplification
type WorkerApplicationInfo<T> = (
    WorkerApplication<
        <T as system::Trait>::AccountId,
        WorkerOpeningId<T>,
        <T as membership::members::Trait>::MemberId,
        <T as hiring::Trait>::ApplicationId,
    >,
    WorkerApplicationId<T>,
    WorkerOpening<
        <T as hiring::Trait>::OpeningId,
        <T as system::Trait>::BlockNumber,
        BalanceOf<T>,
        WorkerApplicationId<T>,
    >,
);

type WorkerOf<T> = Worker<
    <T as system::Trait>::AccountId,
    //    T::RewardRelationshipId,
    //    T::StakeId,
    //    <T as system::Trait>::BlockNumber,
    //    LeadId<T>,
    //    WorkerApplicationId<T>,
    //    PrincipalId<T>,
>;

/// The bureaucracy main _Trait_
pub trait Trait<I: Instance>: system::Trait + membership::members::Trait + hiring::Trait {
    /// Engine event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    /// Proposals engine events
    pub enum Event<T, I>
    where
        <T as membership::members::Trait>::MemberId,
        <T as system::Trait>::AccountId,
        WorkerOpeningId = WorkerOpeningId<T>,
        WorkerApplicationId = WorkerApplicationId<T>,
        WorkerApplicationIdToWorkerIdMap = WorkerApplicationIdToWorkerIdMap<T>,
    {
        /// Emits on setting the leader.
        /// Params:
        /// - Member id of the leader.
        /// - Role account id of the leader.
        LeaderSet(MemberId, AccountId),
        /// Emits on adding new worker opening.
        /// Params:
        /// - Worker opening id
        WorkerOpeningAdded(WorkerOpeningId),
        /// Emits on accepting application for the worker opening.
        /// Params:
        /// - Worker opening id
        AcceptedWorkerApplications(WorkerOpeningId),
        /// Emits on adding the application for the worker opening.
        /// Params:
        /// - Worker opening id
        /// - Worker application id
        AppliedOnWorkerOpening(WorkerOpeningId, WorkerApplicationId),
        /// Emits on withdrawing the application for the worker opening.
        /// Params:
        /// - Worker application id
        WorkerApplicationWithdrawn(WorkerApplicationId),
        /// Emits on terminating the application for the worker opening.
        /// Params:
        /// - Worker application id
        WorkerApplicationTerminated(WorkerApplicationId),
        /// Emits on beginning the application review for the worker opening.
        /// Params:
        /// - Worker opening id
        BeganWorkerApplicationReview(WorkerOpeningId),
        /// Emits on filling the worker opening.
        /// Params:
        /// - Worker opening id
        /// - Worker application id to the worker id dictionary
        WorkerOpeningFilled(WorkerOpeningId, WorkerApplicationIdToWorkerIdMap),
    }
);

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance> as Bureaucracy {
        /// The current lead.
        pub CurrentLead get(current_lead) : Option<LeadOf<T>>;

        /// Next identifier value for new worker opening.
        pub NextWorkerOpeningId get(next_worker_opening_id): WorkerOpeningId<T>;

        /// Maps identifier to worker opening.
        pub WorkerOpeningById get(worker_opening_by_id): linked_map WorkerOpeningId<T> => WorkerOpening<T::OpeningId, T::BlockNumber, BalanceOf<T>, WorkerApplicationId<T>>;

        /// Opening human readable text length limits
        pub OpeningHumanReadableText get(opening_human_readable_text): InputValidationLengthConstraint;

        /// Maps identifier to worker application on opening.
        pub WorkerApplicationById get(worker_application_by_id) : linked_map WorkerApplicationId<T> => WorkerApplication<T::AccountId, WorkerOpeningId<T>, T::MemberId, T::ApplicationId>;

        /// Next identifier value for new worker application.
        pub NextWorkerApplicationId get(next_worker_application_id) : WorkerApplicationId<T>;

        /// Worker application human readable text length limits
        pub WorkerApplicationHumanReadableText get(worker_application_human_readable_text) : InputValidationLengthConstraint;

        /// Maps identifier to corresponding worker.
        pub WorkerById get(worker_by_id) : linked_map WorkerId<T> => WorkerOf<T>;

        /// Next identifier for new worker.
        pub NextWorkerId get(next_worker_id) : WorkerId<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance> for enum Call where origin: T::Origin {
        /// Default deposit_event() handler
        fn deposit_event() = default;

        /// Introduce a lead when one is not currently set.
        pub fn set_lead(origin, member_id: T::MemberId, role_account_id: T::AccountId) -> dispatch::Result {
            ensure_root(origin)?;

            // Construct lead
            let new_lead = Lead {
                member_id,
                role_account_id: role_account_id.clone(),
            };

            // mutation

            // Update current lead
            <CurrentLead<T, I>>::put(new_lead);

            // Trigger an event
            Self::deposit_event(RawEvent::LeaderSet(member_id, role_account_id));

            Ok(())
        }

         /// Add an opening for a worker role.
        pub fn add_worker_opening(
            origin,
            activate_at: hiring::ActivateOpeningAt<T::BlockNumber>,
            commitment: OpeningPolicyCommitment<T::BlockNumber,
            BalanceOf<T>>,
            human_readable_text: Vec<u8>
        ){
            // Ensure lead is set and is origin signer
            Self::ensure_origin_is_set_lead(origin)?;

            Self::ensure_opening_human_readable_text_is_valid(&human_readable_text)?;

            // Add opening
            // NB: This call can in principle fail, because the staking policies
            // may not respect the minimum currency requirement.

            let policy_commitment = commitment.clone();

            // mutation

            let opening_id = ensure_on_wrapped_error!(
                hiring::Module::<T>::add_opening(
                    activate_at,
                    commitment.max_review_period_length,
                    commitment.application_rationing_policy,
                    commitment.application_staking_policy,
                    commitment.role_staking_policy,
                    human_readable_text,
            ))?;

            let new_worker_opening_id = NextWorkerOpeningId::<T, I>::get();

            // Create and add worker opening.
            let new_opening_by_id = WorkerOpening::<WorkerOpeningId<T>, T::BlockNumber, BalanceOf<T>, WorkerApplicationId<T>> {
                opening_id : opening_id,
                worker_applications: BTreeSet::new(),
                policy_commitment: policy_commitment
            };

            WorkerOpeningById::<T, I>::insert(new_worker_opening_id, new_opening_by_id);

            // Update NextWorkerOpeningId
            NextWorkerOpeningId::<T, I>::mutate(|id| *id += <WorkerOpeningId<T> as One>::one());

            // Trigger event
            Self::deposit_event(RawEvent::WorkerOpeningAdded(new_worker_opening_id));
        }

            /// Begin accepting worker applications to an opening that is active.
        pub fn accept_worker_applications(origin, worker_opening_id: WorkerOpeningId<T>)  {

            // Ensure lead is set and is origin signer
            Self::ensure_origin_is_set_lead(origin)?;

            // Ensure opening exists in this working group
            // NB: Even though call to hiring module will have implicit check for
            // existence of opening as well, this check is to make sure that the opening is for
            // this working group, not something else.
            let (worker_opening, _opening) = Self::ensure_worker_opening_exists(&worker_opening_id)?;

            // Attempt to begin accepting applications
            // NB: Combined ensure check and mutation in hiring module

            // mutation

            ensure_on_wrapped_error!(
                hiring::Module::<T>::begin_accepting_applications(worker_opening.opening_id)
            )?;


            // Trigger event
            Self::deposit_event(RawEvent::AcceptedWorkerApplications(worker_opening_id));
        }

        /// Apply on a worker opening.
        pub fn apply_on_worker_opening(
            origin,
            member_id: T::MemberId,
            worker_opening_id: WorkerOpeningId<T>,
            role_account: T::AccountId,
            opt_role_stake_balance: Option<BalanceOf<T>>,
            opt_application_stake_balance: Option<BalanceOf<T>>,
            human_readable_text: Vec<u8>
        ) {
            // Ensure origin which will server as the source account for staked funds is signed
            let source_account = ensure_signed(origin)?;

            // In absence of a more general key delegation system which allows an account with some funds to
            // grant another account permission to stake from its funds, the origin of this call must have the funds
            // and cannot specify another arbitrary account as the source account.
            // Ensure the source_account is either the controller or root account of member with given id
            ensure!(
                membership::members::Module::<T>::ensure_member_controller_account(&source_account, &member_id).is_ok() ||
                membership::members::Module::<T>::ensure_member_root_account(&source_account, &member_id).is_ok(),
                MSG_ORIGIN_IS_NEITHER_MEMBER_CONTROLLER_OR_ROOT
            );

            // Ensure worker opening exists
            let (worker_opening, _opening) = Self::ensure_worker_opening_exists(&worker_opening_id)?;

            // Ensure that there is sufficient balance to cover stake proposed
            Self::ensure_can_make_stake_imbalance(
                vec![&opt_role_stake_balance, &opt_application_stake_balance],
                &source_account)
                .map_err(|_err| MSG_INSUFFICIENT_BALANCE_TO_APPLY)?;

            // Ensure application text is valid
            Self::ensure_worker_application_text_is_valid(&human_readable_text)?;

            // Ensure application can actually be added
            ensure_on_wrapped_error!(
                hiring::Module::<T>::ensure_can_add_application(worker_opening.opening_id, opt_role_stake_balance, opt_application_stake_balance)
            )?;

            // Ensure member does not have an active application to this opening
            Self::ensure_member_has_no_active_application_on_opening(
                worker_opening.worker_applications,
                member_id
            )?;

            // mutation

            // Make imbalances for staking
            let opt_role_stake_imbalance = Self::make_stake_opt_imbalance(&opt_role_stake_balance, &source_account);
            let opt_application_stake_imbalance = Self::make_stake_opt_imbalance(&opt_application_stake_balance, &source_account);

            // Call hiring module to add application
            let add_application_result = hiring::Module::<T>::add_application(
                worker_opening.opening_id,
                opt_role_stake_imbalance,
                opt_application_stake_imbalance,
                human_readable_text
            );

            // Has to hold
            assert!(add_application_result.is_ok());

            let application_id = add_application_result.unwrap().application_id_added;

            // Get id of new worker application
            let new_worker_application_id = NextWorkerApplicationId::<T, I>::get();

            // Make worker application
            let worker_application = WorkerApplication::new(&role_account, &worker_opening_id, &member_id, &application_id);

            // Store application
            WorkerApplicationById::<T, I>::insert(new_worker_application_id, worker_application);

            // Update next worker application identifier value
            NextWorkerApplicationId::<T, I>::mutate(|id| *id += <WorkerApplicationId<T> as One>::one());

            // Add application to set of application in worker opening
            WorkerOpeningById::<T, I>::mutate(worker_opening_id, |worker_opening| {
                worker_opening.worker_applications.insert(new_worker_application_id);
            });

            // Trigger event
            Self::deposit_event(RawEvent::AppliedOnWorkerOpening(worker_opening_id, new_worker_application_id));
        }

        pub fn withdraw_worker_application(
            origin,
            worker_application_id: WorkerApplicationId<T>
        ) {
            // Ensuring worker application actually exists
            let (worker_application, _, worker_opening) = Self::ensure_worker_application_exists(&worker_application_id)?;

            // Ensure that it is signed
            let signer_account = ensure_signed(origin)?;

            // Ensure that signer is applicant role account
            ensure!(
                signer_account == worker_application.role_account,
                MSG_ORIGIN_IS_NOT_APPLICANT
            );

            // Attempt to deactivate application
            // NB: Combined ensure check and mutation in hiring module
            ensure_on_wrapped_error!(
                hiring::Module::<T>::deactive_application(
                    worker_application.application_id,
                    worker_opening.policy_commitment.exit_worker_role_application_stake_unstaking_period,
                    worker_opening.policy_commitment.exit_worker_role_stake_unstaking_period
                )
            )?;

            // mutation

            // Trigger event
            Self::deposit_event(RawEvent::WorkerApplicationWithdrawn(worker_application_id));
        }

        pub fn terminate_worker_application(
            origin,
            worker_application_id: WorkerApplicationId<T>
        ) {

            // Ensure lead is set and is origin signer
            Self::ensure_origin_is_set_lead(origin)?;

            // Ensuring worker application actually exists
            let (worker_application, _, worker_opening) = Self::ensure_worker_application_exists(&worker_application_id)?;

            // Attempt to deactivate application
            // NB: Combined ensure check and mutation in hiring module
            ensure_on_wrapped_error!(
                hiring::Module::<T>::deactive_application(
                    worker_application.application_id,
                    worker_opening.policy_commitment.terminate_worker_application_stake_unstaking_period,
                    worker_opening.policy_commitment.terminate_worker_role_stake_unstaking_period
                )
            )?;

            // mutation

            // Trigger event
            Self::deposit_event(RawEvent::WorkerApplicationTerminated(worker_application_id));
        }

        /// Begin reviewing, and therefore not accepting new applications.
        pub fn begin_worker_applicant_review(origin, worker_opening_id: WorkerOpeningId<T>) {

            // Ensure lead is set and is origin signer
            Self::ensure_origin_is_set_lead(origin)?;

            // Ensure opening exists
            // NB: Even though call to hiring modul will have implicit check for
            // existence of opening as well, this check is to make sure that the opening is for
            // this working group, not something else.
            let (worker_opening, _opening) = Self::ensure_worker_opening_exists(&worker_opening_id)?;

            // Attempt to begin review of applications
            // NB: Combined ensure check and mutation in hiring module
            ensure_on_wrapped_error!(
                hiring::Module::<T>::begin_review(worker_opening.opening_id)
                )?;

            // mutation

            // Trigger event
            Self::deposit_event(RawEvent::BeganWorkerApplicationReview(worker_opening_id));
        }

                /// Fill opening for worker
        pub fn fill_worker_opening(
            origin,
            worker_opening_id: WorkerOpeningId<T>,
            successful_worker_application_ids: WorkerApplicationIdSet<T>,
//            reward_policy: Option<RewardPolicy<minting::BalanceOf<T>, T::BlockNumber>>
        ) {
            // Ensure lead is set and is origin signer
            Self::ensure_origin_is_set_lead(origin)?;

            // Ensure worker opening exists
            let (worker_opening, _) = Self::ensure_worker_opening_exists(&worker_opening_id)?;

            // Make iterator over successful worker application
            let successful_iter = successful_worker_application_ids
                                    .iter()
                                    // recover worker application from id
                                    .map(|worker_application_id| { Self::ensure_worker_application_exists(worker_application_id)})
                                    // remove Err cases, i.e. non-existing applications
                                    .filter_map(|result| result.ok());

            // Count number of successful workers provided
            let num_provided_successful_worker_application_ids = successful_worker_application_ids.len();

            // Ensure all worker applications exist
            let number_of_successful_applications = successful_iter
                                                    .clone()
                                                    .count();

            ensure!(
                number_of_successful_applications == num_provided_successful_worker_application_ids,
                MSG_SUCCESSFUL_WORKER_APPLICATION_DOES_NOT_EXIST
            );

            // Attempt to fill opening
            let successful_application_ids = successful_iter
                                            .clone()
                                            .map(|(successful_worker_application, _, _)| successful_worker_application.application_id)
                                            .collect::<BTreeSet<_>>();

            // NB: Combined ensure check and mutation in hiring module
            ensure_on_wrapped_error!(
                hiring::Module::<T>::fill_opening(
                    worker_opening.opening_id,
                    successful_application_ids,
                    worker_opening.policy_commitment.fill_opening_successful_applicant_application_stake_unstaking_period,
                    worker_opening.policy_commitment.fill_opening_failed_applicant_application_stake_unstaking_period,
                    worker_opening.policy_commitment.fill_opening_failed_applicant_role_stake_unstaking_period
                )
            )?;

            // TODO: should we implement this?
            // let create_reward_settings = if let Some(policy) = reward_policy {
            //     // A reward will need to be created so ensure our configured mint exists
            //     let mint_id = Self::mint();
            //
            //     ensure!(<minting::Mints<T>>::exists(mint_id), MSG_FILL_WORKER_OPENING_MINT_DOES_NOT_EXIST);
            //
            //     // Make sure valid parameters are selected for next payment at block number
            //     ensure!(policy.next_payment_at_block > <system::Module<T>>::block_number(), MSG_FILL_WORKER_OPENING_INVALID_NEXT_PAYMENT_BLOCK);
            //
            //     // The verified reward settings to use
            //     Some((mint_id, policy))
            // } else {
            //     None
            // };

            //
            // == MUTATION SAFE ==
            //

            let _current_block = <system::Module<T>>::block_number();

            // For each successful application
            // - create and hold on to worker
            // - register role with membership module

            let mut worker_application_id_to_worker_id = BTreeMap::new();

            successful_iter
            .clone()
            .for_each(|(successful_worker_application, id, _)| {
                // TODO: should we implement this?
                // // Create a reward relationship
                // let reward_relationship = if let Some((mint_id, checked_policy)) = create_reward_settings.clone() {
                //
                //     // Create a new recipient for the new relationship
                //     let recipient = <recurringrewards::Module<T>>::add_recipient();
                //
                //     // member must exist, since it was checked that it can enter the role
                //     let member_profile = <members::Module<T>>::member_profile(successful_worker_application.member_id).unwrap();
                //
                //     // rewards are deposited in the member's root account
                //     let reward_destination_account = member_profile.root_account;
                //
                //     // values have been checked so this should not fail!
                //     let relationship_id = <recurringrewards::Module<T>>::add_reward_relationship(
                //         mint_id,
                //         recipient,
                //         reward_destination_account,
                //         checked_policy.amount_per_payout,
                //         checked_policy.next_payment_at_block,
                //         checked_policy.payout_interval,
                //     ).expect("Failed to create reward relationship!");
                //
                //     Some(relationship_id)
                // } else {
                //     None
                // };

                // Get possible stake for role
                let application = hiring::ApplicationById::<T>::get(successful_worker_application.application_id);

                // Staking profile for worker
                let _stake_profile =
                    if let Some(ref stake_id) = application.active_role_staking_id {

                        Some(
                            WorkerRoleStakeProfile::new(
                                stake_id,
                                &worker_opening.policy_commitment.terminate_worker_role_stake_unstaking_period,
                                &worker_opening.policy_commitment.exit_worker_role_stake_unstaking_period
                            )
                        )
                    } else {
                        None
                    };

                // Get worker id
                let new_worker_id = <NextWorkerId<T, I>>::get();

                // Make and add new principal
 //               let principal_id = Self::add_new_principal(&Principal::Worker(new_worker_id));

                // TODO: should we implement this?
                // Construct worker
                let worker = Worker::new(
                    &(successful_worker_application.role_account),
//                    &reward_relationship,
//                    &stake_profile,
//                    &WorkerRoleStage::Active,
//                    &WorkerInduction::new(&lead_id, &id, &current_block),
//                    &principal_id
                );

                // Store worker
                <WorkerById<T, I>>::insert(new_worker_id, worker);

                // TODO: should we implement this?
                // // Register role on member
                // let registered_role = members::Module::<T>::register_role_on_member(
                //     successful_worker_application.member_id,
                //     &role_types::ActorInRole::new(role_types::Role::Worker, new_worker_id)
                // ).is_ok();

                //assert!(registered_role);

                // Update next worker id
                <NextWorkerId<T, I>>::mutate(|id| *id += <WorkerId<T> as One>::one());

                worker_application_id_to_worker_id.insert(id, new_worker_id);
            });

            // Trigger event
            Self::deposit_event(RawEvent::WorkerOpeningFilled(worker_opening_id, worker_application_id_to_worker_id));
        }
    }
}

impl<Origin, T, I> EnsureOrigin<Origin> for Module<T, I>
where
    Origin: Into<Result<RawOrigin<T::AccountId>, Origin>> + From<RawOrigin<T::AccountId>>,
    T: Trait<I>,
    I: Instance,
{
    type Success = ();

    fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(account_id) => {
                Self::ensure_is_lead_account(account_id).map_err(|_| RawOrigin::None.into())
            }
            _ => Err(RawOrigin::None.into()),
        })
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    /// Checks that provided lead account id belongs to the current bureaucracy leader
    pub fn ensure_is_lead_account(lead_account_id: T::AccountId) -> Result<(), &'static str> {
        let lead = <CurrentLead<T, I>>::get();

        if let Some(lead) = lead {
            if lead.role_account_id != lead_account_id {
                return Err(MSG_IS_NOT_LEAD_ACCOUNT);
            }
        } else {
            return Err(MSG_CURRENT_LEAD_NOT_SET);
        }

        Ok(())
    }

    fn ensure_opening_human_readable_text_is_valid(text: &[u8]) -> dispatch::Result {
        <OpeningHumanReadableText<I>>::get().ensure_valid(
            text.len(),
            MSG_OPENING_TEXT_TOO_SHORT,
            MSG_OPENING_TEXT_TOO_LONG,
        )
    }

    fn ensure_origin_is_set_lead(origin: T::Origin) -> Result<(), &'static str> {
        // Ensure is signed
        let signer = ensure_signed(origin)?;

        Self::ensure_is_lead_account(signer)
    }

    fn ensure_worker_opening_exists(
        worker_opening_id: &WorkerOpeningId<T>,
    ) -> Result<WorkerOpeningInfo<T>, &'static str> {
        ensure!(
            WorkerOpeningById::<T, I>::exists(worker_opening_id),
            MSG_WORKER_OPENING_DOES_NOT_EXIST
        );

        let worker_opening = WorkerOpeningById::<T, I>::get(worker_opening_id);

        let opening = hiring::OpeningById::<T>::get(worker_opening.opening_id);

        Ok((worker_opening, opening))
    }

    fn make_stake_opt_imbalance(
        opt_balance: &Option<BalanceOf<T>>,
        source_account: &T::AccountId,
    ) -> Option<NegativeImbalance<T>> {
        if let Some(balance) = opt_balance {
            let withdraw_result = CurrencyOf::<T>::withdraw(
                source_account,
                *balance,
                WithdrawReasons::all(),
                ExistenceRequirement::AllowDeath,
            );

            assert!(withdraw_result.is_ok());

            withdraw_result.ok()
        } else {
            None
        }
    }

    fn ensure_member_has_no_active_application_on_opening(
        worker_applications: WorkerApplicationIdSet<T>,
        member_id: T::MemberId,
    ) -> Result<(), &'static str> {
        for worker_application_id in worker_applications {
            let worker_application = WorkerApplicationById::<T, I>::get(worker_application_id);
            // Look for application by the member for the opening
            if worker_application.member_id != member_id {
                continue;
            }
            // Get application details
            let application = <hiring::ApplicationById<T>>::get(worker_application.application_id);
            // Return error if application is in active stage
            if application.stage == hiring::ApplicationStage::Active {
                return Err(MSG_MEMBER_HAS_ACTIVE_APPLICATION_ON_OPENING);
            }
        }
        // Member does not have any active applications to the opening
        Ok(())
    }

    fn ensure_worker_application_text_is_valid(text: &[u8]) -> dispatch::Result {
        <WorkerApplicationHumanReadableText<I>>::get().ensure_valid(
            text.len(),
            MSG_WORKER_APPLICATION_TEXT_TOO_SHORT,
            MSG_WORKER_APPLICATION_TEXT_TOO_LONG,
        )
    }

    // CRITICAL:
    // https://github.com/Joystream/substrate-runtime-joystream/issues/92
    // This assumes that ensure_can_withdraw can be don
    // for a sum of balance that later will be actually withdrawn
    // using individual terms in that sum.
    // This needs to be fully checked across all possibly scenarios
    // of actual balance, minimum balance limit, reservation, vesting and locking.
    fn ensure_can_make_stake_imbalance(
        opt_balances: Vec<&Option<BalanceOf<T>>>,
        source_account: &T::AccountId,
    ) -> Result<(), &'static str> {
        let zero_balance = <BalanceOf<T> as Zero>::zero();

        // Total amount to be staked
        let total_amount = opt_balances.iter().fold(zero_balance, |sum, opt_balance| {
            sum + if let Some(balance) = opt_balance {
                *balance
            } else {
                zero_balance
            }
        });

        if total_amount > zero_balance {
            // Ensure that
            if CurrencyOf::<T>::free_balance(source_account) < total_amount {
                Err(MSG_INSUFFICIENT_BALANCE_TO_COVER_STAKE)
            } else {
                let new_balance = CurrencyOf::<T>::free_balance(source_account) - total_amount;

                CurrencyOf::<T>::ensure_can_withdraw(
                    source_account,
                    total_amount,
                    WithdrawReasons::all(),
                    new_balance,
                )
            }
        } else {
            Ok(())
        }
    }

    fn ensure_worker_application_exists(
        worker_application_id: &WorkerApplicationId<T>,
    ) -> Result<WorkerApplicationInfo<T>, &'static str> {
        ensure!(
            WorkerApplicationById::<T, I>::exists(worker_application_id),
            MSG_WORKER_APPLICATION_DOES_NOT_EXIST
        );

        let worker_application = WorkerApplicationById::<T, I>::get(worker_application_id);

        let worker_opening = WorkerOpeningById::<T, I>::get(worker_application.worker_opening_id);

        Ok((worker_application, *worker_application_id, worker_opening))
    }
}
