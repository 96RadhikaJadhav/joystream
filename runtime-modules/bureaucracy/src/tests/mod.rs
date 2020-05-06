mod mock;

use crate::constraints::InputValidationLengthConstraint;
use crate::types::{Lead, OpeningPolicyCommitment};
use crate::{Instance1, RawEvent};
use mock::{build_test_externalities, Bureaucracy1, System, TestEvent};
use srml_support::StorageValue;
use system::{EventRecord, Phase, RawOrigin};

struct SetLeadFixture;
impl SetLeadFixture {
    fn set_lead(lead_account_id: u64) {
        assert_eq!(
            Bureaucracy1::set_lead(RawOrigin::Root.into(), 1, lead_account_id),
            Ok(())
        );
    }
}

struct AddCuratorOpeningFixture {
    origin: RawOrigin<u64>,
    activate_at: hiring::ActivateOpeningAt<u64>,
    commitment: OpeningPolicyCommitment<u64, u64>,
    human_readable_text: Vec<u8>,
}

impl Default for AddCuratorOpeningFixture {
    fn default() -> Self {
        AddCuratorOpeningFixture {
            origin: RawOrigin::Signed(1),
            activate_at: hiring::ActivateOpeningAt::CurrentBlock,
            commitment: <OpeningPolicyCommitment<u64, u64>>::default(),
            human_readable_text: Vec::new(),
        }
    }
}

impl AddCuratorOpeningFixture {
    pub fn call_and_assert(&self, expected_result: Result<(), &str>) {
        let actual_result = Bureaucracy1::add_curator_opening(
            self.origin.clone().into(),
            self.activate_at.clone(),
            self.commitment.clone(),
            self.human_readable_text.clone(),
        );
        assert_eq!(actual_result, expected_result);
    }

    fn with_text(self, text: Vec<u8>) -> Self {
        AddCuratorOpeningFixture {
            human_readable_text: text,
            ..self
        }
    }

    fn with_activate_at(self, activate_at: hiring::ActivateOpeningAt<u64>) -> Self {
        AddCuratorOpeningFixture {
            activate_at,
            ..self
        }
    }
}

struct EventFixture;
impl EventFixture {
    fn assert_events(expected_raw_events: Vec<RawEvent<u64, u64, u64, u64, crate::Instance1>>) {
        let expected_events = expected_raw_events
            .iter()
            .map(|ev| EventRecord {
                phase: Phase::ApplyExtrinsic(0),
                event: TestEvent::bureaucracy_Instance1(ev.clone()),
                topics: vec![],
            })
            .collect::<Vec<EventRecord<_, _>>>();

        assert_eq!(System::events(), expected_events);
    }
}

#[test]
fn set_forum_sudo_set() {
    build_test_externalities().execute_with(|| {
        // Ensure that lead is default
        assert_eq!(Bureaucracy1::current_lead(), None);

        let lead_account_id = 1;
        let lead_member_id = 1;

        // Set lead
        assert_eq!(
            Bureaucracy1::set_lead(RawOrigin::Root.into(), lead_member_id, lead_account_id),
            Ok(())
        );

        let lead = Lead {
            member_id: lead_member_id,
            role_account_id: lead_account_id,
        };
        assert_eq!(Bureaucracy1::current_lead(), Some(lead));

        EventFixture::assert_events(vec![RawEvent::LeaderSet(lead_member_id, lead_account_id)]);
    });
}

#[test]
fn add_curator_opening_succeeds() {
    build_test_externalities().execute_with(|| {
        SetLeadFixture::set_lead(1);

        let add_curator_opening_fixture = AddCuratorOpeningFixture::default();

        add_curator_opening_fixture.call_and_assert(Ok(()));
    });
}

#[test]
fn add_curator_opening_fails_with_lead_is_not_set() {
    build_test_externalities().execute_with(|| {
        let add_curator_opening_fixture = AddCuratorOpeningFixture::default();

        add_curator_opening_fixture.call_and_assert(Err("Current lead is not set"));
    });
}

#[test]
fn add_curator_opening_fails_with_invalid_human_readable_text() {
    build_test_externalities().execute_with(|| {
        SetLeadFixture::set_lead(1);

        <crate::OpeningHumanReadableText<Instance1>>::put(InputValidationLengthConstraint {
            min: 1,
            max_min_diff: 5,
        });

        let add_curator_opening_fixture = AddCuratorOpeningFixture::default().with_text(Vec::new());

        add_curator_opening_fixture.call_and_assert(Err(crate::MSG_OPENING_TEXT_TOO_SHORT));

        let add_curator_opening_fixture =
            AddCuratorOpeningFixture::default().with_text(b"Long text".to_vec());

        add_curator_opening_fixture.call_and_assert(Err(crate::MSG_OPENING_TEXT_TOO_LONG));
    });
}

#[test]
fn add_curator_opening_fails_with_hiring_error() {
    build_test_externalities().execute_with(|| {
        SetLeadFixture::set_lead(1);

        let add_curator_opening_fixture = AddCuratorOpeningFixture::default()
            .with_activate_at(hiring::ActivateOpeningAt::ExactBlock(0));

        add_curator_opening_fixture.call_and_assert(Err("Opening does not activate in the future"));
    });
}
