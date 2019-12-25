// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use runtime_primitives::traits::Zero;
use runtime_primitives::traits::{MaybeSerialize, Member, One, SimpleArithmetic};

use srml_support::traits::Currency;
use srml_support::{decl_module, decl_storage, ensure, Parameter};

use crate::sr_api_hidden_includes_decl_storage::hidden_include::traits::Imbalance;

use rstd::collections::btree_map::BTreeMap;
use rstd::collections::btree_set::BTreeSet;
use rstd::iter::Iterator;
use rstd::prelude::*;

use codec::Codec;
use system;

mod hiring;
#[macro_use]
mod macroes;
mod mock;
mod test;

pub use hiring::*;
use stake;

/// ...
pub trait ApplicationDeactivatedHandler<T: Trait> {
    /// An application, with the given id, was fully deactivated, with the
    /// given cause, and was put in the inactive state.
    fn deactivated(application_id: &T::ApplicationId, cause: hiring::ApplicationDeactivationCause);
}

pub trait Trait: system::Trait + stake::Trait + Sized {
    type OpeningId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq;

    type ApplicationId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq;

    /// Type that will handle various staking events
    type ApplicationDeactivatedHandler: ApplicationDeactivatedHandler<Self>;
}

/// Helper implementation so we can provide multiple handlers by grouping handlers in tuple pairs.
/// For example for three handlers, A, B and C we can set the StakingEventHandler type on the trait to:
/// type StakingEventHandler = ((A, B), C)
///
impl<T: Trait> ApplicationDeactivatedHandler<T> for () {
    fn deactivated(
        _application_id: &T::ApplicationId,
        _cause: hiring::ApplicationDeactivationCause,
    ) {
    }
}

pub type BalanceOf<T> =
    <<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub type NegativeImbalance<T> =
    <<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

decl_storage! {
    trait Store for Module<T: Trait> as Hiring {

        /// Openings.
        pub OpeningById get(opening_by_id): linked_map T::OpeningId => Opening<BalanceOf<T>, T::BlockNumber, T::ApplicationId>;

        /// Identifier for next opening to be added.
        pub NextOpeningId get(next_opening_id): T::OpeningId;

        /// Applications
        pub ApplicationById get(application_by_id): linked_map T::ApplicationId => Application<T::OpeningId, T::BlockNumber, T::StakeId>;

        /// Identifier for next application to be added.
        pub NextApplicationId get(next_application_id): T::ApplicationId;

        /// Internal purpose of given stake, i.e. fro what application, and whether for the role or for the application.
        pub ApplicationIdByStakingId get(stake_purpose_by_staking_id): linked_map T::StakeId => T::ApplicationId;

    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn on_finalize(now: T::BlockNumber) {

            //
            // == MUTATION SAFE ==
            //

            // NB: This routine is plauged by this problem
            // https://github.com/Joystream/joystream/issues/126#issuecomment-542268343 ,
            // would be much cleaner otherwise.

            // Compute iterator of openings waiting to begin
            let openings_ready_to_accept_applications_iter =
                <OpeningById<T>>::enumerate()
                .filter_map(|(opening_id, opening)| {

                    if let hiring::OpeningStage::WaitingToBegin {
                        begins_at_block
                    } = opening.stage {

                        if begins_at_block == now {
                            Some((
                                opening_id,
                                opening,
                                begins_at_block
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

            // ...
            for (opening_id, opening, _) in openings_ready_to_accept_applications_iter {

                // NB: Shows issue https://github.com/Joystream/substrate-hiring-module/issues/5
                let opening_accepting_applications = hiring::Opening{
                    stage : hiring::OpeningStage::Active {
                        stage: hiring::ActiveOpeningStage::AcceptingApplications {
                            started_accepting_applicants_at_block: now
                        },
                        applications_added: BTreeSet::new(),
                        active_application_count: 0,
                        unstaking_application_count: 0,
                        deactivated_application_count: 0
                    },
                    ..(opening.clone())
                };

                <OpeningById<T>>::insert(opening_id, opening_accepting_applications);
            }

            // Compute iterator of openings in expired review period
            let openings_in_expired_review_period_iter =
                <OpeningById<T>>::enumerate()
                .filter_map(|(opening_id, opening)| {

                    if let hiring::OpeningStage::Active {
                        ref stage,
                        ref applications_added,
                        ref active_application_count,
                        ref unstaking_application_count,
                        ref deactivated_application_count
                    } = opening.stage {

                        if let hiring::ActiveOpeningStage::ReviewPeriod {
                                ref started_accepting_applicants_at_block,
                                ref started_review_period_at_block
                            } = stage {

                                Some((
                                    opening_id,
                                    opening.clone(),
                                    (
                                        stage.clone(),
                                        applications_added.clone(),
                                        *active_application_count,
                                        *unstaking_application_count,
                                        *deactivated_application_count,
                                        *started_accepting_applicants_at_block,
                                        *started_review_period_at_block
                                    )
                                ))

                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

            // ...
            for (opening_id,
                opening,
                (
                    _stage,
                    applications_added,
                    _active_application_count,
                    _unstaking_application_count,
                    _deactivated_application_count,
                    started_accepting_applicants_at_block,
                    started_review_period_at_block
                )) in openings_in_expired_review_period_iter {

                //
                // Deactivat all applications that are part of this opening
                //

                // Get unstaking periods
                let application_stake_unstaking_period = StakingPolicy::opt_staking_policy_to_review_period_expired_unstaking_period(&opening.application_staking_policy);
                let role_stake_unstaking_period = StakingPolicy::opt_staking_policy_to_review_period_expired_unstaking_period(&opening.role_staking_policy);


                // Get applications
                let applications_map = Self::application_id_iter_to_map(applications_added.iter());

                // Deactivate
                Self::initiate_application_deactivations(
                    &applications_map,
                    application_stake_unstaking_period,
                    role_stake_unstaking_period,
                    hiring::ApplicationDeactivationCause::ReviewPeriodExpired
                );

                // NB: Shows issue https://github.com/Joystream/substrate-hiring-module/issues/5
                let opening_accepting_applications = hiring::Opening {

                    stage: hiring::OpeningStage::Active {

                        stage: hiring::ActiveOpeningStage::Deactivated {

                            cause: hiring::OpeningDeactivationCause::ReviewPeriodExpired,
                            deactivated_at_block: now,
                            started_accepting_applicants_at_block: started_accepting_applicants_at_block,
                            started_review_period_at_block: Some(started_review_period_at_block),
                        },

                        applications_added: BTreeSet::new(),
                        active_application_count: 0,
                        unstaking_application_count: 0,
                        deactivated_application_count: 0
                    },

                    ..(opening.clone())
                };

                <OpeningById<T>>::insert(opening_id, opening_accepting_applications);

            }
        }
    }
}

/// Error due to attempting to fill an opening.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum FillOpeningError<T: Trait> {
    OpeningDoesNotExist,
    OpeningNotInReviewPeriodStage,
    UnstakingPeriodTooShort(StakePurpose, ApplicationOutcomeInFilledOpening),
    RedundantUnstakingPeriodProvided(StakePurpose, ApplicationOutcomeInFilledOpening),
    ApplicationDoesNotExist(T::ApplicationId),
    ApplicationNotInActiveStage(T::ApplicationId),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct DestructuredApplicationCanBeAddedEvaluation<T: Trait> {
    pub opening: Opening<BalanceOf<T>, T::BlockNumber, T::ApplicationId>,

    pub active_stage: ActiveOpeningStage<T::BlockNumber>,

    pub applications_added: BTreeSet<T::ApplicationId>,

    pub active_application_count: u32,

    pub unstaking_application_count: u32,

    pub deactivated_application_count: u32,

    pub would_get_added_success: ApplicationAddedSuccess<T>,
}

impl<T: Trait> Module<T> {
    pub fn add_opening(
        activate_at: ActivateOpeningAt<T::BlockNumber>,
        max_review_period_length: T::BlockNumber,
        application_rationing_policy: Option<ApplicationRationingPolicy>,
        application_staking_policy: Option<StakingPolicy<BalanceOf<T>, T::BlockNumber>>,
        role_staking_policy: Option<StakingPolicy<BalanceOf<T>, T::BlockNumber>>,
        human_readable_text: Vec<u8>,
    ) -> Result<T::OpeningId, AddOpeningError> {
        let current_block_height = <system::Module<T>>::block_number();

        Opening::<BalanceOf<T>, T::BlockNumber, T::ApplicationId>::ensure_can_add_opening(
            current_block_height,
            activate_at.clone(),
            T::Currency::minimum_balance(),
            application_staking_policy.clone(),
            role_staking_policy.clone(),
        )?;

        // == MUTATION SAFE ==

        let new_opening = hiring::Opening::new(
            current_block_height,
            activate_at,
            max_review_period_length,
            application_rationing_policy,
            application_staking_policy,
            role_staking_policy,
            human_readable_text,
        );

        // Get Id for new opening
        let new_opening_id = <NextOpeningId<T>>::get();

        // Insert opening in storage
        <OpeningById<T>>::insert(new_opening_id, new_opening);

        // Update NextOpeningId counter
        <NextOpeningId<T>>::mutate(|id| *id += T::OpeningId::one());

        // Return
        Ok(new_opening_id)
    }

    /// Cancels opening with given identifier, using provided unstaking periods for
    /// application and role, as necesary.
    pub fn cancel_opening(
        opening_id: T::OpeningId,
        application_stake_unstaking_period: Option<T::BlockNumber>,
        role_stake_unstaking_period: Option<T::BlockNumber>,
    ) -> Result<OpeningCancelled, CancelOpeningError> {
        // Ensure that the opening exists
        let opening =
            ensure_opening_exists!(T, opening_id, CancelOpeningError::OpeningDoesNotExist)?;

        // Opening is in stage Active.{AcceptingApplications or ReviewPeriod}

        let (
            active_stage,
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
        ) = ensure_opening_is_active!(
            opening.stage,
            CancelOpeningError::OpeningNotInCancellableStage
        )?;

        //
        let current_block_height = <system::Module<T>>::block_number(); // move later!

        let new_active_stage = match active_stage {
            ActiveOpeningStage::AcceptingApplications {
                started_accepting_applicants_at_block,
            } => Ok(ActiveOpeningStage::Deactivated {
                cause: OpeningDeactivationCause::CancelledAcceptingApplications,
                deactivated_at_block: current_block_height,
                started_accepting_applicants_at_block,
                started_review_period_at_block: None,
            }),
            ActiveOpeningStage::ReviewPeriod {
                started_accepting_applicants_at_block,
                started_review_period_at_block,
            } => Ok(ActiveOpeningStage::Deactivated {
                cause: OpeningDeactivationCause::CancelledInReviewPeriod,
                deactivated_at_block: current_block_height,
                started_accepting_applicants_at_block,
                started_review_period_at_block: Some(started_review_period_at_block),
            }),
            ActiveOpeningStage::Deactivated { .. } => {
                Err(CancelOpeningError::OpeningNotInCancellableStage)
            }
        }?;

        // Ensure unstaking periods are OK.
        ensure_opt_unstaking_period_is_ok!(
            application_stake_unstaking_period,
            opening.application_staking_policy,
            CancelOpeningError::UnstakingPeriodTooShort(StakePurpose::Application),
            CancelOpeningError::RedundantUnstakingPeriodProvided(StakePurpose::Application)
        )?;

        ensure_opt_unstaking_period_is_ok!(
            role_stake_unstaking_period,
            opening.role_staking_policy,
            CancelOpeningError::UnstakingPeriodTooShort(StakePurpose::Role),
            CancelOpeningError::RedundantUnstakingPeriodProvided(StakePurpose::Role)
        )?;

        //
        // == MUTATION SAFE ==
        //

        // Create and store new cancelled opening
        let new_opening = Opening {
            stage: hiring::OpeningStage::Active {
                stage: new_active_stage,
                applications_added: applications_added.clone(),
                active_application_count,
                unstaking_application_count,
                deactivated_application_count,
            },
            ..opening
        };

        OpeningById::<T>::insert(opening_id, new_opening);

        // Map with applications
        let applications_map = Self::application_id_iter_to_map(applications_added.iter());

        // Initiate deactivation of all active applications
        let net_result = Self::initiate_application_deactivations(
            &applications_map,
            application_stake_unstaking_period,
            role_stake_unstaking_period,
            hiring::ApplicationDeactivationCause::OpeningCancelled,
        );

        // Return
        Ok(OpeningCancelled {
            number_of_unstaking_applications: net_result.number_of_unstaking_applications,
            number_of_deactivated_applications: net_result.number_of_deactivated_applications,
        })
    }

    pub fn begin_accepting_applications(
        opening_id: T::OpeningId,
    ) -> Result<(), BeginAcceptingApplicationsError> {
        // Ensure that the opening exists
        let opening = ensure_opening_exists!(
            T,
            opening_id,
            BeginAcceptingApplicationsError::OpeningDoesNotExist
        )?;

        // Ensure that it is the waiting to begin stage
        opening.stage.ensure_opening_stage_is_waiting_to_begin(
            BeginAcceptingApplicationsError::OpeningIsNotInWaitingToBeginStage,
        )?;

        //
        // == MUTATION SAFE ==
        //

        let current_block_height = <system::Module<T>>::block_number();

        // Update state of opening
        let new_opening = opening.clone_with_new_active_opening_stage(
            hiring::ActiveOpeningStage::AcceptingApplications {
                started_accepting_applicants_at_block: current_block_height,
            },
        );

        // Write back opening
        <OpeningById<T>>::insert(opening_id, new_opening);

        // DONE
        Ok(())
    }

    pub fn begin_review(opening_id: T::OpeningId) -> Result<(), BeginReviewError> {
        // Ensure that the opening exists
        let opening = ensure_opening_exists!(T, opening_id, BeginReviewError::OpeningDoesNotExist)?;

        // Opening is accepting applications
        let (active_stage, _, _, _, _) = ensure_opening_is_active!(
            opening.stage,
            BeginReviewError::OpeningNotInAcceptingApplicationsStage
        )?;

        let started_accepting_applicants_at_block = ensure_active_opening_is_accepting_applications!(
            active_stage,
            BeginReviewError::OpeningNotInAcceptingApplicationsStage
        )?;

        //
        // == MUTATION SAFE ==
        //

        let current_block_height = <system::Module<T>>::block_number();

        let new_opening =
            opening.clone_with_new_active_opening_stage(hiring::ActiveOpeningStage::ReviewPeriod {
                started_accepting_applicants_at_block,
                started_review_period_at_block: current_block_height,
            });

        // Update to new opening
        <OpeningById<T>>::insert(opening_id, new_opening);

        Ok(())
    }

    /// Fill an opening, identifed with `opening_id`, currently in the review period.
    ///
    /// TODO: Name _a bit_ too long? but descriptive though...
    pub fn fill_opening(
        opening_id: T::OpeningId,
        successful_applications: BTreeSet<T::ApplicationId>,
        opt_successful_applicant_application_stake_unstaking_period: Option<T::BlockNumber>,
        opt_failed_applicant_application_stake_unstaking_period: Option<T::BlockNumber>,
        /* this parameter does not make sense? opt_successful_applicant_role_stake_unstaking_period: Option<T::BlockNumber>, */
        opt_failed_applicant_role_stake_unstaking_period: Option<T::BlockNumber>,
    ) -> Result<(), FillOpeningError<T>> {
        // Ensure that the opening exists
        let opening = ensure_opening_exists!(T, opening_id, FillOpeningError::OpeningDoesNotExist)?;

        let (
            active_stage,
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
        ) = ensure_opening_is_active!(
            opening.stage,
            FillOpeningError::OpeningNotInReviewPeriodStage
        )?;

        // Ensure opening is in review period
        let (started_accepting_applicants_at_block, started_review_period_at_block) = active_stage
            .ensure_active_opening_is_in_review_period(
                FillOpeningError::OpeningNotInReviewPeriodStage,
            )?;

        //
        // Ensure that all unstaking periods are neither too short (0) nor redundant.
        //

        ensure_opt_unstaking_period_is_ok!(
            opt_successful_applicant_application_stake_unstaking_period,
            opening.application_staking_policy,
            FillOpeningError::UnstakingPeriodTooShort(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Success
            ),
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Success
            )
        )?;

        ensure_opt_unstaking_period_is_ok!(
            opt_failed_applicant_application_stake_unstaking_period,
            opening.application_staking_policy,
            FillOpeningError::UnstakingPeriodTooShort(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Failure
            ),
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Failure
            )
        )?;

        ensure_opt_unstaking_period_is_ok!(
            opt_failed_applicant_role_stake_unstaking_period,
            opening.role_staking_policy,
            FillOpeningError::UnstakingPeriodTooShort(
                StakePurpose::Role,
                ApplicationOutcomeInFilledOpening::Failure
            ),
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Role,
                ApplicationOutcomeInFilledOpening::Failure
            )
        )?;

        // Ensure that all provided application ids are in fact valid
        let invalid_application_ids = successful_applications
            .clone()
            .iter()
            .filter_map(|application_id| {
                if !<ApplicationById<T>>::exists(application_id) {
                    Some(*application_id)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<T::ApplicationId>>();

        if !invalid_application_ids.is_empty() {
            let first_missing_application_id = invalid_application_ids.iter().next();

            if let Some(application_id) = first_missing_application_id {
                return Err(FillOpeningError::ApplicationDoesNotExist(*application_id));
            }
        }

        // Ensure that all claimed successful applications actually exist, and collect@
        // underlying applications into a map.
        let successful_applications_map = successful_applications
            .clone()
            .iter()
            .map(|application_id| {
                assert!(<ApplicationById<T>>::exists(application_id));

                let application = <ApplicationById<T>>::get(application_id);

                (*application_id, application)
            })
            .collect::<BTreeMap<T::ApplicationId, _>>();

        // Ensure that all successful applications are actually active.
        let opt_non_active_application = successful_applications_map
            .iter()
            .find(|(_application_id, application)| application.stage != ApplicationStage::Active);

        if let Some((application_id, _application)) = opt_non_active_application {
            return Err(FillOpeningError::ApplicationNotInActiveStage(
                *application_id,
            ));
        }

        //
        // == MUTATION SAFE ==
        //

        // Deactivate all successful applications, with cause being hired
        for (application_id, application) in &successful_applications_map {
            Self::try_to_initiate_application_deactivation(
                &application,
                *application_id,
                opt_successful_applicant_application_stake_unstaking_period,
                None, // <= We do not unstake role stake for successful applicants, opt_successful_applicant_role_stake_unstaking_period,
                hiring::ApplicationDeactivationCause::Hired,
            );
        }

        // Deactivate all unsuccessful applications, with cause being not being hired.

        // First get all failed applications by their id.
        let failed_applications_map = applications_added
            .difference(&successful_applications)
            .cloned()
            .map(|application_id| {
                let application = <ApplicationById<T>>::get(application_id);

                (application_id, application)
            })
            .collect::<BTreeMap<_, _>>();

        // Deactivate all successful applications, with cause being hired
        for (application_id, application) in &failed_applications_map {
            Self::try_to_initiate_application_deactivation(
                &application,
                *application_id,
                opt_failed_applicant_application_stake_unstaking_period,
                opt_failed_applicant_role_stake_unstaking_period,
                hiring::ApplicationDeactivationCause::NotHired,
            );
        }

        // Grab current block height
        let current_block_height = <system::Module<T>>::block_number();

        // Deactivate opening itself
        let new_opening = hiring::Opening {
            stage: hiring::OpeningStage::Active {
                stage: hiring::ActiveOpeningStage::Deactivated {
                    cause: OpeningDeactivationCause::Filled,
                    deactivated_at_block: current_block_height,
                    started_accepting_applicants_at_block,
                    started_review_period_at_block: Some(started_review_period_at_block),
                },
                //.. <== cant use here, same issue
                applications_added,
                active_application_count,
                unstaking_application_count,
                deactivated_application_count,
            },
            ..opening
        };

        // Write back new opening
        <OpeningById<T>>::insert(opening_id, new_opening);

        // DONE
        Ok(())
    }

    /// Adds a new application on the given opening, and begins staking for
    /// the role, the application or both possibly.
    ///
    /// Returns ..
    pub fn ensure_can_add_application(
        opening_id: T::OpeningId,
        opt_role_stake_balance: Option<BalanceOf<T>>,
        opt_application_stake_balance: Option<BalanceOf<T>>,
    ) -> Result<DestructuredApplicationCanBeAddedEvaluation<T>, AddApplicationError> {
        // Ensure that the opening exists
        let opening =
            ensure_opening_exists!(T, opening_id, AddApplicationError::OpeningDoesNotExist)?;

        // Ensure that proposed stakes match the policy of the opening.
        let opt_role_stake_balance = ensure_stake_balance_matches_staking_policy!(
            &opt_role_stake_balance,
            &opening.role_staking_policy,
            AddApplicationError::StakeMissingWhenRequired(StakePurpose::Role),
            AddApplicationError::StakeProvidedWhenRedundant(StakePurpose::Role),
            AddApplicationError::StakeAmountTooLow(StakePurpose::Role)
        )?;

        let opt_application_stake_balance = ensure_stake_balance_matches_staking_policy!(
            &opt_application_stake_balance,
            &opening.application_staking_policy,
            AddApplicationError::StakeMissingWhenRequired(StakePurpose::Application),
            AddApplicationError::StakeProvidedWhenRedundant(StakePurpose::Application),
            AddApplicationError::StakeAmountTooLow(StakePurpose::Application)
        )?;

        // Opening is accepting applications

        let (
            active_stage,
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
        ) = ensure_opening_is_active!(
            opening.stage,
            AddApplicationError::OpeningNotInAcceptingApplicationsStage
        )?;

        active_stage.ensure_active_opening_is_accepting_applications(
            AddApplicationError::OpeningNotInAcceptingApplicationsStage,
        )?;

        // Ensure that the new application would actually make it
        let would_get_added_success = ensure_application_would_get_added!(
            &opening.application_rationing_policy,
            &applications_added,
            &opt_role_stake_balance,
            &opt_application_stake_balance,
            AddApplicationError::NewApplicationWasCrowdedOut
        )?;

        Ok(DestructuredApplicationCanBeAddedEvaluation {
            opening,
            active_stage,
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
            would_get_added_success,
        })
    }

    /// Adds a new application on the given opening, and begins staking for
    /// the role, the application or both possibly.
    ///
    /// Returns ..
    pub fn add_application(
        opening_id: T::OpeningId,
        opt_role_stake_imbalance: Option<NegativeImbalance<T>>,
        opt_application_stake_imbalance: Option<NegativeImbalance<T>>,
        human_readable_text: Vec<u8>,
    ) -> Result<ApplicationAdded<T::ApplicationId>, AddApplicationError> {
        let opt_role_stake_balance = if let Some(ref imbalance) = opt_role_stake_imbalance {
            Some(imbalance.peek())
        } else {
            None
        };

        let opt_application_stake_balance =
            if let Some(ref imbalance) = opt_application_stake_imbalance {
                Some(imbalance.peek())
            } else {
                None
            };

        let can_be_added_destructured = Self::ensure_can_add_application(
            opening_id,
            opt_role_stake_balance,
            opt_application_stake_balance,
        )?;

        //
        // == MUTATION SAFE ==
        //

        // If required, deactive another application that was crowded out.
        if let ApplicationAddedSuccess::CrowdsOutExistingApplication(
            id_of_croweded_out_application,
        ) = can_be_added_destructured.would_get_added_success
        {
            // Get relevant unstaking periods
            let opt_application_stake_unstaking_period =
                hiring::StakingPolicy::opt_staking_policy_to_crowded_out_unstaking_period(
                    &can_be_added_destructured.opening.application_staking_policy,
                );
            let opt_role_stake_unstaking_period =
                hiring::StakingPolicy::opt_staking_policy_to_crowded_out_unstaking_period(
                    &can_be_added_destructured.opening.role_staking_policy,
                );

            // Fetch application
            let crowded_out_application = <ApplicationById<T>>::get(id_of_croweded_out_application);

            // Initiate actual deactivation
            //
            // MUST not have been ignored, is runtime invariant, false means code is broken.
            // But should we do panic in runtime? Is there safer way?
            let deactivation_result = Self::try_to_initiate_application_deactivation(
                &crowded_out_application,
                id_of_croweded_out_application,
                opt_application_stake_unstaking_period,
                opt_role_stake_unstaking_period,
                hiring::ApplicationDeactivationCause::CrowdedOut,
            );

            assert_ne!(
                deactivation_result,
                ApplicationDeactivationInitationResult::Ignored
            );
        }

        // Get Id for this new application
        let new_application_id = <NextApplicationId<T>>::get();

        // Possibly initiate staking
        let active_role_staking_id =
            Self::infallible_opt_stake_initiation(opt_role_stake_imbalance, &new_application_id);
        let active_application_staking_id = Self::infallible_opt_stake_initiation(
            opt_application_stake_imbalance,
            &new_application_id,
        );

        // Stage of new application
        let application_stage = hiring::ApplicationStage::Active;

        // Grab current block height
        let current_block_height = <system::Module<T>>::block_number();

        // Compute index for this new application
        // TODO: fix so that `number_of_appliations_ever_added` can be invoked.
        let application_index_in_opening = can_be_added_destructured.active_application_count
            + can_be_added_destructured.unstaking_application_count
            + can_be_added_destructured.deactivated_application_count; // cant do this due to bad design of stage => opening.stage.number_of_appliations_ever_added();

        // Create a new application
        let new_application = hiring::Application {
            opening_id,
            application_index_in_opening,
            add_to_opening_in_block: current_block_height,
            active_role_staking_id,
            active_application_staking_id,
            stage: application_stage,
            human_readable_text,
        };

        // Insert into main application map
        <ApplicationById<T>>::insert(new_application_id, new_application);

        // Update next application id
        <NextApplicationId<T>>::mutate(|id| *id += One::one());

        // Update counter on opening

        let mut apps_added = can_be_added_destructured.applications_added;
        apps_added.insert(new_application_id);

        let new_active_stage;
        // Should reload after possible deactivation in try_to_initiate_application_deactivation
        let opening_needed_for_data = <OpeningById<T>>::get(opening_id);
        if let hiring::OpeningStage::Active {
            stage,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
            ..
        } = opening_needed_for_data.stage
        {
            /*
               TODO:
               Yet another instance of problems due to not following https://github.com/Joystream/joystream/issues/36#issuecomment-539567407
            */
            new_active_stage = hiring::OpeningStage::Active {
                stage,
                applications_added: apps_added,
                active_application_count: active_application_count + 1,
                unstaking_application_count,
                deactivated_application_count
            };
        } else {
            panic!("updated opening should be in active stage");
        }

        <OpeningById<T>>::mutate(opening_id, |opening| {
            opening.stage = new_active_stage;
        });

        // DONE
        let application_added_result = ApplicationAdded {
            application_id_added: new_application_id,
            application_id_crowded_out: match can_be_added_destructured.would_get_added_success {
                ApplicationAddedSuccess::CrowdsOutExistingApplication(id) => Some(id),
                _ => None,
            },
        };

        Ok(application_added_result)
    }

    /// Deactive an active application.
    ///
    /// Does currently not support slashing
    pub fn deactive_application(
        application_id: T::ApplicationId,
        application_stake_unstaking_period: Option<T::BlockNumber>,
        role_stake_unstaking_period: Option<T::BlockNumber>,
    ) -> Result<(), DeactivateApplicationError> {
        // Check that application id is valid, and if so,
        // grab corresponding application and opening.
        let (application, opening) = ensure_application_exists!(
            T,
            application_id,
            DeactivateApplicationError::ApplicationDoesNotExist,
            auto_fetch_opening
        )?;

        // Application is active
        ensure_eq!(
            application.stage,
            hiring::ApplicationStage::Active,
            DeactivateApplicationError::ApplicationNotActive
        );

        // Opening is accepting applications
        let (active_stage, ..) = ensure_opening_is_active!(
            opening.stage,
            DeactivateApplicationError::OpeningNotAcceptingApplications
        )?;

        active_stage.ensure_active_opening_is_accepting_applications(
            DeactivateApplicationError::OpeningNotAcceptingApplications,
        )?;

        // Ensure unstaking periods are OK.
        ensure_opt_unstaking_period_is_ok!(
            application_stake_unstaking_period,
            opening.application_staking_policy,
            DeactivateApplicationError::UnstakingPeriodTooShort(StakePurpose::Application),
            DeactivateApplicationError::RedundantUnstakingPeriodProvided(StakePurpose::Application)
        )?;

        ensure_opt_unstaking_period_is_ok!(
            role_stake_unstaking_period,
            opening.role_staking_policy,
            DeactivateApplicationError::UnstakingPeriodTooShort(StakePurpose::Role),
            DeactivateApplicationError::RedundantUnstakingPeriodProvided(StakePurpose::Role)
        )?;

        //
        // == MUTATION SAFE ==
        //

        // Deactive application
        let result = Self::try_to_initiate_application_deactivation(
            &application,
            application_id,
            application_stake_unstaking_period,
            role_stake_unstaking_period,
            hiring::ApplicationDeactivationCause::External,
        );

        assert_ne!(result, ApplicationDeactivationInitationResult::Ignored);

        // DONE
        Ok(())
    }

    /// The stake, with the given id, was unstaked.
    pub fn unstaked(stake_id: T::StakeId) {
        // Ignore unstaked
        if !<ApplicationIdByStakingId<T>>::exists(stake_id) {
            return;
        }

        // Get application
        let application_id = <ApplicationIdByStakingId<T>>::get(stake_id);

        assert!(<ApplicationById<T>>::exists(application_id));

        let application = <ApplicationById<T>>::get(application_id);

        // Make sure that we are actually unstaking, ignore otherwise.
        let (deactivation_initiated, cause) = if let ApplicationStage::Unstaking {
            deactivation_initiated,
            cause,
        } = application.stage
        {
            (deactivation_initiated, cause)
        } else {
            return;
        };

        //
        // == MUTATION SAFE ==
        //

        // Drop stake from stake to application map
        <ApplicationIdByStakingId<T>>::remove(stake_id);

        // New values for application stakes
        let new_active_role_staking_id = match application.active_role_staking_id {
            // If there is a match, reset.
            Some(id) => {
                if id == stake_id {
                    None
                } else {
                    Some(id)
                }
            }
            _ => None,
        };

        let new_active_application_staking_id = match application.active_application_staking_id {
            // If there is a match, reset.
            Some(id) => {
                if id == stake_id {
                    None
                } else {
                    Some(id)
                }
            }
            _ => None,
        };

        // Are we now done unstaking?
        // Is the case if thereare no application stakes set.
        let is_now_done_unstaking =
            new_active_role_staking_id.is_none() && new_active_application_staking_id.is_none();

        // Stage of application after unstaking.
        let new_stage =

            // If we are done unstaking, then we go to the inactive stage
            if is_now_done_unstaking {

                let current_block_height = <system::Module<T>>::block_number();

                ApplicationStage::Inactive {
                    deactivation_initiated,
                    deactivated: current_block_height,
                    cause
                }

            } else {
                application.stage
            };

        // New application computed
        let new_application = hiring::Application {
            active_role_staking_id: new_active_role_staking_id,
            active_application_staking_id: new_active_application_staking_id,
            stage: new_stage,
            ..application
        };

        // Update to new application
        <ApplicationById<T>>::insert(&application_id, new_application);

        // If the application is now finished compeleting any pending unstaking process,
        // then we need to update the opening counters, and make the deactivation callback.
        if is_now_done_unstaking {
            // Update Opening
            // We know the stage MUST be active, hence mutate is certain.
            <OpeningById<T>>::mutate(application.opening_id, |opening| {
                // NB: This ugly byref destructuring is same issue as pointed out multiple times now.
                if let hiring::OpeningStage::Active {
                    ref stage,
                    ref applications_added,
                    active_application_count,
                    unstaking_application_count,
                    deactivated_application_count,
                } = opening.stage
                {
                    opening.stage = hiring::OpeningStage::Active {
                        stage: stage.clone(),
                        applications_added: applications_added.clone(),
                        active_application_count,
                        unstaking_application_count: unstaking_application_count - 1,
                        deactivated_application_count: deactivated_application_count + 1,
                    };
                } else {
                    panic!("stage MUST be active")
                }
            });

            // Call handler
            T::ApplicationDeactivatedHandler::deactivated(&application_id, cause);
        }
    }
}

/*
 *  ======== ======== ======== ======== =======
 *  ======== PRIVATE TYPES AND METHODS ========
 *  ======== ======== ======== ======== =======
 */

/*
 * Used by multiple methods.
 */

struct ApplicationsDeactivationsInitiationResult {
    number_of_unstaking_applications: u32,
    number_of_deactivated_applications: u32,
}

#[derive(PartialEq, Debug)]
enum ApplicationDeactivationInitationResult {
    Ignored, // <= is there a case for kicking this out, making sure that initation cannot happen when it may fail?
    Unstaking,
    Deactivated,
}

pub type ApplicationBTreeMap<T> = BTreeMap<
    <T as Trait>::ApplicationId,
    hiring::Application<
        <T as Trait>::OpeningId,
        <T as system::Trait>::BlockNumber,
        <T as stake::Trait>::StakeId,
    >,
>;

impl<T: Trait> Module<T> {
    fn application_id_iter_to_map<'a>(
        application_id_iter: impl Iterator<Item = &'a T::ApplicationId>,
    ) -> ApplicationBTreeMap<T> {
        application_id_iter
            .map(|application_id| {
                let application = <ApplicationById<T>>::get(application_id);

                (*application_id, application)
            })
            .collect::<BTreeMap<_, _>>()
    }
}

impl<T: Trait> Module<T> {
    fn initiate_application_deactivations(
        applications: &ApplicationBTreeMap<T>,
        application_stake_unstaking_period: Option<T::BlockNumber>,
        role_stake_unstaking_period: Option<T::BlockNumber>,
        cause: ApplicationDeactivationCause,
    ) -> ApplicationsDeactivationsInitiationResult {
        // Update stage on active applications, and collect result

        applications
            .iter()
            .map(
                |(application_id, application)| -> ApplicationDeactivationInitationResult {
                    // Initiate deactivations!
                    Self::try_to_initiate_application_deactivation(
                        application,
                        *application_id,
                        application_stake_unstaking_period,
                        role_stake_unstaking_period,
                        cause,
                    )
                },
            )
            .fold(
                // Initiatial reducer value
                ApplicationsDeactivationsInitiationResult {
                    number_of_unstaking_applications: 0,
                    number_of_deactivated_applications: 0,
                },
                |acc, deactivation_result| {
                    // Update accumulator counters based on what actually happened
                    match deactivation_result {
                        ApplicationDeactivationInitationResult::Ignored => acc,

                        ApplicationDeactivationInitationResult::Unstaking => {
                            ApplicationsDeactivationsInitiationResult {
                                number_of_unstaking_applications: 1 + acc
                                    .number_of_unstaking_applications,
                                number_of_deactivated_applications: acc
                                    .number_of_deactivated_applications,
                            }
                        }

                        ApplicationDeactivationInitationResult::Deactivated => {
                            ApplicationsDeactivationsInitiationResult {
                                number_of_unstaking_applications: acc
                                    .number_of_unstaking_applications,
                                number_of_deactivated_applications: 1 + acc
                                    .number_of_deactivated_applications,
                            }
                        }
                    }
                },
            )
    }

    /// Initiates
    fn try_to_initiate_application_deactivation(
        application: &Application<T::OpeningId, T::BlockNumber, T::StakeId>,
        application_id: T::ApplicationId,
        application_stake_unstaking_period: Option<T::BlockNumber>,
        role_stake_unstaking_period: Option<T::BlockNumber>,
        cause: hiring::ApplicationDeactivationCause,
    ) -> ApplicationDeactivationInitationResult {
        match application.stage {
            ApplicationStage::Active => {
                // Initiate unstaking of any active stake for the application or the role.
                let was_unstaked = Self::opt_infallible_unstake(
                    application.active_role_staking_id,
                    role_stake_unstaking_period,
                ) || Self::opt_infallible_unstake(
                    application.active_application_staking_id,
                    application_stake_unstaking_period,
                );

                // Grab current block height
                let current_block_height = <system::Module<T>>::block_number();

                /*
                 * TODO:
                 * There should be a single transformation based on
                 * was_unstaked which renders a new value for `application.stage`
                 * and `opening.stage`, which guarantees to only produces new values
                 * for given variant values, but the state machine types are currently
                 * not well organised to support this.
                 *
                 * Likewise the construction of hiring::OpeningStage::Active below
                 * is a wreck because of this.
                 *
                 * Issue: https://github.com/Joystream/joystream/issues/36#issuecomment-539567407
                 */

                // Figure out new stage for the applcation
                let new_application_stage = if was_unstaked {
                    ApplicationStage::Unstaking {
                        deactivation_initiated: current_block_height,
                        cause,
                    }
                } else {
                    ApplicationStage::Inactive {
                        deactivation_initiated: current_block_height,
                        deactivated: current_block_height,
                        cause,
                    }
                };

                // Update the application stage
                <ApplicationById<T>>::mutate(application_id, |application| {
                    application.stage = new_application_stage;
                });

                // Update counters on opening
                <OpeningById<T>>::mutate(application.opening_id, |opening| {
                    // NB: This ugly byref destructuring is same issue as pointed out multiple times now.
                    if let hiring::OpeningStage::Active {
                        ref stage,
                        ref applications_added,
                        ref active_application_count,
                        ref unstaking_application_count,
                        ref deactivated_application_count,
                    } = opening.stage
                    {
                        assert!(*active_application_count > 0);

                        let new_active_application_count = active_application_count - 1;

                        let new_unstaking_application_count =
                            unstaking_application_count + if was_unstaked { 1 } else { 0 };

                        let new_deactivated_application_count =
                            deactivated_application_count + if was_unstaked { 0 } else { 1 };

                        opening.stage = hiring::OpeningStage::Active {
                            stage: stage.clone(),
                            applications_added: applications_added.clone(),
                            active_application_count: new_active_application_count,
                            unstaking_application_count: new_unstaking_application_count,
                            deactivated_application_count: new_deactivated_application_count,
                        };
                    } else {
                        panic!("opening stage must be 'Active'");
                    }
                });

                // Call handler(s)
                if was_unstaked {
                    T::ApplicationDeactivatedHandler::deactivated(&application_id, cause);
                }

                // Return conclusion
                if was_unstaked {
                    ApplicationDeactivationInitationResult::Unstaking
                } else {
                    ApplicationDeactivationInitationResult::Deactivated
                }
            }
            _ => ApplicationDeactivationInitationResult::Ignored,
        }
    }

    /// Tries to unstake, based on a stake id which, if set, MUST
    /// be ready to be unstaked, with an optional unstaking period.
    ///
    ///
    /// Returns whether unstaking was actually initiated.
    fn opt_infallible_unstake(
        opt_stake_id: Option<T::StakeId>,
        opt_unstaking_period: Option<T::BlockNumber>,
    ) -> bool {
        if let Some(stake_id) = opt_stake_id {
            // `initiate_unstaking` MUST hold, is runtime invariant, false means code is broken.
            // But should we do panic in runtime? Is there safer way?

            assert!(
                <stake::Module<T>>::initiate_unstaking(&stake_id, opt_unstaking_period).is_ok()
            );
        }

        opt_stake_id.is_some()
    }
}

impl<T: Trait> Module<T> {
    fn infallible_opt_stake_initiation(
        opt_imbalance: Option<NegativeImbalance<T>>,
        application_id: &T::ApplicationId,
    ) -> Option<T::StakeId> {
        if let Some(imbalance) = opt_imbalance {
            Some(Self::infallible_stake_initiation_on_application(
                imbalance,
                application_id,
            ))
        } else {
            None
        }
    }

    fn infallible_stake_initiation_on_application(
        imbalance: NegativeImbalance<T>,
        application_id: &T::ApplicationId,
    ) -> T::StakeId {
        // Create stake
        let new_stake_id = <stake::Module<T>>::create_stake();

        // Keep track of this stake id to process unstaking callbacks that may
        // be invoked later.
        // NB: We purposefully update state to reflect mapping _before_ initiating staking below
        // in order to be safe from race conditions arising out of third party code executing in callback of staking module.

        // MUST never already be a key for new stake, false means code is broken.
        // But should we do panic in runtime? Is there safer way?
        assert!(!<ApplicationIdByStakingId<T>>::exists(new_stake_id));

        <ApplicationIdByStakingId<T>>::insert(new_stake_id, application_id);

        // Initiate staking
        //
        // MUST work, is runtime invariant, false means code is broken.
        // But should we do panic in runtime? Is there safer way?
        assert_eq!(<stake::Module<T>>::stake(&new_stake_id, imbalance), Ok(()));

        new_stake_id
    }
}

/*
 * Used by `add_application` method.
 */

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ApplicationAddedSuccess<T: Trait> {
    Unconditionally,
    CrowdsOutExistingApplication(T::ApplicationId),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ApplicationWouldGetAddedEvaluation<T: Trait> {
    No,
    Yes(ApplicationAddedSuccess<T>),
}

impl<T: Trait> Module<T> {
    /// Evaluates prospects for a new application
    ///
    fn would_application_get_added(
        possible_opening_application_rationing_policy: &Option<ApplicationRationingPolicy>,
        opening_applicants: &BTreeSet<T::ApplicationId>,
        opt_role_stake_balance: &Option<BalanceOf<T>>,
        opt_application_stake_balance: &Option<BalanceOf<T>>,
    ) -> ApplicationWouldGetAddedEvaluation<T> {
        // Check whether any rationing policy is set at all, if not
        // then there is no rationing, and any application can get added.
        let application_rationing_policy = if let Some(application_rationing_policy) =
            possible_opening_application_rationing_policy
        {
            application_rationing_policy
        } else {
            return ApplicationWouldGetAddedEvaluation::Yes(
                ApplicationAddedSuccess::Unconditionally,
            );
        };

        // Map with applications
        let applications_map = Self::application_id_iter_to_map(opening_applicants.iter());

        //
        let active_applications_with_stake_iter =
            applications_map
                .iter()
                .filter_map(|(application_id, application)| {
                    if application.stage == hiring::ApplicationStage::Active {
                        let total_stake =
                            Self::get_opt_stake_amount(application.active_role_staking_id)
                                + Self::get_opt_stake_amount(
                                    application.active_application_staking_id,
                                );

                        Some((application_id, application, total_stake))
                    } else {
                        None
                    }
                });

        // Compute number of active applications
        let number_of_active_applications = active_applications_with_stake_iter.clone().count();

        // Check whether the current number of _active_ applicants is either at or above the maximum
        // limit, if not, then we can add at least one additional application,
        // otherwise we must evaluate whether this new application would specifically get added.
        if (number_of_active_applications as u32)
            < application_rationing_policy.max_active_applicants
        {
            return ApplicationWouldGetAddedEvaluation::Yes(
                ApplicationAddedSuccess::Unconditionally,
            );
        }

        // Here we try to figure out if the new application
        // has sufficient stake to crowd out one of the already
        // active applicants.

        // The total stake of new application
        let total_stake_of_new_application = opt_role_stake_balance.unwrap_or_default()
            + opt_application_stake_balance.unwrap_or_default();

        // The total stake of all current active applications
        let opt_min_item = active_applications_with_stake_iter
            .clone()
            .min_by_key(|(_, _, total_stake)| *total_stake);

        if let Some((application_id, _, lowest_active_total_stake)) = opt_min_item {
            // Finally we compare the two and come up with a final evaluation
            if total_stake_of_new_application <= lowest_active_total_stake {
                ApplicationWouldGetAddedEvaluation::No // stake too low!
            } else {
                ApplicationWouldGetAddedEvaluation::Yes(
                    ApplicationAddedSuccess::CrowdsOutExistingApplication(*application_id),
                )
            }
        } else {
            panic!("`number_of_active_applications` (length of `active_applications_iter`) == 0")
        }
    }

    fn get_opt_stake_amount(stake_id: Option<T::StakeId>) -> BalanceOf<T> {
        stake_id.map_or(<BalanceOf<T> as Zero>::zero(), |stake_id| {
            // INVARIANT: stake MUST exist in the staking module
            assert!(<stake::Stakes<T>>::exists(stake_id));

            let stake = <stake::Stakes<T>>::get(stake_id);

            match stake.staking_status {
                // INVARIANT: stake MUST be in the staked state.
                stake::StakingStatus::Staked(staked_state) => staked_state.staked_amount,
                _ => panic!("stake MUST be in the staked state."),
            }
        })
    }
}
