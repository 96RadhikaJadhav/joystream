use crate::mock::*;
use crate::test::*;

use rstd::collections::btree_set::BTreeSet;
use rstd::result::Result;
/*
Not covered:
- ApplicationDeactivatedHandler
- Application content check

- staking state checks:
i.application.active_role_staking_id;
ii.application.active_application_staking_id;
*/

pub struct FillOpeningFixture {
    pub opening_id: OpeningId,
    pub successful_applications: BTreeSet<<mock::Test as Trait>::ApplicationId>,
    pub opt_successful_applicant_application_stake_unstaking_period: Option<BlockNumber>,
    pub opt_failed_applicant_application_stake_unstaking_period: Option<BlockNumber>,
    pub opt_failed_applicant_role_stake_unstaking_period: Option<BlockNumber>,
}

impl FillOpeningFixture {
    pub(crate) fn default_for_opening(opening_id: OpeningId) -> Self {
        FillOpeningFixture {
            opening_id,
            successful_applications: BTreeSet::new(),
            opt_successful_applicant_application_stake_unstaking_period: None,
            opt_failed_applicant_application_stake_unstaking_period: None,
            opt_failed_applicant_role_stake_unstaking_period: None,
        }
    }

    fn call_and_assert(&self, expected_result: Result<(), FillOpeningError<mock::Test>>) {
        let old_opening = <OpeningById<Test>>::get(self.opening_id);
        let old_applications = self.extract_successful_applications();

        let fill_opening_result = self.fill_opening();

        assert_eq!(fill_opening_result, expected_result);

        self.assert_opening_content(old_opening, fill_opening_result.clone());

        if !fill_opening_result.is_ok() {
            self.assert_same_applications(old_applications);
        }
    }

    fn assert_same_applications(
        &self,
        old_applications: BTreeMap<ApplicationId, Application<OpeningId, BlockNumber, StakeId>>,
    ) {
        for (app_id, application) in old_applications {
            let test_application = <ApplicationById<Test>>::get(app_id);
            assert_eq!(application, test_application)
        }
    }

    fn extract_successful_applications(
        &self,
    ) -> BTreeMap<ApplicationId, Application<OpeningId, BlockNumber, StakeId>> {
        self.successful_applications
            .iter()
            .map(|app_id| (*app_id, <ApplicationById<Test>>::get(app_id)))
            .collect::<BTreeMap<ApplicationId, Application<OpeningId, BlockNumber, StakeId>>>()
    }

    pub(crate) fn fill_opening(&self) -> Result<(), FillOpeningError<mock::Test>> {
        Hiring::fill_opening(
            self.opening_id,
            self.successful_applications.clone(),
            self.opt_successful_applicant_application_stake_unstaking_period,
            self.opt_failed_applicant_application_stake_unstaking_period,
            self.opt_failed_applicant_role_stake_unstaking_period,
        )
    }

    fn assert_opening_content(
        &self,
        old_opening: Opening<Balance, BlockNumber, ApplicationId>,
        fill_opening_result: Result<(), FillOpeningError<mock::Test>>,
    ) {
        let new_opening = <OpeningById<Test>>::get(self.opening_id);
        let mut expected_opening = old_opening.clone();

        if fill_opening_result.is_ok() {
            if let hiring::OpeningStage::Active {
                applications_added, ..
            } = old_opening.stage
            {
                // compose expected stage
                let expected_active_stage = ActiveOpeningStage::Deactivated {
                    cause: OpeningDeactivationCause::Filled,
                    deactivated_at_block: 1,
                    started_accepting_applicants_at_block: 1,
                    started_review_period_at_block: Some(1),
                };

                // calculate application counters
                let mut deactivated_app_count = 0;
                let mut unstaking_app_count = 0;
                for app_id in applications_added.clone() {
                    let application = <ApplicationById<Test>>::get(app_id);

                    match application.stage {
                        ApplicationStage::Active => panic!("Cannot be in active stage"),
                        ApplicationStage::Inactive { .. } => {
                            deactivated_app_count += 1;
                        }
                        ApplicationStage::Unstaking { .. } => {
                            unstaking_app_count += 1;
                        }
                    }
                }

                expected_opening.stage = hiring::OpeningStage::Active {
                    stage: expected_active_stage,
                    applications_added,
                    active_application_count: 0,
                    unstaking_application_count: unstaking_app_count,
                    deactivated_application_count: deactivated_app_count,
                };
            } else {
                panic!("old opening stage MUST be active")
            }
        };
        assert_eq!(expected_opening, new_opening);
    }
}

#[test]
fn fill_opening_fails_due_to_opening_not_existing() {
    build_test_externalities().execute_with(|| {
        let fill_opening_fixture = FillOpeningFixture::default_for_opening(0);
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::OpeningDoesNotExist));
    });
}

#[test]
fn fill_opening_fails_due_to_opening_is_not_active() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.activate_at = ActivateOpeningAt::ExactBlock(5);

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::OpeningNotInReviewPeriodStage));
    });
}

#[test]
fn fill_opening_fails_due_to_redundant_successful_application_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_successful_applicant_application_stake_unstaking_period = Some(50);
        fill_opening_fixture.call_and_assert(Err(
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Success,
            ),
        ));
    });
}

#[test]
fn fill_opening_fails_due_too_short_successful_application_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_successful_applicant_application_stake_unstaking_period = Some(0);
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::UnstakingPeriodTooShort(
            StakePurpose::Application,
            ApplicationOutcomeInFilledOpening::Success,
        )));
    });
}

#[test]
fn fill_opening_fails_due_to_redundant_failed_application_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_failed_applicant_application_stake_unstaking_period = Some(50);
        fill_opening_fixture.call_and_assert(Err(
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Application,
                ApplicationOutcomeInFilledOpening::Failure,
            ),
        ));
    });
}

#[test]
fn fill_opening_fails_due_too_short_failed_application_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_failed_applicant_application_stake_unstaking_period = Some(0);
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::UnstakingPeriodTooShort(
            StakePurpose::Application,
            ApplicationOutcomeInFilledOpening::Failure,
        )));
    });
}

#[test]
fn fill_opening_fails_due_to_redundant_failed_role_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_failed_applicant_role_stake_unstaking_period = Some(50);
        fill_opening_fixture.call_and_assert(Err(
            FillOpeningError::RedundantUnstakingPeriodProvided(
                StakePurpose::Role,
                ApplicationOutcomeInFilledOpening::Failure,
            ),
        ));
    });
}

#[test]
fn fill_opening_fails_due_too_short_failed_role_unstaking_period() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        fill_opening_fixture.opt_failed_applicant_role_stake_unstaking_period = Some(0);
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::UnstakingPeriodTooShort(
            StakePurpose::Role,
            ApplicationOutcomeInFilledOpening::Failure,
        )));
    });
}

#[test]
fn fill_opening_fails_due_not_existing_application() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        let mut apps = BTreeSet::new();
        let invalid_app_id = 10;
        apps.insert(invalid_app_id);
        fill_opening_fixture.successful_applications = apps;
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::ApplicationDoesNotExist(
            invalid_app_id,
        )));
    });
}

#[test]
fn fill_opening_fails_due_not_active_application() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let application_fixture = AddApplicationFixture::default_for_opening(opening_id);
        let app_application_result = application_fixture.add_application();
        let application_id = app_application_result.unwrap().application_id_added;

        let deactivate_application_fixture =
            DeactivateApplicationFixture::default_for_application_id(application_id);

        assert!(deactivate_application_fixture
            .deactivate_application()
            .is_ok());
        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        let mut apps = BTreeSet::new();
        apps.insert(application_id);

        fill_opening_fixture.successful_applications = apps;
        fill_opening_fixture.call_and_assert(Err(FillOpeningError::ApplicationNotInActiveStage(
            application_id,
        )));
    });
}

#[test]
fn fill_opening_succeeds() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let application_fixture = AddApplicationFixture::default_for_opening(opening_id);
        let to_deactivate_app_result = application_fixture.add_application();
        let to_deactivate_app_id = to_deactivate_app_result.unwrap().application_id_added;

        assert!(Hiring::deactive_application(to_deactivate_app_id, None, None).is_ok());

        let app_application_result = application_fixture.add_application();
        let application_id = app_application_result.unwrap().application_id_added;

        assert!(Hiring::begin_review(opening_id).is_ok());

        let mut fill_opening_fixture = FillOpeningFixture::default_for_opening(opening_id);
        let mut apps = BTreeSet::new();
        apps.insert(application_id);

        fill_opening_fixture.successful_applications = apps;
        fill_opening_fixture.call_and_assert(Ok(()));
    });
}
