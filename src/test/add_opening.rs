use super::*;
use crate::mock::*;
use crate::StakingAmountLimitMode::Exact;
use rstd::collections::btree_set::BTreeSet;

static FIRST_BLOCK_HEIGHT: <Test as system::Trait>::BlockNumber = 1;
pub static OPENING_HUMAN_READABLE_TEXT: &[u8] = b"OPENING_HUMAN_READABLE_TEXT!!!!";

/*
Not covered:
- ApplicationRationingPolicy (no ensures yet in add_opening)
*/

pub struct AddOpeningFixture<Balance> {
    pub activate_at: ActivateOpeningAt<u64>,
    pub max_review_period_length: u64,
    pub application_rationing_policy: Option<ApplicationRationingPolicy>,
    pub application_staking_policy: Option<StakingPolicy<Balance, u64>>,
    pub role_staking_policy: Option<StakingPolicy<Balance, u64>>,
    pub human_readable_text: Vec<u8>,
}

impl<Balance> Default for AddOpeningFixture<Balance> {
    fn default() -> Self {
        AddOpeningFixture {
            activate_at: ActivateOpeningAt::CurrentBlock,
            max_review_period_length: 672,
            application_rationing_policy: None,
            application_staking_policy: None,
            role_staking_policy: None,
            human_readable_text: OPENING_HUMAN_READABLE_TEXT.to_vec(),
        }
    }
}

impl AddOpeningFixture<u64> {
    fn call_and_assert(&self, expected_result: Result<u64, AddOpeningError>) {
        let expected_opening_id = Hiring::next_opening_id();

        let add_opening_result = self.add_opening();
        assert_eq!(add_opening_result, expected_result);

        if add_opening_result.is_ok() {
            // Check next opening id has been updated
            assert_eq!(Hiring::next_opening_id(), expected_opening_id + 1);
            // Check opening exists
            assert!(<OpeningById<Test>>::exists(expected_opening_id));
        } else {
            // Check next opening id has not been updated
            assert_eq!(Hiring::next_opening_id(), expected_opening_id);
            // Check opening does not exist
            assert!(!<OpeningById<Test>>::exists(expected_opening_id));
        };

        //Check opening content
        if add_opening_result.is_ok() {
            self.assert_opening_content(expected_opening_id);
        }
    }

    fn assert_opening_content(&self, expected_opening_id: u64) {
        let expected_opening_stage = match self.activate_at {
            ActivateOpeningAt::CurrentBlock => OpeningStage::Active {
                stage: ActiveOpeningStage::AcceptingApplications {
                    started_accepting_applicants_at_block: FIRST_BLOCK_HEIGHT,
                },
                applications_added: BTreeSet::new(),
                active_application_count: 0,
                unstaking_application_count: 0,
                deactivated_application_count: 0,
            },
            ActivateOpeningAt::ExactBlock(block_number) => OpeningStage::WaitingToBegin {
                begins_at_block: block_number,
            },
        };

        let expected_opening = Opening {
            created: FIRST_BLOCK_HEIGHT,
            stage: expected_opening_stage,
            max_review_period_length: self.max_review_period_length,
            application_rationing_policy: self.application_rationing_policy.clone(),
            application_staking_policy: self.application_staking_policy.clone(),
            role_staking_policy: self.role_staking_policy.clone(),
            human_readable_text: OPENING_HUMAN_READABLE_TEXT.to_vec(),
        };

        let found_opening = Hiring::opening_by_id(expected_opening_id);
        assert_eq!(found_opening, expected_opening);
    }

    pub(crate) fn add_opening(&self) -> Result<u64, AddOpeningError> {
        Hiring::add_opening(
            self.activate_at.clone(),
            self.max_review_period_length,
            self.application_rationing_policy.clone(),
            self.application_staking_policy.clone(),
            self.role_staking_policy.clone(),
            self.human_readable_text.clone(),
        )
    }
}

#[test]
fn add_opening_succeeds_with_exact_block() {
    build_test_externalities().execute_with(|| {
        let opening_data = AddOpeningFixture::default();

        let expected_opening_id = 0;

        // Add an opening, check that the returned value is Zero
        opening_data.call_and_assert(Ok(expected_opening_id));
    });
}

#[test]
fn add_opening_succeeds_with_waiting_to_begin() {
    build_test_externalities().execute_with(|| {
        let mut opening_data = AddOpeningFixture::default();
        opening_data.activate_at = ActivateOpeningAt::ExactBlock(22);
        let expected_opening_id = 0;

        // Add an opening, check that the returned value is Zero
        opening_data.call_and_assert(Ok(expected_opening_id));
    });
}

#[test]
fn add_opening_fails_due_to_activation_in_the_past() {
    build_test_externalities().execute_with(|| {
        let mut opening_data = AddOpeningFixture::default();
        opening_data.activate_at = ActivateOpeningAt::ExactBlock(0);

        opening_data.call_and_assert(Err(AddOpeningError::OpeningMustActivateInTheFuture));
    });
}

#[test]
fn add_opening_succeeds_or_fails_due_to_application_staking_policy() {
    build_test_externalities().execute_with(|| {
        let mut opening_data = AddOpeningFixture::default();
        //Valid stake amount
        opening_data.application_staking_policy = Some(StakingPolicy {
            amount: 300,
            amount_mode: Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        opening_data.call_and_assert(Ok(0));

        //Invalid stake amount
        opening_data.application_staking_policy = Some(StakingPolicy {
            amount: 1,
            amount_mode: Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        opening_data.call_and_assert(Err(
            AddOpeningError::StakeAmountLessThanMinimumCurrencyBalance(StakePurpose::Application),
        ));
    });
}

#[test]
fn add_opening_succeeds_or_fails_due_to_role_staking_policy() {
    build_test_externalities().execute_with(|| {
        let mut opening_data = AddOpeningFixture::default();
        //Valid stake amount
        opening_data.role_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        opening_data.call_and_assert(Ok(0));

        //Invalid stake amount
        opening_data.role_staking_policy = Some(StakingPolicy {
            amount: 1,
            amount_mode: Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        opening_data.call_and_assert(Err(
            AddOpeningError::StakeAmountLessThanMinimumCurrencyBalance(StakePurpose::Role),
        ));
    });
}

#[test]
fn add_opening_succeeds_or_fails_due_to_invalid_application_rationing_policy() {
    build_test_externalities().execute_with(|| {
        let mut opening_data = AddOpeningFixture::default();
        opening_data.application_rationing_policy = Some(ApplicationRationingPolicy {
            max_active_applicants: 0,
        });

        opening_data.call_and_assert(Err(AddOpeningError::ApplicationRationingZeroMaxApplicants));
    });
}
