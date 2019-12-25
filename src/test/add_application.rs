use super::*;
use crate::mock::*;
use stake::NegativeImbalance;

use add_opening::AddOpeningFixture;

/*
Most 'ensures' (add_application() fail reasons) covered in ensure_can_add_application_* tests.

Not covered:
- staking state checks:
i.application.active_role_staking_id;
ii.application.active_application_staking_id;
- stake module calls:
i.infallible_opt_stake_initiation -> infallible_stake_initiation_on_application -> stake::create_stake()

*/

pub struct AddApplicationFixture {
    pub opening_id: u64,
    pub opt_role_stake_imbalance: Option<NegativeImbalance<Test>>,
    pub opt_application_stake_imbalance: Option<NegativeImbalance<Test>>,
    pub human_readable_text: Vec<u8>,
}

impl AddApplicationFixture {
    pub(crate) fn default_for_opening(opening_id: u64) -> Self {
        AddApplicationFixture {
            opening_id,
            opt_role_stake_imbalance: None,
            opt_application_stake_imbalance: None,
            human_readable_text: add_opening::OPENING_HUMAN_READABLE_TEXT.to_vec(),
        }
    }

    pub(crate) fn add_application(&self) -> Result<ApplicationAdded<u64>, AddApplicationError> {
        let mut opt_role_stake_imbalance = None;
        if let Some(ref imbalance) = self.opt_role_stake_imbalance {
            opt_role_stake_imbalance =
                Some(stake::NegativeImbalance::<Test>::new(imbalance.peek()));
        }

        let mut opt_application_stake_imbalance = None;
        if let Some(ref imbalance) = self.opt_application_stake_imbalance {
            opt_application_stake_imbalance =
                Some(stake::NegativeImbalance::<Test>::new(imbalance.peek()));
        }

        Hiring::add_application(
            self.opening_id,
            opt_role_stake_imbalance,
            opt_application_stake_imbalance,
            self.human_readable_text.clone(),
        )
    }

    fn call_and_assert(&self, expected_result: Result<ApplicationAdded<u64>, AddApplicationError>) {
        let expected_application_id = Hiring::next_application_id();
        // save opening state (can be invalid if invalid opening_id provided)
        let old_opening_state = <OpeningById<Test>>::get(self.opening_id);

        let add_application_result = self.add_application();

        // Check expected result
        assert_eq!(add_application_result, expected_result);

        if add_application_result.is_ok() {
            // Check next application id has been updated
            assert_eq!(Hiring::next_application_id(), expected_application_id + 1);
            // Check application exists
            assert!(<ApplicationById<Test>>::exists(expected_application_id));
        } else {
            // Check next application id has not been updated
            assert_eq!(Hiring::next_application_id(), expected_application_id);
            // Check application does not exist
            assert!(!<ApplicationById<Test>>::exists(expected_application_id));
        };

        //Check application content
        self.assert_application_content(add_application_result.clone(), expected_application_id);

        //Check opening state after add_application() call
        self.assert_opening_content(
            old_opening_state,
            add_application_result,
            expected_application_id,
        );
    }

    fn assert_application_content(
        &self,
        add_application_result: Result<ApplicationAdded<u64>, AddApplicationError>,
        expected_application_id: u64,
    ) {
        if add_application_result.is_ok() {
            let opening = <OpeningById<Test>>::get(self.opening_id);
            let total_applications_count;
            if let OpeningStage::Active {
                applications_added, ..
            } = opening.stage
            {
                total_applications_count = applications_added.len();
            } else {
                panic!("Opening should be in active stage");
            }

            let found_application = <ApplicationById<Test>>::get(expected_application_id);
            let expected_application_index_in_opening = total_applications_count as u32 - 1;

            // Skip this check due external stake module dependency
            let expected_active_role_staking_id = found_application.active_role_staking_id;

            // Skip this check due external stake module dependency
            let expected_active_application_staking_id =
                found_application.active_application_staking_id;

            let expected_application = Application {
                opening_id: self.opening_id,
                application_index_in_opening: expected_application_index_in_opening,
                add_to_opening_in_block: 1,
                active_role_staking_id: expected_active_role_staking_id,
                active_application_staking_id: expected_active_application_staking_id,
                stage: ApplicationStage::Active,
                human_readable_text: add_opening::OPENING_HUMAN_READABLE_TEXT.to_vec(),
            };

            assert_eq!(found_application, expected_application);
        }
    }

    fn assert_opening_content(
        &self,
        old_opening: Opening<u64, u64, u64>,
        add_application_result: Result<ApplicationAdded<u64>, AddApplicationError>,
        expected_application_id: u64,
    ) {
        let new_opening_state = <OpeningById<Test>>::get(self.opening_id);

        let mut expected_added_apps_in_opening;
        let mut expected_active_application_count;
        let mut expected_unstaking_application_count;
        let expected_deactivated_application_count;
        if let OpeningStage::Active {
            applications_added,
            active_application_count,
            unstaking_application_count,
            deactivated_application_count,
            ..
        } = old_opening.stage
        {
            expected_added_apps_in_opening = applications_added.clone();
            expected_active_application_count = active_application_count;
            expected_deactivated_application_count = deactivated_application_count;
            expected_unstaking_application_count = unstaking_application_count;

            if let Ok(add_app_data) = add_application_result {
                expected_added_apps_in_opening.insert(expected_application_id);
                if add_app_data.application_id_crowded_out.is_some() {
                    expected_unstaking_application_count += 1;
                } else {
                    expected_active_application_count += 1;
                }
            }
        } else {
            panic!("Opening should be in active stage");
        }

        let expected_opening = Opening {
            stage: OpeningStage::Active {
                stage: ActiveOpeningStage::AcceptingApplications {
                    started_accepting_applicants_at_block: 1,
                },
                applications_added: expected_added_apps_in_opening,
                active_application_count: expected_active_application_count,
                unstaking_application_count: expected_unstaking_application_count,
                deactivated_application_count: expected_deactivated_application_count,
            },
            ..old_opening
        };
        assert_eq!(new_opening_state, expected_opening);
    }
}

#[test]
fn add_application_succeeds() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let application_fixture = AddApplicationFixture::default_for_opening(opening_id);
        application_fixture.call_and_assert(Ok(ApplicationAdded {
            application_id_added: 0,
            application_id_crowded_out: None,
        }));
    });
}

#[test]
fn add_application_succeeds_with_crowding_out() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.application_rationing_policy = Some(hiring::ApplicationRationingPolicy {
            max_active_applicants: 1,
        });
        opening_fixture.application_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::AtLeast,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let mut application_fixture = AddApplicationFixture::default_for_opening(opening_id);
        application_fixture.opt_application_stake_imbalance =
            Some(stake::NegativeImbalance::<Test>::new(100));

        assert!(application_fixture.add_application().is_ok());

        application_fixture.opt_application_stake_imbalance =
            Some(stake::NegativeImbalance::<Test>::new(101));

        application_fixture.call_and_assert(Ok(ApplicationAdded {
            application_id_added: 1,
            application_id_crowded_out: Some(0),
        }));
    });
}

#[test]
fn add_application_fails() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.application_rationing_policy = Some(hiring::ApplicationRationingPolicy {
            max_active_applicants: 1,
        });
        opening_fixture.application_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::AtLeast,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let mut application_fixture = AddApplicationFixture::default_for_opening(opening_id);
        application_fixture.opt_application_stake_imbalance =
            Some(stake::NegativeImbalance::<Test>::new(50));

        application_fixture.call_and_assert(Err(AddApplicationError::StakeAmountTooLow(
            StakePurpose::Application,
        )));
    });
}
