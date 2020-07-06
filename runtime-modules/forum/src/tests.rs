#![cfg(test)]

use super::*;
use crate::mock::*;
/// test cases are arranged as two layers.
/// first layer is each method in defined in module.
/// second layer is each parameter of the specific method.

/*
 * update_category_membership_of_moderator_origin
 */
#[test]
// test case for check if origin is forum lead
fn update_category_membership_of_moderator_origin() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin,
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            NOT_FORUM_LEAD_ORIGIN,
            moderator_id,
            category_id,
            true,
            Err(ERROR_ORIGIN_NOT_FORUM_LEAD),
        );
    });
}

#[test]
// test case for check whether category is existed.
fn update_category_membership_of_moderator_category() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            INVLAID_CATEGORY_ID,
            true,
            Err(ERROR_CATEGORY_DOES_NOT_EXIST),
        );
    });
}

#[test]
// test case for check if account id registered as moderator
fn update_category_membership_of_moderator_account_id() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(default_genesis_config()).execute_with(|| {
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            NOT_REGISTER_MODERATOR_ID,
            category_id,
            true,
            Err(ERROR_MODERATOR_ID_NOT_MATCH_ACCOUNT),
        );
    });

    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin,
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
    });
}

#[test]
// test case for check if origin is forum lead
fn create_category_origin() {
    let origins = vec![FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_ORIGIN];
    let results = vec![Ok(()), Err(ERROR_ORIGIN_NOT_FORUM_LEAD)];
    for index in 0..origins.len() {
        let config = default_genesis_config();
        build_test_externalities(config).execute_with(|| {
            create_category_mock(
                origins[index].clone(),
                None,
                good_category_title(),
                good_category_description(),
                results[index],
            );
        });
    }
}

#[test]
// test case for check if parent category is archived or not existing.
fn create_category_parent() {
    let parents = vec![Some(1), Some(2), Some(3)];
    let results = vec![
        Ok(()),
        Err(ERROR_ANCESTOR_CATEGORY_IMMUTABLE),
        Err(ERROR_CATEGORY_DOES_NOT_EXIST),
    ];

    for index in 0..parents.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            create_category_mock(
                origin.clone(),
                Some(1),
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            update_category_archival_status_mock(origin.clone(), 2, true, Ok(()));

            create_category_mock(
                origin.clone(),
                parents[index],
                good_category_title(),
                good_category_description(),
                results[index],
            );
        });
    }
}

#[test]
// test case set category depth
fn create_category_depth() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let max_depth = <Runtime as Trait>::get_max_category_depth();
        println!("{:?}", max_depth);
        for i in 0..(max_depth + 1) {
            let parent_category_id = match i {
                0 => None,
                _ => Some(i),
            };
            let expected_result = match i {
                _ if i >= max_depth => Err(ERROR_MAX_VALID_CATEGORY_DEPTH_EXCEEDED),
                _ => Ok(()),
            };

            if let Some(tmp_parent_category_id) = parent_category_id {
                let tree_path = TestForumModule::build_category_tree_path(&tmp_parent_category_id);
            }

            let tmp = create_category_mock(
                origin.clone(),
                parent_category_id,
                good_category_title(),
                good_category_description(),
                expected_result,
            );
        }
    });
}

/*
 ** update_category
 */
#[test]
// test if category updator is forum lead
fn update_category_archival_status_origin() {
    let origins = [FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_ORIGIN];
    let results = vec![Ok(()), Err(ERROR_ORIGIN_NOT_FORUM_LEAD)];

    for index in 0..origins.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            create_category_mock(
                origin,
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            update_category_archival_status_mock(origins[index].clone(), 1, true, results[index]);
        });
    }
}

#[test]
// test case for new setting actually not update category status
fn update_category_archival_status_no_change() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_archival_status_mock(
            origin,
            1,
            false,
            Err(ERROR_CATEGORY_NOT_BEING_UPDATED),
        );
    });
}

#[test]
// test case for editing nonexistent category
fn update_category_archival_status_category_exists() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_archival_status_mock(origin.clone(), 1, true, Ok(()));
        update_category_archival_status_mock(
            origin.clone(),
            2,
            true,
            Err(ERROR_CATEGORY_DOES_NOT_EXIST),
        );
    });
}

/*
 ** create_thread
 */
#[test]
// test if thread creator is valid forum user
fn create_thread_origin() {
    let origins = [NOT_FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_2_ORIGIN];
    let forum_user_id = NOT_FORUM_LEAD_ORIGIN_ID;
    let results = vec![Ok(()), Err(ERROR_FORUM_USER_ID_NOT_MATCH_ACCOUNT)];
    for index in 0..origins.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            let category_id = create_category_mock(
                origin,
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            create_thread_mock(
                origins[index].clone(),
                forum_user_id,
                category_id,
                good_thread_title(),
                good_thread_text(),
                None,
                results[index],
            );
        });
    }
}

#[test]
// test if timestamp of poll start time and end time are valid
fn create_thread_poll_timestamp() {
    let expiration_diff = 10;
    let results = vec![Ok(()), Err(ERROR_POLL_TIME_SETTING)];

    for index in 0..results.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);

        build_test_externalities(config).execute_with(|| {
            change_current_time(1);
            let poll = generate_poll_timestamp_cases(index, expiration_diff);
            change_current_time(index as u64 * expiration_diff + 1);

            let category_id = create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );

            create_thread_mock(
                origin.clone(),
                forum_lead,
                category_id,
                good_thread_title(),
                good_thread_text(),
                Some(poll),
                results[index],
            );
        });
    }
}

#[test]
// test if author can edit thread's title
fn edit_thread_title() {
    let forum_users = [NOT_FORUM_LEAD_ORIGIN_ID, NOT_FORUM_LEAD_2_ORIGIN_ID];
    let origins = [NOT_FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_2_ORIGIN];

    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        // create thread by author
        let thread_id = create_thread_mock(
            origins[0].clone(),
            forum_users[0],
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );

        // check author can edit text
        edit_thread_title_mock(
            origins[0].clone(),
            forum_users[0],
            thread_id,
            good_thread_new_title(),
            Ok(()),
        );

        // check non-author is forbidden from editing text
        edit_thread_title_mock(
            origins[1].clone(),
            forum_users[1],
            thread_id,
            good_thread_new_title(),
            Err(ERROR_ACCOUNT_DOES_NOT_MATCH_THREAD_AUTHOR),
        );
    });
}

/*
 ** vote_on_poll
 */
#[test]
// test if poll submitter is a forum user
fn vote_on_poll_origin() {
    let origins = vec![FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_ORIGIN];
    let results = vec![Ok(()), Err(ERROR_FORUM_USER_ID_NOT_MATCH_ACCOUNT)];
    let expiration_diff = 10;

    for index in 0..origins.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            let category_id = create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            let thread_id = create_thread_mock(
                origin.clone(),
                forum_lead,
                category_id,
                good_thread_title(),
                good_thread_text(),
                Some(generate_poll(expiration_diff)),
                Ok(()),
            );

            vote_on_poll_mock(
                origins[index].clone(),
                forum_lead,
                thread_id,
                1,
                results[index],
            );
        });
    }
}

#[test]
// test if poll metadata created
fn vote_on_poll_exists() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        vote_on_poll_mock(
            origin.clone(),
            forum_lead,
            thread_id,
            1,
            Err(ERROR_POLL_NOT_EXIST),
        );
    });
}

#[test]
// test if forum reject poll submit after expiration
fn vote_on_poll_expired() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    let expiration_diff = 10;

    build_test_externalities(config).execute_with(|| {
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            Some(generate_poll(expiration_diff)),
            Ok(()),
        );
        change_current_time(expiration_diff + 1);
        vote_on_poll_mock(
            origin.clone(),
            forum_lead,
            thread_id,
            1,
            Err(ERROR_POLL_COMMIT_EXPIRED),
        );
    });
}

/*
 ** moderate_thread
 */

#[test]
// test if thread moderator registered as valid moderator
fn moderate_thread_origin_ok() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        moderate_thread_mock(origin, moderator_id, thread_id, Ok(()));
    });
}

/*
 ** add_post
 */

#[test]
// test if post origin registered as forum user
fn add_post_origin() {
    let origins = vec![FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_ORIGIN];
    let results = vec![Ok(()), Err(ERROR_FORUM_USER_ID_NOT_MATCH_ACCOUNT)];
    for index in 0..origins.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            let category_id = create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );

            let thread_id = create_thread_mock(
                origin.clone(),
                forum_lead,
                category_id,
                good_thread_title(),
                good_thread_text(),
                None,
                Ok(()),
            );
            create_post_mock(
                origins[index].clone(),
                forum_lead,
                thread_id,
                good_post_text(),
                results[index],
            );
        });
    }
}

#[test]
// test if post text can be edited by author
fn edit_post_text() {
    let config = default_genesis_config();
    let forum_users = [NOT_FORUM_LEAD_ORIGIN_ID, NOT_FORUM_LEAD_2_ORIGIN_ID];
    let origins = [NOT_FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_2_ORIGIN];

    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);

    build_test_externalities(config).execute_with(|| {
        // prepare category and thread
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );

        // create post by author
        let post_id = create_post_mock(
            origins[0].clone(),
            forum_users[0],
            thread_id,
            good_post_text(),
            Ok(()),
        );

        // check author can edit text
        edit_post_text_mock(
            origins[0].clone(),
            forum_users[0],
            post_id,
            good_post_new_text(),
            Ok(()),
        );

        // check non-author is forbidden from editing text
        edit_post_text_mock(
            origins[1].clone(),
            forum_users[1],
            post_id,
            good_post_new_text(),
            Err(ERROR_ACCOUNT_DOES_NOT_MATCH_POST_AUTHOR),
        );
    });
}

/*
 ** react_post
 */
#[test]
// test if post react take effect
fn react_post() {
    // three reations to post, test them one by one.
    let new_values = vec![
        PostReaction::ThumbUp,
        PostReaction::ThumbDown,
        PostReaction::Like,
    ];
    for index in 0..new_values.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = FORUM_LEAD_ORIGIN;

        build_test_externalities(config).execute_with(|| {
            let category_id = create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );

            let thread_id = create_thread_mock(
                origin.clone(),
                forum_lead,
                category_id,
                good_thread_title(),
                good_thread_text(),
                None,
                Ok(()),
            );
            let post_id = create_post_mock(
                origin.clone(),
                forum_lead,
                thread_id,
                good_post_text(),
                Ok(()),
            );
            assert_eq!(
                TestForumModule::react_post(
                    mock_origin(origin.clone()),
                    forum_lead,
                    post_id,
                    new_values[index]
                ),
                Ok(())
            );
            assert_eq!(
                TestForumModule::reaction_by_post(post_id, forum_lead),
                new_values[index]
            );
        });
    }
}

/*
 ** moderate_post
 */

#[test]
// test if post moderator registered
fn moderate_post_origin() {
    let origins = vec![FORUM_LEAD_ORIGIN, NOT_FORUM_LEAD_ORIGIN];
    let results = vec![Ok(()), Err(ERROR_MODERATOR_ID_NOT_MATCH_ACCOUNT)];
    for index in 0..origins.len() {
        let config = default_genesis_config();
        let forum_lead = FORUM_LEAD_ORIGIN_ID;
        let origin = OriginType::Signed(forum_lead);
        build_test_externalities(config).execute_with(|| {
            let moderator_id = forum_lead;

            let category_id = create_category_mock(
                origin.clone(),
                None,
                good_category_title(),
                good_category_description(),
                Ok(()),
            );
            update_category_membership_of_moderator_mock(
                origin.clone(),
                moderator_id,
                category_id,
                true,
                Ok(()),
            );

            let thread_id = create_thread_mock(
                origin.clone(),
                forum_lead,
                category_id,
                good_thread_title(),
                good_thread_text(),
                None,
                Ok(()),
            );
            let post_id = create_post_mock(
                origin.clone(),
                forum_lead,
                thread_id,
                good_post_text(),
                Ok(()),
            );
            moderate_post_mock(
                origins[index].clone(),
                moderator_id,
                post_id,
                results[index],
            );
        });
    }
}

#[test]
fn set_stickied_threads_ok() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        set_stickied_threads_mock(origin, moderator_id, category_id, vec![thread_id], Ok(()));
    });
}

#[test]
fn set_stickied_threads_wrong_moderator() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );

        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        set_stickied_threads_mock(
            origin,
            moderator_id,
            category_id,
            vec![thread_id],
            Err(ERROR_MODERATOR_MODERATE_CATEGORY),
        );
    });
}

#[test]
fn set_stickied_threads_thread_not_exists() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        let wrong_thread_id = thread_id + 1;
        set_stickied_threads_mock(
            origin,
            moderator_id,
            category_id,
            vec![wrong_thread_id],
            Err(ERROR_THREAD_DOES_NOT_EXIST),
        );
    });
}

#[test]
fn set_stickied_threads_wrong_category() {
    let config = default_genesis_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let moderator_id = forum_lead;
        let category_id = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        update_category_membership_of_moderator_mock(
            origin.clone(),
            moderator_id,
            category_id,
            true,
            Ok(()),
        );
        let _ = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        let category_id_2 = create_category_mock(
            origin.clone(),
            None,
            good_category_title(),
            good_category_description(),
            Ok(()),
        );
        let thread_id = create_thread_mock(
            origin.clone(),
            forum_lead,
            category_id_2,
            good_thread_title(),
            good_thread_text(),
            None,
            Ok(()),
        );
        set_stickied_threads_mock(
            origin,
            moderator_id,
            category_id,
            vec![thread_id],
            Err(ERROR_THREAD_WITH_WRONG_CATEGORY_ID),
        );
    });
}

#[test]
fn test_migration_not_done() {
    let config = migration_not_done_config();
    let forum_lead = FORUM_LEAD_ORIGIN_ID;
    let origin = OriginType::Signed(forum_lead);
    build_test_externalities(config).execute_with(|| {
        let forum_user_id = 1;
        let moderator_id = 1;
        let category_id = 1;
        let thread_id = 1;
        let post_id = 1;

        assert_eq!(
            TestForumModule::create_category(
                mock_origin(origin.clone()),
                None,
                good_category_title(),
                good_category_description()
            ),
            Err(ERROR_DATA_MIGRATION_NOT_DONE),
        );

        assert_eq!(
            TestForumModule::create_thread(
                mock_origin(origin.clone()),
                forum_user_id,
                category_id,
                good_thread_title(),
                good_thread_text(),
                None,
            ),
            Err(ERROR_DATA_MIGRATION_NOT_DONE),
        );

        assert_eq!(
            TestForumModule::add_post(
                mock_origin(origin.clone()),
                forum_user_id,
                thread_id,
                good_post_text(),
            ),
            Err(ERROR_DATA_MIGRATION_NOT_DONE),
        );

        assert_eq!(
            TestForumModule::moderate_thread(mock_origin(origin.clone()), moderator_id, thread_id,),
            Err(ERROR_DATA_MIGRATION_NOT_DONE),
        );

        assert_eq!(
            TestForumModule::moderate_post(mock_origin(origin.clone()), moderator_id, post_id,),
            Err(ERROR_DATA_MIGRATION_NOT_DONE),
        );
    });
}
