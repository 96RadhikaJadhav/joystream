// Copyright 2017-2019 Parity Technologies (UK) Ltd.

// This is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

// Copyright 2019 Joystream Contributors

//! # Runtime Example Module
//!
//! <!-- Original author of paragraph: @gavofyork -->
//! The Example: A simple example of a runtime module demonstrating
//! concepts, APIs and structures common to most runtime modules.
//!
//! Run `cargo doc --package runtime-example-module --open` to view this module's documentation.
//!
//! ### Documentation Template:<br>
//! Add heading with custom module name
//!
//! # <INSERT_CUSTOM_MODULE_NAME> Module
//!
//! Add simple description
//!
//! Include the following links that shows what trait needs to be implemented to use the module
//! and the supported dispatchables that are documented in the Call enum.
//!
//! - [`<INSERT_CUSTOM_MODULE_NAME>::Trait`](./trait.Trait.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! <!-- Original author of paragraph: Various. See https://github.com/paritytech/substrate-developer-hub/issues/44 -->
//! Short description of module purpose.
//! Links to Traits that should be implemented.
//! What this module is for.
//! What functionality the module provides.
//! When to use the module (use case examples).
//! How it is used.
//! Inputs it uses and the source of each input.
//! Outputs it produces.
//!
//! <!-- Original author of paragraph: @Kianenigma in PR https://github.com/paritytech/substrate/pull/1951 -->
//! <!-- and comment https://github.com/paritytech/substrate-developer-hub/issues/44#issuecomment-471982710 -->
//!
//! ## Terminology
//!
//! Add terminology used in the custom module. Include concepts, storage items, or actions that you think
//! deserve to be noted to give context to the rest of the documentation or module usage. The author needs to
//! use some judgment about what is included. We don't want a list of every storage item nor types - the user
//! can go to the code for that. For example, "transfer fee" is obvious and should not be included, but
//! "free balance" and "reserved balance" should be noted to give context to the module.
//! Please do not link to outside resources. The reference docs should be the ultimate source of truth.
//!
//! <!-- Original author of heading: @Kianenigma in PR https://github.com/paritytech/substrate/pull/1951 -->
//!
//! ## Goals
//!
//! Add goals that the custom module is designed to achieve.
//!
//! <!-- Original author of heading: @Kianenigma in PR https://github.com/paritytech/substrate/pull/1951 -->
//!
//! ### Scenarios
//!
//! <!-- Original author of paragraph: @Kianenigma. Based on PR https://github.com/paritytech/substrate/pull/1951 -->
//!
//! #### <INSERT_SCENARIO_NAME>
//!
//! Describe requirements prior to interacting with the custom module.
//! Describe the process of interacting with the custom module for this scenario and public API functions used.
//!
//! ## Interface
//!
//! ### Supported Origins
//!
//! What origins are used and supported in this module (root, signed, inherent)
//! i.e. root when `ensure_root` used
//! i.e. inherent when `ensure_inherent` used
//! i.e. signed when `ensure_signed` used
//!
//! `inherent` <INSERT_DESCRIPTION>
//!
//! <!-- Original author of paragraph: @Kianenigma in comment -->
//! <!-- https://github.com/paritytech/substrate-developer-hub/issues/44#issuecomment-471982710 -->
//!
//! ### Types
//!
//! Type aliases. Include any associated types and where the user would typically define them.
//!
//! `ExampleType` <INSERT_DESCRIPTION>
//!
//! <!-- Original author of paragraph: ??? -->
//!
//!
//! ### Dispatchable Functions
//!
//! <!-- Original author of paragraph: @AmarRSingh & @joepetrowski -->
//!
//! // A brief description of dispatchable functions and a link to the rustdoc with their actual documentation.
//!
//! <b>MUST</b> have link to Call enum
//! <b>MUST</b> have origin information included in function doc
//! <b>CAN</b> have more info up to the user
//!
//! ### Public Functions
//!
//! <!-- Original author of paragraph: @joepetrowski -->
//!
//! A link to the rustdoc and any notes about usage in the module, not for specific functions.
//! For example, in the balances module: "Note that when using the publicly exposed functions,
//! you (the runtime developer) are responsible for implementing any necessary checks
//! (e.g. that the sender is the signer) before calling a function that will affect storage."
//!
//! <!-- Original author of paragraph: @AmarRSingh -->
//!
//! It is up to the writer of the respective module (with respect to how much information to provide).
//!
//! #### Public Inspection functions - Immutable (getters)
//!
//! Insert a subheading for each getter function signature
//!
//! ##### `example_getter_name()`
//!
//! What it returns
//! Why, when, and how often to call it
//! When it could panic or error
//! When safety issues to consider
//!
//! #### Public Mutable functions (changing state)
//!
//! Insert a subheading for each setter function signature
//!
//! ##### `example_setter_name(origin, parameter_name: T::ExampleType)`
//!
//! What state it changes
//! Why, when, and how often to call it
//! When it could panic or error
//! When safety issues to consider
//! What parameter values are valid and why
//!
//! ### Storage Items
//!
//! Explain any storage items included in this module
//!
//! ### Digest Items
//!
//! Explain any digest items included in this module
//!
//! ### Inherent Data
//!
//! Explain what inherent data (if any) is defined in the module and any other related types
//!
//! ### Events:
//!
//! Insert events for this module if any
//!
//! ### Errors:
//!
//! Explain what generates errors
//!
//! ## Usage
//!
//! Insert 2-3 examples of usage and code snippets that show how to use <INSERT_CUSTOM_MODULE_NAME> module in a custom module.
//!
//! ### Prerequisites
//!
//! Show how to include necessary imports for <INSERT_CUSTOM_MODULE_NAME> and derive
//! your module configuration trait with the `INSERT_CUSTOM_MODULE_NAME` trait.
//!
//! ```rust
//! // use <INSERT_CUSTOM_MODULE_NAME>;
//!
//! // pub trait Trait: <INSERT_CUSTOM_MODULE_NAME>::Trait { }
//! ```
//!
//! ### Simple Code Snippet
//!
//! Show a simple example (e.g. how to query a public getter function of <INSERT_CUSTOM_MODULE_NAME>)
//!
//! ## Genesis Config
//!
//! <!-- Original author of paragraph: @joepetrowski -->
//!
//! ## Dependencies
//!
//! Dependencies on other SRML modules and the genesis config should be mentioned,
//! but not the Rust Standard Library.
//! Genesis configuration modifications that may be made to incorporate this module
//! Interaction with other modules
//!
//! <!-- Original author of heading: @AmarRSingh -->
//!
//! ## Related Modules
//!
//! Interaction with other modules in the form of a bullet point list
//!
//! ## References
//!
//! <!-- Original author of paragraph: @joepetrowski -->
//!
//! Links to reference material, if applicable. For example, Phragmen, W3F research, etc.
//! that the implementation is based on.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[macro_use]
extern crate serde_derive;

use rstd::prelude::*;

use parity_codec_derive::{Decode, Encode};
use srml_support::{ decl_event, decl_module, decl_storage, ensure, dispatch, StorageValue, StorageMap};

/// Length constraint for input validation

// Drop later!!!
enum LengthValidationResult {
    TooShort,
    TooLong,
    Success
}


/*
 * MOVE ALL OF THESE OUT TO COMMON LATER
 */

struct InputValidationLengthConstraint {

    /// Minimum length
    min : usize,

    /// Maximum length
    /// While having max would have been more direct, this
    /// way makes max < min unrepresentable semantically, 
    /// which is safer.
    max_as_min_diff: usize,

    /*

    add later after review from Alex done

    too_short_error_msg: &'static str,

    too_long_error_msg: &'static str,
    */

}

impl InputValidationLengthConstraint {
    
    /// Helper for computing max
    fn max(&self) -> usize {
        self.min + self.max_as_min_diff
    }

    /// Just to give method interface to read min, like max
    fn min(&self) -> usize {
        self.min
    }

    fn validate(&self, length: usize) -> LengthValidationResult {

        if length < self.min {
            LengthValidationResult::TooShort
        }
        else if length > self.max() {
            LengthValidationResult::TooLong
        }
        else {
            LengthValidationResult::Success
        }
    } 

    // TODO: add more vliadtion stuff here in the future?
}

/// Constants
const CATEGORY_TITLE: InputValidationLengthConstraint = InputValidationLengthConstraint{
    min: 3,
    max_as_min_diff: 30
};

const CATEGORY_DESCRIPTION: InputValidationLengthConstraint = InputValidationLengthConstraint{
    min: 10,
    max_as_min_diff: 140
};

const THREAD_TITLE: InputValidationLengthConstraint = InputValidationLengthConstraint{
    min: 3,
    max_as_min_diff: 43
};

const POST_TEXT: InputValidationLengthConstraint = InputValidationLengthConstraint{
    min: 1,
    max_as_min_diff: 1001
};

const THREAD_MODERATION_RATIONALE: InputValidationLengthConstraint = InputValidationLengthConstraint{
    min: 100,
    max_as_min_diff: 2000
};

/// The greatest valid depth of a category.
/// The depth of a root category is 0.
const MAX_CATEGORY_DEPTH: u32 = 3;

/// Error messages for dispatchables
/// Later perhaps make error message functions, to add parametization

const ERROR_FORUM_SUDO_NOT_SET: &str = "Forum sudo not set.";
const ERROR_ORIGIN_NOT_FORUM_SUDO: &str = "Origin not forum sudo.";

const ERROR_CATEGORY_TITLE_TOO_SHORT: &str = "Category title too short.";
const ERROR_CATEGORY_TITLE_TOO_LONG: &str = "Category title too long.";

const ERROR_CATEGORY_DESCRIPTION_TOO_SHORT: &str = "Category description too long.";
const ERROR_CATEGORY_DESCRIPTION_TOO_LONG: &str = "Category description too long.";

//  drop, using ERROR_CATEGORY_DOES_NOT_EXIST instead of const ERROR_PARENT_CATEGORY_DOES_NOT_EXIST: &str = "Parent category does not exist.";
const ERROR_ANCESTOR_CATEGORY_IMMUTABLE: &str = "Ancestor category immutable, i.e. deleted or archived";
const ERROR_MAX_VALID_CATEGORY_DEPTH_EXCEEDED: &str = "Maximum valid category depth exceeded.";

const ERROR_CATEGORY_DOES_NOT_EXIST: &str = "Category does not exist.";

const ERROR_NOT_FORUM_USER: &str = "Not forum user.";

const ERROR_THREAD_TITLE_TOO_SHORT: &str = "Thread title too short.";
const ERROR_THREAD_TITLE_TOO_LONG: &str = "Thread title too long.";

const ERROR_POST_TEXT_TOO_SHORT: &str = "Post text too short.";
const ERROR_POST_TEXT_TOO_LONG: &str = "Post too long.";

const ERROR_THREAD_DOES_NOT_EXIST: &str = "Thread does not exist";

const ERROR_THREAD_MODERATION_RATIONALE_TOO_SHORT: &str = "Thread moderation rationale too short.";
const ERROR_THREAD_MODERATION_RATIONALE_TOO_LONG: &str = "Thread moderation rationale too long.";

const ERROR_THREAD_ALREADY_MODERATED: &str = "Thread alrady moderated.";

//use srml_support::storage::*;

//use sr_io::{StorageOverlay, ChildrenStorageOverlay};

//#[cfg(feature = "std")]
//use runtime_io::{StorageOverlay, ChildrenStorageOverlay};

//#[cfg(any(feature = "std", test))]
//use sr_primitives::{StorageOverlay, ChildrenStorageOverlay};

use system::{ensure_signed};
use system;

use rstd::collections::btree_map::BTreeMap;

/// Constant values
/// Later add to st
///

/// Represents a user in this forum.
#[derive(Debug, Copy, Clone)]
pub struct ForumUser<AccountId> {
    
    /// Identifier of user 
    id: AccountId

    // In the future one could add things like 
    // - updating post count of a user
    // - updating status (e.g. hero, new, etc.)
    //

}

/// Represents a regsitry of `ForumUser` instances.
pub trait ForumUserRegistry<AccountId> {
    
    fn get_forum_user(id: &AccountId) -> Option<ForumUser<AccountId>>;

}

/// Convenient composite time stamp 
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct BlockchainTimestamp<BlockNumber, Moment> {
    block : BlockNumber,
    time: Moment
}

/// Represents a moderation outcome applied to a post or a thread. 
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ModerationAction<BlockNumber, Moment, AccountId> {

    /// When action occured.
    moderated_at: BlockchainTimestamp<BlockNumber, Moment>,

    /// Account forum sudo which acted.
    moderator_id: AccountId,

    /// Moderation rationale
    rationale: Vec<u8>

}

/// Represents a revision of the text of a Post
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct PostTextChange<BlockNumber, Moment> {

    /// When this expiration occured
    expired_at: BlockchainTimestamp<BlockNumber, Moment>,

    /// Text that expired
    text: Vec<u8>
}

/// Represents a post identifier
pub type PostId = u64;

/// Represents a thread post
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Post<BlockNumber, Moment, AccountId> {

    /// Post identifier
    id: PostId,

    /// Id of thread to which this post corresponds.
    thread_id: ThreadId,

    /// The post number of this post in its thread, i.e. total number of posts added (including this)
    /// to a thread when it was added.
    /// Is needed to give light clients assurance about getting all posts in a given range,
    // `created_at` is not sufficient.
    /// Starts at 1 for first post in thread.
    nr_in_thread: u32,

    /// Current text of post
    current_text: Vec<u8>,

    /// Possible moderation of this post
    moderation : Option<ModerationAction<BlockNumber, Moment, AccountId>>,
    
    /// Edits of post ordered chronologically by edit time.
    text_change_history: Vec<PostTextChange<BlockNumber, Moment>>,

    /// When post was submitted.
    created_at : BlockchainTimestamp<BlockNumber, Moment>,
    
    /// Author of post.
    author_id : AccountId

}

/// Represents a thread identifier
pub type ThreadId = u64;

/// Represents a thread
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Thread<BlockNumber, Moment, AccountId> {

    /// Thread identifier
    id : ThreadId,

    /// Title
    title : Vec<u8>,

    /// Category in which this thread lives
    category_id: CategoryId,

    /// The thread number of this thread in its category, i.e. total number of thread added (including this)
    /// to a category when it was added.
    /// Is needed to give light clients assurance about getting all threads in a given range,
    /// `created_at` is not sufficient.
    /// Starts at 1 for first thread in category.
    nr_in_category: u32,

    /// Possible moderation of this thread
    moderation : Option<ModerationAction<BlockNumber, Moment, AccountId>>,

    /// Number of unmoderated and moderated posts in this thread.
    /// The sum of these two only increases, and former is incremented
    /// for each new post added to this thread. A new post is added 
    /// with a `nr_in_thread` equal to this sum
    /// 
    /// When there is a moderation
    /// of a post, the variables are incremented and decremented, respectively.
    /// 
    /// These values are vital for light clients, in order to validate that they are
    /// not being censored from posts in a thread.
    num_unmoderated_posts: u32,
    num_moderated_posts: u32,

    /// When thread was established.
    created_at : BlockchainTimestamp<BlockNumber, Moment>,
    
    /// Author of post.
    author_id : AccountId
}

/// Represents a category identifier
pub type CategoryId = u64;

/// Represents 
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct ChildPositionInParentCategory {

    /// Id of parent category
    parent_id: CategoryId,

    /// Nr of the child in the parent
    /// Starts at 1
    child_nr_in_parent_category: u32
}

/// Represents a category
//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Category<BlockNumber, Moment, AccountId> {

    /// Category identifier
    id : CategoryId,

    /// Title
    title : Vec<u8>,

    /// Description
    description: Vec<u8>,

    /// When category was established.
    created_at : BlockchainTimestamp<BlockNumber, Moment>,

    /// Whether category is deleted.
    deleted: bool,

    /// Whether category is archived.
    archived: bool,

    /// Number of subcategories (deleted, archived or neither),
    /// unmoderated threads and moderated threads, _directly_ in this category.
    /// 
    /// As noted, the first is unaffected by any change in state of direct subcategory.
    ///  
    /// The sum of the latter two only increases, and former is incremented
    /// for each new thread added to this category. A new thread is added 
    /// with a `nr_in_category` equal to this sum.
    /// 
    /// When there is a moderation
    /// of a thread, the variables are incremented and decremented, respectively.
    /// 
    /// These values are vital for light clients, in order to validate that they are
    /// not being censored from subcategories or threads in a category.
    num_direct_subcategories: u32,
    num_direct_unmoderated_threads: u32,
    num_direct_moderated_threads: u32,

    /// Position as child in parent, if present, otherwise this category is a root category
    position_in_parent_category: Option<ChildPositionInParentCategory>,

    /// Account of the moderator which created category.
    moderator_id: AccountId
}

impl<BlockNumber, Moment, AccountId> Category<BlockNumber, Moment, AccountId>  {

    fn num_threads_created(&self) -> u32 {
        self.num_direct_unmoderated_threads + self.num_direct_moderated_threads
    }
}

/// Represents a sequence of categories which have child-parent relatioonship
/// where last element is final ancestor, or root, in the context of the category tree.
type CategoryTreePath<BlockNumber, Moment, AccountId> = Vec<Category<BlockNumber, Moment, AccountId>>;

pub trait Trait: system::Trait + timestamp::Trait + Sized {

    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type MembershipRegistry: ForumUserRegistry<Self::AccountId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Forum {

        /// Map category identifier to corresponding category.
        pub CategoryById get(category_by_id) config(): map CategoryId => Category<T::BlockNumber, T::Moment, T::AccountId>;

        /// Category identifier value to be used for the next Category created.
        pub NextCategoryId get(next_category_id) config(): CategoryId;

        /// Map thread identifier to corresponding thread.
        pub ThreadById get(thread_by_id) config(): map ThreadId => Thread<T::BlockNumber, T::Moment, T::AccountId>;

        /// Thread identifier value to be used for next Thread in threadById.
        pub NextThreadId get(next_thread_id) config(): ThreadId;

        /// Map post identifier to corresponding post.
        pub PostById get(post_by_id) config(): map PostId => Post<T::BlockNumber, T::Moment, T::AccountId>;

        /// Post identifier value to be used for for next post created.
        pub NextPostId get(next_post_id) config(): PostId;

        /// Account of forum sudo.
        pub ForumSudo get(forum_sudo) config(): Option<T::AccountId>;

        // === Add constraints here ===

        // Will add all the constrainst here later!

    }
    /*
    JUST GIVING UP ON ALL THIS FOR NOW BECAUSE ITS TAKING TOO LONG
    Review : https://github.com/paritytech/polkadot/blob/620b8610431e7b5fdd71ce3e94c3ee0177406dcc/runtime/src/parachains.rs#L123-L141

    add_extra_genesis {

        // Explain why we need to put this here.
        config(initial_forum_sudo) : Option<T::AccountId>;

        build(|
            storage: &mut generator::StorageOverlay, 
            _: &mut generator::ChildrenStorageOverlay,
            config: &GenesisConfig<T>
            | {


			if let Some(account_id) = &config.initial_forum_sudo {
                println!("{}: <ForumSudo<T>>::put(account_id)", account_id); 
                <ForumSudo<T> as generator::StorageValue<_>>::put(&account_id, storage);
            }
		})
    }
    */
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
    {
        /// A category was introduced
        CategoryCreated(CategoryId),

        /// A category with given id was updated. 
        /// The second argument reflects the new archival status of the category, if changed.
        /// The third argument reflects the new deletion status of the category, if changed.
        CategoryUpdated(CategoryId, Option<bool>, Option<bool>),

        /// A thread with given id was created.
        ThreadCreated(ThreadId),

        /// A thread with given id was moderated.
        ThreadModerated(ThreadId),

        /// Post with given id was created.
        PostAdded(PostId),

        /// Post with givne id was moderated.
        PostModerated(PostId),

        /// Post with given id had its text updated.
        /// The second argument reflects the number of total edits when the text update occurs.
        PostTextUpdated(PostId, u64),

        /// Given account was set as forum sudo.
        ForumSudoSet(Option<AccountId>, Option<AccountId>),
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        /// Set forum sudo.
        fn set_forum_sudo(newForumSudo: Option<T::AccountId>) -> dispatch::Result {

            /*
             * Question: when this routine is called by non sudo or with bad signature, what error is raised?
             * Update ERror set in spec
             */

            // Hold on to old value
            let oldForumSudo = <ForumSudo<T>>::get().clone();

            // Update forum sudo
            match newForumSudo.clone() {
                Some(accountId) => <ForumSudo<T>>::put(accountId),
                None => <ForumSudo<T>>::kill()
            };

            // Generate event
            Self::deposit_event(RawEvent::ForumSudoSet(oldForumSudo, newForumSudo));

            // All good.
            Ok(())
        }

        /// Add a new category.
        fn create_category(origin, parent: Option<CategoryId>, title: Vec<u8>, description: Vec<u8>) -> dispatch::Result {

            // Check that its a valid signature
            let who = ensure_signed(origin)?;

            // Not signed by forum SUDO
            Self::ensure_is_forum_sudo(&who)?;

            // Validate title
            ensure_category_title_is_valid(&title)?;

            // Validate description
            ensure_category_description_is_valid(&description)?;

            // Position in parent field value for new category
            let mut position_in_parent_category_field = None;

            // If not root, then check that we can create in parent category
            if let Some(parent_category_id) = parent {

                let category_tree_path = Self::ensure_valid_category_and_build_category_tree_path(parent_category_id)?;

                // Can we mutate in this category?
                Self::ensure_can_add_subcategory_path_leaf(&category_tree_path)?;

                /*
                 * Here we are safe to mutate
                 */

                // Increment number of subcategories to reflect this new category being
                // added as a child
                <CategoryById<T>>::mutate(parent_category_id, |c| {
                    c.num_direct_subcategories += 1;
                });

                // Set `position_in_parent_category_field`
                let parent_category = category_tree_path.first().unwrap();

                position_in_parent_category_field = Some(ChildPositionInParentCategory{
                    parent_id: parent_category_id,
                    child_nr_in_parent_category: parent_category.num_direct_subcategories
                });

            }

            /*
             * Here we are safe to mutate
             */

            let next_category_id = <NextCategoryId<T>>::get();

            // Create new category
            let new_category = Category {
                id : next_category_id,
                title : title.clone(),
                description: description.clone(),
                created_at : Self::current_block_and_time(),
                deleted: false,
                archived: false,
                num_direct_subcategories: 0,
                num_direct_unmoderated_threads: 0,
                num_direct_moderated_threads: 0,
                position_in_parent_category: position_in_parent_category_field,
                moderator_id: who
            };

            // Insert category in map
            <CategoryById<T>>::insert(new_category.id, new_category);

            // Update other things
            <NextCategoryId<T>>::put(next_category_id + 1);

            // Generate event
            Self::deposit_event(RawEvent::CategoryCreated(next_category_id));

            Ok(())
        }
        
        /// Update category
        fn update_category(origin, category_id: CategoryId, archived: bool, deleted: bool) -> dispatch::Result {

            // Check that its a valid signature
            let who = ensure_signed(origin)?;

            // Not signed by forum SUDO
            Self::ensure_is_forum_sudo(&who)?;

            // Get path from parent to root of category tree.
            let mut category_tree_path = Self::ensure_valid_category_and_build_category_tree_path(category_id)?;

            // Make sure we can actually mutate this category
            Self::ensure_can_mutate_in_path_leaf(&category_tree_path)?;

            // Grab mutable category for updating
            let mut category = category_tree_path.first().unwrap().clone();

            // Value change events params
            let mut archived_change_to = None;
            let mut deleted_changed_to = None;

            // Mutate category, and set possible new change parameters

            if archived != category.archived {
                category.archived = archived;
                archived_change_to = Some(archived);
            }

            if deleted != category.deleted {
                category.deleted = deleted;
                deleted_changed_to = Some(deleted)
            }

            // Write back mutated category
            <CategoryById<T>>::insert(category_id, category);

            // Generate event
            Self::deposit_event(RawEvent::CategoryUpdated(category_id, archived_change_to, deleted_changed_to));

            Ok(())
        }

        /// Create new thread in category
        fn create_thread(origin, category_id: CategoryId, title: Vec<u8>, text: Vec<u8>) -> dispatch::Result {

            /*
             * Update SPEC with new errors,
             * and mutation of Category class,
             * as well as side effect to update Category::num_threads_created.
             */ 

            // Check that its a valid signature
            let who = ensure_signed(origin)?;

            // Check that account is forum member
            Self::ensure_is_forum_member(&who)?;

            // Get path from parent to root of category tree.
            let category_tree_path = Self::ensure_valid_category_and_build_category_tree_path(category_id)?;

            // No ancestor is blocking us doing mutation in this category
            Self::ensure_can_mutate_in_path_leaf(&category_tree_path)?;
            
            // Validate title
            ensure_thread_title_is_valid(&title)?;

            // Validate post text
            ensure_post_text_is_valid(&text)?;

            /*
             * Here it is safe to mutate state.
             */

            // Add thread
            let thread = Self::add_new_thread(category_id, &title, &who);

            assert_eq!(category.id, category_id);
            
            // Make and add initial post

            // TODO: perhaps factor out later?

            let new_post_id = <NextPostId<T>>::get();

            <NextPostId<T>>::put(new_post_id + 1);

            // Create and add new post
            let new_post = Post {
                id: new_post_id,
                thread_id: new_thread_id,
                nr_in_thread: 1, // Starts at 1
                current_text: text.clone(),
                moderation : None,
                text_change_history: vec![],
                created_at : Self::current_block_and_time(),
                author_id : who.clone()
            };

            <PostById<T>>::insert(new_post_id, new_post);

            // Generate event
            Self::deposit_event(RawEvent::ThreadCreated(new_thread_id));

            Ok(())
        }

        /// Moderate thread
        fn moderate_thread(origin, thread_id: ThreadId, rationale: Vec<u8>) -> dispatch::Result {

            // Check that its a valid signature
            let who = ensure_signed(origin)?;
            
            // Signed by forum SUDO
            Self::ensure_is_forum_sudo(&who)?;

            // Get thread
            let mut thread = Self::ensure_thread_exists(&thread_id)?;

            // Thread is not already moderated
            ensure!(thread.moderation.is_none(), ERROR_THREAD_ALREADY_MODERATED);

            // Rationale valid
            ensure_thread_moderation_rationale_is_valid(&rationale)?;

            // Can mutate in corresponding category
            let path = Self::build_category_tree_path(thread.category_id);

            // Path must be non-empty, as category id is from thread in state
            assert!(!path.is_empty());

            Self::ensure_can_mutate_in_path_leaf(&path)?;

            /*
             * Here we are safe to mutate
             */

            // Add moderation to thread
            thread.moderation = Some(ModerationAction {
                moderated_at: Self::current_block_and_time(),
                moderator_id: who,
                rationale: rationale.clone()
            });

            <ThreadById<T>>::insert(thread_id, thread.clone());

            // Update moderation/umoderation count of corresponding category
            <CategoryById<T>>::mutate(thread.category_id, |category| {
                category.num_direct_unmoderated_threads -= 1;
                category.num_direct_moderated_threads += 1;
            });

            // Generate event
            Self::deposit_event(RawEvent::ThreadModerated(thread_id));

            Ok(())
        }

    }
}

/*
 * Drop all of these later
 */

fn ensure_thread_moderation_rationale_is_valid(rationale: &Vec<u8>) -> dispatch::Result {

    match THREAD_MODERATION_RATIONALE.validate(rationale.len()) {
        LengthValidationResult::TooShort => Err(ERROR_THREAD_MODERATION_RATIONALE_TOO_SHORT),
        LengthValidationResult::TooLong => Err(ERROR_THREAD_MODERATION_RATIONALE_TOO_LONG),
        LengthValidationResult::Success => Ok(())
    }
}

fn ensure_category_title_is_valid(title: &Vec<u8>) -> dispatch::Result {

    match CATEGORY_TITLE.validate(title.len()) {
        LengthValidationResult::TooShort => Err(ERROR_CATEGORY_TITLE_TOO_SHORT),
        LengthValidationResult::TooLong => Err(ERROR_CATEGORY_TITLE_TOO_LONG),
        LengthValidationResult::Success => Ok(())
    }
}

fn ensure_category_description_is_valid(description: &Vec<u8>) -> dispatch::Result {

    match CATEGORY_DESCRIPTION.validate(description.len()) {
        LengthValidationResult::TooShort => Err(ERROR_CATEGORY_DESCRIPTION_TOO_SHORT),
        LengthValidationResult::TooLong => Err(ERROR_CATEGORY_DESCRIPTION_TOO_LONG),
        LengthValidationResult::Success => Ok(())
    }

}

fn ensure_thread_title_is_valid(title: &Vec<u8>) -> dispatch::Result {

    match THREAD_TITLE.validate(title.len()) {
        LengthValidationResult::TooShort => Err(ERROR_THREAD_TITLE_TOO_SHORT),
        LengthValidationResult::TooLong => Err(ERROR_THREAD_TITLE_TOO_LONG),
        LengthValidationResult::Success => Ok(())
    }
}

fn ensure_post_text_is_valid(text: &Vec<u8>) -> dispatch::Result {

    match THREAD_TITLE.validate(text.len()) {
        LengthValidationResult::TooShort => Err(ERROR_POST_TEXT_TOO_SHORT),
        LengthValidationResult::TooLong => Err(ERROR_POST_TEXT_TOO_LONG),
        LengthValidationResult::Success => Ok(())
    }
}

impl<T: Trait> Module<T> {

    fn current_block_and_time() -> BlockchainTimestamp<T::BlockNumber, T::Moment> {

        BlockchainTimestamp {
            block: <system::Module<T>>::block_number(),
            time: <timestamp::Module<T>>::now(),
        }
    }

    fn ensure_thread_exists(thread_id: &ThreadId) -> Result<Thread<T::BlockNumber, T::Moment, T::AccountId>, &'static str> {

        if <ThreadById<T>>::exists(thread_id) {
            Ok(<ThreadById<T>>::get(thread_id))
        } else {
            Err(ERROR_THREAD_DOES_NOT_EXIST)
        }

    }

    fn ensure_forum_sudo_set() -> Result<T::AccountId, &'static str> {

        match <ForumSudo<T>>::get() {
            Some(account_id) => Ok(account_id),
            None => Err(ERROR_FORUM_SUDO_NOT_SET)
        }
    }

    fn ensure_is_forum_sudo(account_id: &T::AccountId) -> dispatch::Result {

        let forum_sudo_account = Self::ensure_forum_sudo_set()?;

        ensure!(
            *account_id == forum_sudo_account,
            ERROR_ORIGIN_NOT_FORUM_SUDO
        );
        Ok(())
    }

    fn ensure_is_forum_member(account_id: &T::AccountId) -> Result<ForumUser<T::AccountId>, &'static str> {

        let forum_user_query = T::MembershipRegistry::get_forum_user(account_id);

        if let Some(forum_user) = forum_user_query {
            Ok(forum_user)
        } else {
            Err(ERROR_NOT_FORUM_USER)
        }
    }

    fn ensure_can_mutate_in_path_leaf(category_tree_path:&CategoryTreePath<T::BlockNumber, T::Moment, T::AccountId>) -> dispatch::Result {

        // Is parent category directly or indirectly deleted or archived category
        ensure!(!category_tree_path.iter().any(|c:&Category<T::BlockNumber, T::Moment, T::AccountId>| c.deleted || c.archived ),
            ERROR_ANCESTOR_CATEGORY_IMMUTABLE
        );

        Ok(())
    }

    fn ensure_can_add_subcategory_path_leaf(category_tree_path:&CategoryTreePath<T::BlockNumber, T::Moment, T::AccountId>) -> dispatch::Result {

        Self::ensure_can_mutate_in_path_leaf(category_tree_path)?;

        // Does adding a new category exceed maximum depth
        let depth_of_new_category = 1 + 1 + category_tree_path.len();

        ensure!(depth_of_new_category <= MAX_CATEGORY_DEPTH as usize,
            ERROR_MAX_VALID_CATEGORY_DEPTH_EXCEEDED
        );

        Ok(())
    }

    fn ensure_valid_category_and_build_category_tree_path(category_id:CategoryId) -> Result<CategoryTreePath<T::BlockNumber, T::Moment, T::AccountId>, &'static str> {

        ensure!(<CategoryById<T>>::exists(&category_id), ERROR_CATEGORY_DOES_NOT_EXIST);

        // Get path from parent to root of category tree.
        let category_tree_path = Self::build_category_tree_path(category_id);

        assert!(category_tree_path.len() > 0);

        Ok(category_tree_path)
    }

    /// Builds path and populates in `path`.
    /// Requires that `category_id` is valid
    fn build_category_tree_path(category_id:CategoryId) -> CategoryTreePath<T::BlockNumber, T::Moment, T::AccountId> {

        // Get path from parent to root of category tree.
        let mut category_tree_path = vec![];
        
        Self::_build_category_tree_path(category_id, &mut category_tree_path);

        category_tree_path
    }

    /// Builds path and populates in `path`.
    /// Requires that `category_id` is valid
    fn _build_category_tree_path(category_id:CategoryId, path: &mut CategoryTreePath<T::BlockNumber, T::Moment, T::AccountId>) {

        // Grab category
        let category = <CategoryById<T>>::get(category_id);

        // Copy out position_in_parent_category
        let position_in_parent_category_field = category.position_in_parent_category.clone();

        // Add category to path container
        path.push(category);

        // Make recursive call on parent if we are not at root
        if let Some(child_position_in_parent) = position_in_parent_category_field {

            assert!(<CategoryById<T>>::exists(&child_position_in_parent.parent_id));

            Self::_build_category_tree_path(child_position_in_parent.parent_id, path);
        }
    }

    fn add_new_thread(category_id: CategoryId, title: &Vec<u8>, author_id: &T::AccountId) -> Thread<T::BlockNumber, T::Moment, T::AccountId> {

        // Get category
        let category = <CategoryById<T>>::get(category_id);

        // Create and add new thread
        let new_thread_id = <NextThreadId<T>>::get();

        let new_thread = Thread {
            id : new_thread_id,
            title : title.clone(),
            category_id: category_id,
            thread_nr_in_category: category.num_threads_ever_created() + 1,
            moderation : None,
            num_unmoderated_posts: 0,
            num_moderated_posts: 0,
            created_at : Self::current_block_and_time(),
            author_id : author_id.clone()
        };

        // Store thread
        <ThreadById<T>>::insert(new_thread_id, new_thread.clone());

        // Update next thread id
        <NextThreadId<T>>::mutate(|n| {
            *n += 1;
        });

        // Update unmoderated thread count in corresponding category
        <CategoryById<T>>::mutate(category_id, |c| {
            c.num_direct_unmoderated_threads += 1;
        });

        new_thread
    }

#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use srml_support::{impl_outer_origin, assert_ok}; // assert, assert_eq
    // The testing primitives are very useful for avoiding having to work with signatures
    // or public keys. `u64` is used as the `AccountId` and no `Signature`s are requried.
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup}, //OnFinalize, OnInitialize},
        BuildStorage,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
    }
    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
    }
    impl Trait for Test {
        type Event = ();
        type MembershipRegistry = MockForumUserRegistry; //< <Test as system::Trait>::AccountId >;
    }

    type TestForumModule = Module<Test>;

    // Data store for MockForumUserRegistry

    static mut forum_user_store: Option<BTreeMap< <Test as system::Trait>::AccountId , ForumUser< <Test as system::Trait>::AccountId > > > = None;

    fn initialize_forum_user_store() {

        //formUserStore

    }

    // MockForumUserRegistry
    pub struct MockForumUserRegistry { }

    impl ForumUserRegistry< <Test as system::Trait>::AccountId > for MockForumUserRegistry {

        fn get_forum_user(id: &<Test as system::Trait>::AccountId) -> Option<ForumUser< <Test as system::Trait>::AccountId >> {

            let query_result;
            
            unsafe {
                query_result = forum_user_store.as_ref().unwrap().get(&id);
            }
            
            if let Some(forum_user) = query_result {
                Some(forum_user.clone())
            } else {
                None
            }

        }
        
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.

    // refactor
    /// - add each config as parameter, then 
    /// 
    
    fn default_genesis_config() -> GenesisConfig<Test> {

        GenesisConfig::<Test> {
            category_by_id: vec![], // endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
            next_category_id: 0,
            thread_by_id: vec![],
            next_thread_id: 0,
            post_by_id: vec![],
            next_post_id: 0,

            forum_sudo: 33


            // JUST GIVING UP ON ALL THIS FOR NOW BECAUSE ITS TAKING TOO LONG

            // Extra genesis fields
            //initial_forum_sudo: Some(143)
        }
    }

    // Wanted to have payload: a: &GenesisConfig<Test>
    // but borrow checker made my life miserabl, so giving up for now.
    fn build_test_externalities() -> runtime_io::TestExternalities<Blake2Hasher> {

        let t = default_genesis_config()
            .build_storage()
            .unwrap()
            .0;

        t.into()
    }

    /*
     * NB!: No test checks for even emission!!!!
     */

    /*
     * set_forum_sudo 
     * ==============================================================================
     * 
     * Missing cases
     * 
     * set_forum_bad_origin
     * 
     */

    #[test]
    fn set_forum_sudo_unset() {
        with_externalities(&mut build_test_externalities(), || {

            // Ensure that forum sudo is default
            assert_eq!(TestForumModule::forum_sudo(), Some(33));

            // Unset forum sudo
            assert_ok!(TestForumModule::set_forum_sudo(None));

            // Sudo no longer set
            assert!(!<ForumSudo<Test>>::exists());

            // event emitted?!

        });
    }

    #[test]
    fn set_forum_sudo_update() {
        with_externalities(&mut build_test_externalities(), || {

            // Ensure that forum sudo is default
            assert_eq!(TestForumModule::forum_sudo(), Some(default_genesis_config().forum_sudo));

            let new_forum_sudo_account_id = 780;

            // Unset forum sudo
            assert_ok!(TestForumModule::set_forum_sudo(Some(new_forum_sudo_account_id)));

            // Sudo no longer set
            //assert!(!<ForumSudo<Test>>::exists());
            assert_eq!(<ForumSudo<Test>>::get(), Some(new_forum_sudo_account_id));

        });
    }

    /*
     * create_category 
     * ==============================================================================
     * 
     * Missing cases
     * 
     * create_category_bad_origin
     * create_category_forum_sudo_not_set
     * create_category_origin_not_forum_sudo
     * create_category_title_too_short
     * create_category_title_too_long
     * create_category_description_too_short
     * create_category_description_too_long
     */

    // Here are a few testing utilities and fixtures, will reorganize
    // later with more tests.

    enum OriginType {
        Signed(<Test as system::Trait>::AccountId),
        //Inherent, <== did not find how to make such an origin yet
        Root
    }

    struct CreateCategoryFixture {
        origin: OriginType,
        parent: Option<CategoryId>,
        title: Vec<u8>,
        description: Vec<u8>
    }

    impl CreateCategoryFixture {

        fn call_module(&self) -> dispatch::Result {

            TestForumModule::create_category(
                match self.origin {
                    OriginType::Signed(account_id) => Origin::signed(account_id),
                    //OriginType::Inherent => Origin::inherent,
                    OriginType::Root => system::RawOrigin::Root.into() //Origin::root
                },
                self.parent,
                self.title.clone(),
                self.description.clone()
            )
        }
    }

    #[test]
    fn create_category_successfully() {
        with_externalities(&mut build_test_externalities(), || {

            // Make some new catg
            let f1 = CreateCategoryFixture {
                origin: OriginType::Signed(default_genesis_config().forum_sudo),
                parent: None,
                title: "My new category".as_bytes().to_vec(),
                description: "This is a great new category for the forum".as_bytes().to_vec()
            };

            // let f2 = ...
            // let f3 = ...
            // let f4 = ...

            // Make module call
            f1.call_module().is_ok();

            // f2.call_module();
            // f3.call_module();
            // f4.call_module();

            // assert state!

        });
    }

    /*
     * update_category 
     * ==============================================================================
     * 
     * Missing cases
     * 
     * create_category_bad_origin
     * create_category_forum_sudo_not_set
     * create_category_origin_not_forum_sudo
     * create_category_immutable_ancestor_category
     */

    /*
     * create_thread 
     * ==============================================================================
     * 
     * Missing cases
     * 
     * create_thread_bad_origin
     * create_thread_forum_sudo_not_set
     * ...
     */




}
