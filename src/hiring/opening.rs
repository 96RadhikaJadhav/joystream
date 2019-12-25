use rstd::clone::Clone;
use rstd::collections::btree_set::BTreeSet;
use rstd::prelude::*;
use rstd::vec::Vec;

use codec::{Decode, Encode};
use srml_support::ensure;

use crate::hiring::StakePurpose;
use crate::{hiring, ApplicationRationingPolicy, StakingPolicy};

#[derive(Encode, Decode, Default, Debug, Eq, PartialEq, Clone)]
pub struct Opening<Balance, BlockNumber, ApplicationId> {
    /// Block at which opening was added
    pub created: BlockNumber,

    /// Current stage for this opening
    pub stage: OpeningStage<BlockNumber, ApplicationId>,

    /// Maximum length of the review stage.
    pub max_review_period_length: BlockNumber,

    /// Whether, and if so how, to limit the number of active applicants....
    pub application_rationing_policy: Option<ApplicationRationingPolicy>,

    /// Whether any staking is required just to apply, and if so, how that stake is managed.
    pub application_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,

    /// Whether any staking is required for the role, and if so, how that stake is managed.
    pub role_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,

    /// Description of opening
    pub human_readable_text: Vec<u8>,
}

impl<Balance, BlockNumber, ApplicationId> Opening<Balance, BlockNumber, ApplicationId>
where
    Balance: PartialOrd + Clone,
    BlockNumber: Clone + PartialOrd,
    ApplicationId: Ord,
{
    ///Creates new instance of Opening
    pub(crate) fn new(
        current_block_height: BlockNumber,
        activate_at: ActivateOpeningAt<BlockNumber>,
        max_review_period_length: BlockNumber,
        application_rationing_policy: Option<ApplicationRationingPolicy>,
        application_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,
        role_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,
        human_readable_text: Vec<u8>,
    ) -> Self {
        // Construct new opening
        let opening_stage = match activate_at {
            ActivateOpeningAt::CurrentBlock => hiring::OpeningStage::Active {
                // We immediately start accepting applications
                stage: hiring::ActiveOpeningStage::AcceptingApplications {
                    started_accepting_applicants_at_block: current_block_height.clone(),
                },

                // Empty set of applicants
                applications_added: BTreeSet::new(),

                // All counters set to 0
                active_application_count: 0,
                unstaking_application_count: 0,
                deactivated_application_count: 0,
            },

            ActivateOpeningAt::ExactBlock(block_number) => hiring::OpeningStage::WaitingToBegin {
                begins_at_block: block_number,
            },
        };

        hiring::Opening {
            created: current_block_height,
            stage: opening_stage,
            max_review_period_length,
            application_rationing_policy,
            application_staking_policy,
            role_staking_policy,
            human_readable_text,
        }
    }

    pub(crate) fn clone_with_new_active_opening_stage(
        self,
        active_opening_stage: hiring::ActiveOpeningStage<BlockNumber>,
    ) -> Self {
        //TODO: hiring::OpeningStage::Active params should be changed to struct
        //Copy parameters from previous active stage if any or set defaults
        let (
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
        ) = if let hiring::OpeningStage::Active {
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
            ..
        } = self.stage
        {
            //Active opening stage
            (
                applications_added,
                active_application_count,
                unstaking_application_count,
                deactivated_application_count,
            )
        } else {
            //Not active opening stage
            (BTreeSet::new(), 0, 0, 0)
        };

        hiring::Opening {
            stage: hiring::OpeningStage::Active {
                stage: active_opening_stage,
                applications_added,
                active_application_count,
                unstaking_application_count,
                deactivated_application_count,
            },
            ..self
        }
    }

    /// Performs all necessary check before adding an opening
    pub(crate) fn ensure_can_add_opening(
        current_block_height: BlockNumber,
        activate_at: ActivateOpeningAt<BlockNumber>,
        runtime_minimum_balance: Balance,
        application_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,
        role_staking_policy: Option<StakingPolicy<Balance, BlockNumber>>,
    ) -> Result<(), AddOpeningError> {
        // Check that exact activation is actually in the future
        ensure!(
            match activate_at {
                ActivateOpeningAt::ExactBlock(block_number) => block_number > current_block_height,
                _ => true,
            },
            AddOpeningError::OpeningMustActivateInTheFuture
        );

        // Check that staking amounts clear minimum balance required.
        StakingPolicy::ensure_amount_valid_in_opt_staking_policy(
            application_staking_policy,
            runtime_minimum_balance.clone(),
            AddOpeningError::StakeAmountLessThanMinimumCurrencyBalance(StakePurpose::Application),
        )?;

        // Check that staking amounts clear minimum balance required.
        StakingPolicy::ensure_amount_valid_in_opt_staking_policy(
            role_staking_policy,
            runtime_minimum_balance,
            AddOpeningError::StakeAmountLessThanMinimumCurrencyBalance(StakePurpose::Role),
        )?;

        Ok(())
    }
}

/// The stage at which an `Opening` may be at.
#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub enum OpeningStage<BlockNumber, ApplicationId> {
    // ..
    WaitingToBegin {
        begins_at_block: BlockNumber,
    },

    // TODO: Fix this bad name
    //
    Active {
        /// Active stage
        stage: hiring::ActiveOpeningStage<BlockNumber>,

        /// Set of identifiers for all applications which have been added, but not removed, for this opening.
        /// Is required for timely on-chain lookup of all applications associated with an opening.
        applications_added: BTreeSet<ApplicationId>, //BTreeMap<ApplicationId, ()>, //Vec<T::ApplicationId>,

        // TODO: Drop these counters
        // https://github.com/Joystream/substrate-hiring-module/issues/9
        //
        // Counters over all possible application states.
        // Are needed to set `application_index_in_opening` in new applications
        // Are very useful for light clients.
        //
        // NB: Remember that _all_ state transitions in applications will require updating these variables,
        // its easy to forget!
        //
        // NB: The sum of
        // - `active_application_count`
        // - `unstaking_application_count`
        // - `deactivated_application_count`
        //
        // equals the total number of applications ever added to the openig via `add_application`.
        /// Active NOW
        active_application_count: u32,

        /// Unstaking NOW
        unstaking_application_count: u32,

        /// Deactivated at any time for any cause.
        deactivated_application_count: u32,

        // Removed at any time.
        //removed_application_count: u32
    },
}

impl<BlockNumber: Clone, ApplicationId: Ord + Clone> OpeningStage<BlockNumber, ApplicationId> {
    /// The number of applications ever added to the opening via
    /// `add_opening` extrinsic.
    pub fn number_of_applications_ever_added(&self) -> u32 {
        match self {
            OpeningStage::WaitingToBegin { .. } => 0,

            OpeningStage::Active {
                active_application_count,
                unstaking_application_count,
                deactivated_application_count,
                ..
            } => {
                active_application_count
                    + unstaking_application_count
                    + deactivated_application_count
            }
        }
    }

    /// Ensures that an opening is waiting to begin.
    pub(crate) fn ensure_opening_stage_is_waiting_to_begin<Err>(
        &self,
        error: Err,
    ) -> Result<BlockNumber, Err> {
        if let OpeningStage::WaitingToBegin { begins_at_block } = self {
            return Ok(begins_at_block.clone());
        }

        Err(error)
    }

    /// Clones current stage. Panics if not Active.
    /// Adds application_id to applications_added collection.
    /// Increments 'active_application_count' counter.
    pub(crate) fn clone_with_added_active_application(
        self,
        new_application_id: ApplicationId,
    ) -> Self {
        if let hiring::OpeningStage::Active {
            stage,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
            applications_added,
        } = self.clone()
        {
            let mut apps_added = applications_added.clone();
            apps_added.insert(new_application_id);

            hiring::OpeningStage::Active {
                stage,
                applications_added: apps_added,
                active_application_count: active_application_count + 1,
                unstaking_application_count,
                deactivated_application_count,
            }
        } else {
            panic!("updated opening should be in active stage");
        }
    }
}

/// OpeningStage must be default constructible because it indirectly is a value in a storage map.
/// ***SHOULD NEVER ACTUALLY GET CALLED, IS REQUIRED TO DUE BAD STORAGE MODEL IN SUBSTRATE***
impl<BlockNumber: Default, ApplicationId> Default for OpeningStage<BlockNumber, ApplicationId> {
    fn default() -> Self {
        OpeningStage::WaitingToBegin {
            begins_at_block: BlockNumber::default(),
        }
    }
}

#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub enum ActiveOpeningStage<BlockNumber> {
    AcceptingApplications {
        //
        started_accepting_applicants_at_block: BlockNumber,
    },

    //
    ReviewPeriod {
        started_accepting_applicants_at_block: BlockNumber,

        started_review_period_at_block: BlockNumber,
    },

    //
    Deactivated {
        cause: OpeningDeactivationCause,

        deactivated_at_block: BlockNumber,

        started_accepting_applicants_at_block: BlockNumber,

        /// Whether the review period had ever been started, and if so, at what block.
        /// Deactivation can also occur directly from the AcceptingApplications stage.
        started_review_period_at_block: Option<BlockNumber>,
    },
}

impl<BlockNumber: Clone> ActiveOpeningStage<BlockNumber> {
    /// Ensures that active opening stage is accepting applications.
    pub(crate) fn ensure_active_opening_is_accepting_applications<Err>(
        &self,
        error: Err,
    ) -> Result<BlockNumber, Err> {
        if let ActiveOpeningStage::AcceptingApplications {
            started_accepting_applicants_at_block,
        } = self
        {
            return Ok(started_accepting_applicants_at_block.clone());
        }

        Err(error)
    }

    /// Ensures that active opening stage is in review period.
    pub(crate) fn ensure_active_opening_is_in_review_period<Err>(
        &self,
        error: Err,
    ) -> Result<(BlockNumber, BlockNumber), Err> {
        match self {
            ActiveOpeningStage::ReviewPeriod {
                started_accepting_applicants_at_block,
                started_review_period_at_block,
            } => Ok((
                started_accepting_applicants_at_block.clone(),
                started_review_period_at_block.clone(),
            )), // <= need proper type here in the future, not param
            _ => Err(error),
        }
    }
}

#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub enum OpeningDeactivationCause {
    CancelledBeforeActivation,
    CancelledAcceptingApplications,
    CancelledInReviewPeriod,
    ReviewPeriodExpired,
    Filled,
}

/// NB:
/// `OpeningCancelled` does not have the ideal form.
/// https://github.com/Joystream/substrate-hiring-module/issues/10
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct OpeningCancelled {
    pub number_of_unstaking_applications: u32,
    pub number_of_deactivated_applications: u32,
}

// Safe and explict way of chosing
#[derive(Encode, Decode, Eq, PartialEq, Clone, Debug)]
pub enum ActivateOpeningAt<BlockNumber> {
    CurrentBlock,
    ExactBlock(BlockNumber),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum AddOpeningError {
    OpeningMustActivateInTheFuture,

    /// It is not possible to stake less than the minimum balance defined in the
    /// `Currency` module.
    StakeAmountLessThanMinimumCurrencyBalance(StakePurpose),
}

/// The possible outcome for an application in an opening which is being filled.
#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ApplicationOutcomeInFilledOpening {
    Success,
    Failure,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum CancelOpeningError {
    UnstakingPeriodTooShort(StakePurpose),
    RedundantUnstakingPeriodProvided(StakePurpose),
    OpeningDoesNotExist,
    OpeningNotInCancellableStage,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum RemoveOpeningError {
    OpeningDoesNotExist,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BeginReviewError {
    OpeningDoesNotExist,
    OpeningNotInAcceptingApplicationsStage,
}
