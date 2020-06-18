// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use core::fmt::Debug;
use core::ops::AddAssign;
use std::hash::Hash;

use codec::{Codec, Decode, Encode};
use rstd::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use rstd::prelude::*;
use runtime_primitives::traits::{MaybeSerializeDeserialize, Member, One, SimpleArithmetic, Zero};
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get, Parameter,
    StorageDoubleMap,
};
use system::ensure_signed;

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

mod errors;
mod helpers;
mod mock;
mod operations;
mod permissions;
mod schema;
mod tests;

pub use errors::*;
pub use helpers::*;
pub use operations::*;
pub use permissions::*;
pub use schema::*;

/// Type, used in diffrent numeric constraints representations
type MaxNumber = u32;

pub trait Trait: system::Trait + ActorAuthenticator + Debug + Clone {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Nonce type is used to avoid data race update conditions, when performing property value vector operations
    type Nonce: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + Clone
        + One
        + Zero
        + MaybeSerializeDeserialize
        + Eq
        + PartialEq
        + Ord
        + From<u32>;

    /// Type of identifier for classes
    type ClassId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + Clone
        + One
        + Zero
        + MaybeSerializeDeserialize
        + Eq
        + PartialEq
        + Ord;

    /// Type of identifier for entities
    type EntityId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + Clone
        + Hash
        + One
        + Zero
        + MaybeSerializeDeserialize
        + Eq
        + PartialEq
        + Ord;

    /// Security/configuration constraints

    /// Type, representing min & max property name length constraints
    type PropertyNameLengthConstraint: Get<InputValidationLengthConstraint>;

    /// Type, representing min & max property description length constraints
    type PropertyDescriptionLengthConstraint: Get<InputValidationLengthConstraint>;

    /// Type, representing min & max class name length constraints
    type ClassNameLengthConstraint: Get<InputValidationLengthConstraint>;

    /// Type, representing min & max class description length constraints
    type ClassDescriptionLengthConstraint: Get<InputValidationLengthConstraint>;

    /// The maximum number of classes
    type MaxNumberOfClasses: Get<MaxNumber>;

    /// The maximum number of maintainers per class constraint
    type MaxNumberOfMaintainersPerClass: Get<MaxNumber>;

    /// The maximum number of curators per group constraint
    type MaxNumberOfCuratorsPerGroup: Get<MaxNumber>;

    /// The maximum number of schemas per class constraint
    type NumberOfSchemasPerClass: Get<MaxNumber>;

    /// The maximum number of properties per class constraint
    type MaxNumberOfPropertiesPerClass: Get<MaxNumber>;

    /// The maximum number of operations during single invocation of `transaction`
    type MaxNumberOfOperationsDuringAtomicBatching: Get<MaxNumber>;

    /// The maximum length of vector property value constarint
    type VecMaxLengthConstraint: Get<VecMaxLength>;

    /// The maximum length of text property value constarint
    type TextMaxLengthConstraint: Get<TextMaxLength>;

    /// Entities creation constraint per class
    type MaxNumberOfEntitiesPerClass: Get<Self::EntityId>;

    /// Entities creation constraint per individual
    type IndividualEntitiesCreationLimit: Get<Self::EntityId>;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Eq, PartialEq, Clone, Debug)]
pub struct Class<T: Trait> {
    /// Permissions for an instance of a Class.
    class_permissions: ClassPermissions<T>,
    /// All properties that have been used on this class across different class schemas.
    /// Unlikely to be more than roughly 20 properties per class, often less.
    /// For Person, think "height", "weight", etc.
    pub properties: Vec<Property<T>>,

    /// All schemas that are available for this class, think v0.0 Person, v.1.0 Person, etc.
    pub schemas: Vec<Schema>,

    pub name: Vec<u8>,

    pub description: Vec<u8>,

    /// The maximum number of entities which can be created.
    maximum_entities_count: T::EntityId,

    /// The current number of entities which exist.
    current_number_of_entities: T::EntityId,

    /// How many entities a given controller may create at most.
    default_entity_creation_voucher_upper_bound: T::EntityId,
}

impl<T: Trait> Default for Class<T> {
    fn default() -> Self {
        Self {
            class_permissions: ClassPermissions::<T>::default(),
            properties: vec![],
            schemas: vec![],
            name: vec![],
            description: vec![],
            maximum_entities_count: T::EntityId::default(),
            current_number_of_entities: T::EntityId::default(),
            default_entity_creation_voucher_upper_bound: T::EntityId::default(),
        }
    }
}

impl<T: Trait> Class<T> {
    /// Create new `Class` with provided parameters
    fn new(
        class_permissions: ClassPermissions<T>,
        name: Vec<u8>,
        description: Vec<u8>,
        maximum_entities_count: T::EntityId,
        default_entity_creation_voucher_upper_bound: T::EntityId,
    ) -> Self {
        Self {
            class_permissions,
            properties: vec![],
            schemas: vec![],
            name,
            description,
            maximum_entities_count,
            current_number_of_entities: T::EntityId::zero(),
            default_entity_creation_voucher_upper_bound,
        }
    }

    /// Used to update `Schema` status under given `schema_index`
    fn update_schema_status(&mut self, schema_index: SchemaId, schema_status: bool) {
        // Such indexing is safe, when length bounds were previously checked
        self.schemas[schema_index as usize].set_status(schema_status);
    }

    /// Used to update `Class` permissions
    fn update_permissions(&mut self, permissions: ClassPermissions<T>) {
        self.class_permissions = permissions
    }

    /// Increment number of entities, associated with this class
    fn increment_entities_count(&mut self) {
        self.current_number_of_entities += T::EntityId::one();
    }

    /// Decrement number of entities, associated with this class
    fn decrement_entities_count(&mut self) {
        self.current_number_of_entities -= T::EntityId::one();
    }

    /// Retrieve `ClassPermissions` by mutable reference
    fn get_permissions_mut(&mut self) -> &mut ClassPermissions<T> {
        &mut self.class_permissions
    }

    /// Retrieve `ClassPermissions` by reference
    fn get_permissions_ref(&self) -> &ClassPermissions<T> {
        &self.class_permissions
    }

    /// Retrieve `ClassPermissions` by value
    fn get_permissions(self) -> ClassPermissions<T> {
        self.class_permissions
    }

    /// Retrieve `Class` properties by value  
    fn get_properties(self) -> Vec<Property<T>> {
        self.properties
    }

    /// Get per controller `Class`- specific limit
    pub fn get_default_entity_creation_voucher_upper_bound(&self) -> T::EntityId {
        self.default_entity_creation_voucher_upper_bound
    }

    /// Retrive the maximum entities count, which can be created for given `Class`
    pub fn get_maximum_entities_count(&self) -> T::EntityId {
        self.maximum_entities_count
    }

    /// Ensure `Class` `Schema` under given index exist, return corresponding `Schema`
    fn ensure_schema_exists(&self, schema_index: SchemaId) -> Result<&Schema, &'static str> {
        self.schemas
            .get(schema_index as usize)
            .ok_or(ERROR_UNKNOWN_CLASS_SCHEMA_ID)
    }

    /// Ensure `schema_id` is a valid index of `Class` schemas vector
    pub fn ensure_schema_id_exists(&self, schema_id: SchemaId) -> dispatch::Result {
        ensure!(
            schema_id < self.schemas.len() as SchemaId,
            ERROR_UNKNOWN_CLASS_SCHEMA_ID
        );
        Ok(())
    }

    /// Ensure `Schema`s limit per `Class` not reached
    pub fn ensure_schemas_limit_not_reached(&self) -> dispatch::Result {
        ensure!(
            T::NumberOfSchemasPerClass::get() < self.schemas.len() as MaxNumber,
            ERROR_CLASS_SCHEMAS_LIMIT_REACHED
        );
        Ok(())
    }

    /// Ensure properties limit per `Class` not reached
    pub fn ensure_properties_limit_not_reached(
        &self,
        new_properties: &[Property<T>],
    ) -> dispatch::Result {
        ensure!(
            T::MaxNumberOfPropertiesPerClass::get()
                <= (self.properties.len() + new_properties.len()) as MaxNumber,
            ERROR_CLASS_PROPERTIES_LIMIT_REACHED
        );
        Ok(())
    }

    /// Ensure `Class` specific entities limit not reached
    pub fn ensure_maximum_entities_count_limit_not_reached(&self) -> dispatch::Result {
        ensure!(
            self.current_number_of_entities < self.maximum_entities_count,
            ERROR_MAX_NUMBER_OF_ENTITIES_PER_CLASS_LIMIT_REACHED
        );
        Ok(())
    }

    /// Ensure `Property` under given `PropertyId` is unlocked from actor with given `EntityAccessLevel`
    /// return corresponding `Property` by value
    pub fn ensure_class_property_type_unlocked_from(
        &self,
        in_class_schema_property_id: PropertyId,
        entity_access_level: EntityAccessLevel,
    ) -> Result<Property<T>, &'static str> {
        // Ensure property values were not locked on Class level
        self.ensure_property_values_unlocked()?;

        // Get class-level information about this `Property`
        let class_property = self
            .properties
            .get(in_class_schema_property_id as usize)
            // Throw an error if a property was not found on class
            // by an in-class index of a property.
            .ok_or(ERROR_CLASS_PROP_NOT_FOUND)?;

        // Ensure Property is unlocked from Actor with given EntityAccessLevel
        class_property.ensure_unlocked_from(entity_access_level)?;

        Ok(class_property.to_owned())
    }

    /// Ensure property values were not locked on `Class` level
    pub fn ensure_property_values_unlocked(&self) -> dispatch::Result {
        ensure!(
            !self
                .get_permissions_ref()
                .all_entity_property_values_locked(),
            ERROR_ALL_PROP_WERE_LOCKED_ON_CLASS_LEVEL
        );
        Ok(())
    }
}

/// Structure, respresenting inbound entity rcs for each `Entity`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Copy, Debug)]
pub struct InboundReferenceCounter {
    /// Total number of inbound references from another entities
    total: u32,
    /// Number of inbound references from another entities with `SameOwner` flag set
    same_owner: u32,
}

impl InboundReferenceCounter {
    /// Create simple `InboundReferenceCounter` instance, based on `same_owner` flag provided
    pub fn new(reference_counter: u32, same_owner: bool) -> Self {
        if same_owner {
            Self {
                total: reference_counter,
                same_owner: reference_counter,
            }
        } else {
            Self {
                total: reference_counter,
                same_owner: 0,
            }
        }
    }

    /// Check if `total` is equal to zero
    pub fn is_total_equal_to_zero(self) -> bool {
        self.total == 0
    }

    /// Check if `same_owner` is equal to zero
    pub fn is_same_owner_equal_to_zero(self) -> bool {
        self.same_owner == 0
    }
}

/// Represents `Entity`, related to a specific `Class`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Entity<T: Trait> {
    /// Permissions for an instance of an Entity.
    pub entity_permissions: EntityPermissions<T>,

    /// The class id of this entity.
    pub class_id: T::ClassId,

    /// What schemas under which this entity of a class is available, think
    /// v.2.0 Person schema for John, v3.0 Person schema for John
    /// Unlikely to be more than roughly 20ish, assuming schemas for a given class eventually stableize, or that very old schema are eventually removed.
    pub supported_schemas: BTreeSet<SchemaId>, // indices of schema in corresponding class

    /// Values for properties on class that are used by some schema used by this entity!
    /// Length is no more than Class.properties.
    pub values: BTreeMap<PropertyId, PropertyValue<T>>,

    /// Number of property values referencing current entity
    pub reference_counter: InboundReferenceCounter,
}

impl<T: Trait> Default for Entity<T> {
    fn default() -> Self {
        Self {
            entity_permissions: EntityPermissions::<T>::default(),
            class_id: T::ClassId::default(),
            supported_schemas: BTreeSet::new(),
            values: BTreeMap::new(),
            reference_counter: InboundReferenceCounter::default(),
        }
    }
}

impl<T: Trait> Entity<T> {
    /// Create new `Entity` instance, related to a given `class_id` with provided parameters,  
    fn new(
        controller: EntityController<T>,
        class_id: T::ClassId,
        supported_schemas: BTreeSet<SchemaId>,
        values: BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> Self {
        Self {
            entity_permissions: EntityPermissions::<T>::default_with_controller(controller),
            class_id,
            supported_schemas,
            values,
            reference_counter: InboundReferenceCounter::default(),
        }
    }

    /// Get `Entity` values by value
    fn get_values(self) -> BTreeMap<PropertyId, PropertyValue<T>> {
        self.values
    }

    /// Get mutable `EntityPermissions` reference, related to given `Entity`
    fn get_permissions_mut(&mut self) -> &mut EntityPermissions<T> {
        &mut self.entity_permissions
    }

    /// Get `EntityPermissions` reference, related to given `Entity`
    fn get_permissions_ref(&self) -> &EntityPermissions<T> {
        &self.entity_permissions
    }

    /// Get `EntityPermissions`, related to given `Entity` by value
    fn get_permissions(self) -> EntityPermissions<T> {
        self.entity_permissions
    }

    /// Update existing `EntityPermissions` with newly provided
    pub fn update_permissions(&mut self, permissions: EntityPermissions<T>) {
        self.entity_permissions = permissions
    }

    /// Ensure `Schema` under given id is not added to given `Entity` yet
    pub fn ensure_schema_id_is_not_added(&self, schema_id: SchemaId) -> dispatch::Result {
        let schema_not_added = !self.supported_schemas.contains(&schema_id);
        ensure!(schema_not_added, ERROR_SCHEMA_ALREADY_ADDED_TO_ENTITY);
        Ok(())
    }

    /// Ensure provided `property_values` are not added to the `Entity` `values` map yet
    pub fn ensure_property_values_are_not_added(
        &self,
        property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> dispatch::Result {
        ensure!(
            property_values
                .keys()
                .all(|key| property_values.contains_key(key)),
            ERROR_ENTITY_ALREADY_CONTAINS_GIVEN_PROPERTY_ID
        );
        Ok(())
    }

    /// Ensure PropertyValue under given `in_class_schema_property_id` is Vector
    fn ensure_property_value_is_vec(
        &self,
        in_class_schema_property_id: PropertyId,
    ) -> Result<VecPropertyValue<T>, &'static str> {
        self.values
            .get(&in_class_schema_property_id)
            // Throw an error if a property was not found on entity
            // by an in-class index of a property.
            .ok_or(ERROR_UNKNOWN_ENTITY_PROP_ID)?
            .as_vec_property_value()
            .map(|property_value_vec| property_value_vec.to_owned())
            // Ensure prop value under given class schema property id is vector
            .ok_or(ERROR_PROP_VALUE_UNDER_GIVEN_INDEX_IS_NOT_A_VECTOR)
    }

    /// Ensure any `PropertyValue` from external entity does not point to the given `Entity`
    pub fn ensure_rc_is_zero(&self) -> dispatch::Result {
        ensure!(
            self.reference_counter.is_total_equal_to_zero(),
            ERROR_ENTITY_RC_DOES_NOT_EQUAL_TO_ZERO
        );
        Ok(())
    }

    /// Ensure any inbound `PropertyValue` with `same_owner` flag set points to the given `Entity`
    pub fn ensure_inbound_same_owner_rc_is_zero(&self) -> dispatch::Result {
        ensure!(
            self.reference_counter.is_same_owner_equal_to_zero(),
            ERROR_ENTITY_RC_DOES_NOT_EQUAL_TO_ZERO
        );
        Ok(())
    }

    /// Get mutable reference to the `Entity`'s `InboundReferenceCounter` instance
    pub fn get_reference_counter_mut(&mut self) -> &mut InboundReferenceCounter {
        &mut self.reference_counter
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ContentDirectory {

        /// Map, representing  CuratorGroupId -> CuratorGroup relation
        pub CuratorGroupById get(curator_group_by_id): map T::CuratorGroupId => CuratorGroup<T>;

        /// Map, representing ClassId -> Class relation
        pub ClassById get(class_by_id) config(): linked_map T::ClassId => Class<T>;

        /// Map, representing EntityId -> Entity relation
        pub EntityById get(entity_by_id) config(): map T::EntityId => Entity<T>;

        /// Next runtime storage values used to maintain next id value, used on creation of respective curator groups, classes and entities

        pub NextCuratorGroupId get(next_curator_group_id) config(): T::CuratorGroupId;

        pub NextClassId get(next_class_id) config(): T::ClassId;

        pub NextEntityId get(next_entity_id) config(): T::EntityId;

        // The voucher associated with entity creation for a given class and controller.
        // Is updated whenever an entity is created in a given class by a given controller.
        // Constraint is updated by Root, an initial value comes from `ClassPermissions::default_entity_creation_voucher_upper_bound`.
        pub EntityCreationVouchers get(entity_creation_vouchers): double_map hasher(blake2_128) T::ClassId, blake2_128(EntityController<T>) => EntityCreationVoucher<T>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        // ======
        // Next set of extrinsics can only be invoked by lead.
        // ======

        // Initializing events
        fn deposit_event() = default;

        /// Add new curator group to runtime storage
        pub fn add_curator_group(
            origin,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            //
            // == MUTATION SAFE ==
            //

            let curator_group_id = Self::next_curator_group_id();

            // Insert empty curator group with `active` parameter set to false
            <CuratorGroupById<T>>::insert(curator_group_id, CuratorGroup::<T>::default());

            // Increment the next curator curator_group_id:
            <NextCuratorGroupId<T>>::mutate(|n| *n += T::CuratorGroupId::one());

            // Trigger event
            Self::deposit_event(RawEvent::CuratorGroupAdded(curator_group_id));
            Ok(())
        }

        /// Remove curator group under given `curator_group_id` from runtime storage
        pub fn remove_curator_group(
            origin,
            curator_group_id: T::CuratorGroupId,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure CuratorGroup under given curator_group_id exists
            let curator_group = Self::ensure_curator_group_exists(&curator_group_id)?;

            // We should previously ensure that curator_group  maintains no classes to be able to remove it
            curator_group.ensure_curator_group_maintains_no_classes()?;

            //
            // == MUTATION SAFE ==
            //


            // Remove curator group under given curator group id from runtime storage
            <CuratorGroupById<T>>::remove(curator_group_id);

            // Trigger event
            Self::deposit_event(RawEvent::CuratorGroupRemoved(curator_group_id));
            Ok(())
        }

        /// Set `is_active` status for curator group under given `curator_group_id`
        pub fn set_curator_group_status(
            origin,
            curator_group_id: T::CuratorGroupId,
            is_active: bool,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure curator group under provided curator_group_id already exist
            Self::ensure_curator_group_under_given_id_exists(&curator_group_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Set `is_active` status for curator group under given `curator_group_id`
            <CuratorGroupById<T>>::mutate(curator_group_id, |curator_group| {
                curator_group.set_status(is_active)
            });

            // Trigger event
            Self::deposit_event(RawEvent::CuratorGroupStatusSet(is_active));
            Ok(())
        }

        /// Add curator to curator group under given `curator_group_id`
        pub fn add_curator_to_group(
            origin,
            curator_group_id: T::CuratorGroupId,
            curator_id: T::CuratorId,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure curator group under provided curator_group_id already exist, retrieve corresponding one
            let curator_group = Self::ensure_curator_group_exists(&curator_group_id)?;

            // Ensure max number of curators per group limit not reached yet
            curator_group.ensure_max_number_of_curators_limit_not_reached()?;

            //
            // == MUTATION SAFE ==
            //

            // Insert curator_id into curator_group under given curator_group_id
            <CuratorGroupById<T>>::mutate(curator_group_id, |curator_group| {
                curator_group.get_curators_mut().insert(curator_id);
            });

            // Trigger event
            Self::deposit_event(RawEvent::CuratorAdded(curator_group_id, curator_id));
            Ok(())
        }

        /// Remove curator from a given curator group
        pub fn remove_curator_from_group(
            origin,
            curator_group_id: T::CuratorGroupId,
            curator_id: T::CuratorId,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure curator group under provided curator_group_id already exist, retrieve corresponding one
            let curator_group = Self::ensure_curator_group_exists(&curator_group_id)?;

            // Ensure curator under provided curator_id is CuratorGroup member
            curator_group.ensure_curator_in_group_exists(&curator_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Remove curator_id from curator_group under given curator_group_id
            <CuratorGroupById<T>>::mutate(curator_group_id, |curator_group| {
                curator_group.get_curators_mut().remove(&curator_id);
            });

            // Trigger event
            Self::deposit_event(RawEvent::CuratorRemoved(curator_group_id, curator_id));
            Ok(())
        }

        /// Add curator group under given `curator_group_id` as `Class` maintainer
        pub fn add_maintainer_to_class(
            origin,
            class_id: T::ClassId,
            curator_group_id: T::CuratorGroupId,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under provided class_id exist, retrieve corresponding one
            let class = Self::ensure_known_class_id(class_id)?;

            // Ensure CuratorGroup under provided curator_group_id exist, retrieve corresponding one
            Self::ensure_curator_group_under_given_id_exists(&curator_group_id)?;

            // Ensure the max number of maintainers per Class limit not reached
            let class_permissions = class.get_permissions_ref();

            // Ensure max number of maintainers per Class constraint satisfied
            Self::ensure_maintainers_limit_not_reached(class_permissions.get_maintainers())?;

            // Ensure maintainer under provided curator_group_id is not added to the Class maintainers set yet
            class_permissions.ensure_maintainer_does_not_exist(&curator_group_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Insert `curator_group_id` into `maintainers` set, associated with given `Class`
            <ClassById<T>>::mutate(class_id, |class|
                class.get_permissions_mut().get_maintainers_mut().insert(curator_group_id)
            );

            // Increment the number of classes, curator group under given `curator_group_id` maintains
            <CuratorGroupById<T>>::mutate(curator_group_id, |curator_group| {
                curator_group.increment_number_of_classes_maintained_count();
            });

            // Trigger event
            Self::deposit_event(RawEvent::MaintainerAdded(class_id, curator_group_id));
            Ok(())
        }

        /// Remove curator group under given `curator_group_id` from `Class` maintainers set
        pub fn remove_maintainer_from_class(
            origin,
            class_id: T::ClassId,
            curator_group_id: T::CuratorGroupId,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under given id exists, return corresponding one
            let class = Self::ensure_known_class_id(class_id)?;

            // Ensure maintainer under provided curator_group_id was previously added
            // to the maintainers set, associated with corresponding Class
            class.get_permissions_ref().ensure_maintainer_exists(&curator_group_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Remove `curator_group_id` from `maintainers` set, associated with given `Class`
            <ClassById<T>>::mutate(class_id, |class|
                class.get_permissions_mut().get_maintainers_mut().remove(&curator_group_id)
            );

            // Decrement the number of classes, curator group under given `curator_group_id` maintains
            <CuratorGroupById<T>>::mutate(curator_group_id, |curator_group| {
                curator_group.decrement_number_of_classes_maintained_count();
            });

            // Trigger event
            Self::deposit_event(RawEvent::MaintainerRemoved(class_id, curator_group_id));
            Ok(())
        }

        /// Updates or creates new `EntityCreationVoucher` for given `EntityController` with individual limit
        pub fn update_entity_creation_voucher(
            origin,
            class_id: T::ClassId,
            controller: EntityController<T>,
            maximum_entities_count: T::EntityId
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under given id exists, return corresponding one
            Self::ensure_known_class_id(class_id)?;

            // Ensure maximum_entities_count does not exceed individual entities creation limit
            Self::ensure_valid_number_of_class_entities_per_actor_constraint(maximum_entities_count)?;

            // Check voucher existance
            let voucher_exists = <EntityCreationVouchers<T>>::exists(class_id, &controller);

            //
            // == MUTATION SAFE ==
            //

            if voucher_exists {

                // Set new maximum_entities_count limit for selected voucher
                let mut entity_creation_voucher = Self::entity_creation_vouchers(class_id, &controller);
                
                entity_creation_voucher.set_maximum_entities_count(maximum_entities_count);

                <EntityCreationVouchers<T>>::insert(class_id, controller.clone(), entity_creation_voucher.clone());

                // Trigger event
                Self::deposit_event(RawEvent::EntityCreationVoucherUpdated(controller, entity_creation_voucher))
            } else {
                // Create new EntityCreationVoucher instance with provided maximum_entities_count
                let entity_creation_voucher = EntityCreationVoucher::new(maximum_entities_count);

                // Add newly created `EntityCreationVoucher` into `EntityCreationVouchers` runtime storage under given `class_id`, `controller` key
                <EntityCreationVouchers<T>>::insert(class_id, controller.clone(), entity_creation_voucher.clone());

                // Trigger event
                Self::deposit_event(RawEvent::EntityCreationVoucherCreated(controller, entity_creation_voucher));
            }

            Ok(())
        }

        /// Create new `Class` with provided parameters
        pub fn create_class(
            origin,
            name: Vec<u8>,
            description: Vec<u8>,
            class_permissions: ClassPermissions<T>,
            maximum_entities_count: T::EntityId,
            default_entity_creation_voucher_upper_bound: T::EntityId
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure, that all entities creation limits, defined for a given Class, are valid
            Self::ensure_entities_creation_limits_are_valid(maximum_entities_count, default_entity_creation_voucher_upper_bound)?;

            // Ensure max number of classes limit not reached
            Self::ensure_class_limit_not_reached()?;

            // Ensure ClassNameLengthConstraint conditions satisfied
            Self::ensure_class_name_is_valid(&name)?;

            // Ensure ClassDescriptionLengthConstraint conditions satisfied
            Self::ensure_class_description_is_valid(&description)?;

            // Perform required checks to ensure classs_maintainers under provided class_permissions are valid
            let classs_maintainers = class_permissions.get_maintainers();
            Self::ensure_class_maintainers_are_valid(classs_maintainers)?;

            //
            // == MUTATION SAFE ==
            //

            // Create new Class instance from provided values
            let class = Class::new(class_permissions, name, description, maximum_entities_count, default_entity_creation_voucher_upper_bound);

            let class_id = Self::next_class_id();

            // Add new `Class` to runtime storage
            <ClassById<T>>::insert(&class_id, class);

            // Increment the next class id:
            <NextClassId<T>>::mutate(|n| *n += T::ClassId::one());

            // Trigger event
            Self::deposit_event(RawEvent::ClassCreated(class_id));
            Ok(())
        }

        /// Update `ClassPermissions` under specific `class_id`
        pub fn update_class_permissions(
            origin,
            class_id: T::ClassId,
            updated_any_member: Option<bool>,
            updated_entity_creation_blocked: Option<bool>,
            updated_all_entity_property_values_locked: Option<bool>,
            updated_maintainers: Option<BTreeSet<T::CuratorGroupId>>,
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under given id exists, return corresponding one
            let class = Self::ensure_known_class_id(class_id)?;

            // Perform required checks to ensure classs_maintainers are valid
            if let Some(ref updated_maintainers) = updated_maintainers {
                Self::ensure_class_maintainers_are_valid(updated_maintainers)?;
            }

            //
            // == MUTATION SAFE ==
            //

            let class_permissions = class.get_permissions();

            // Make updated class_permissions from parameters provided
            let updated_class_permissions = Self::make_updated_class_permissions(
                class_permissions, updated_any_member, updated_entity_creation_blocked,
                updated_all_entity_property_values_locked, updated_maintainers
            );

            // If class_permissions update has been performed
            if let Some(updated_class_permissions) = updated_class_permissions  {

                // Update `class_permissions` under given class id
                <ClassById<T>>::mutate(class_id, |class| {
                    class.update_permissions(updated_class_permissions)
                });

                // Trigger event
                Self::deposit_event(RawEvent::ClassPermissionsUpdated(class_id));
            }

            Ok(())
        }

        /// Create new class schema from existing property ids and new properties
        pub fn add_class_schema(
            origin,
            class_id: T::ClassId,
            existing_properties: BTreeSet<PropertyId>,
            new_properties: Vec<Property<T>>
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under given id exists, return corresponding one
            let class = Self::ensure_known_class_id(class_id)?;

            // Ensure Schemas limit per Class not reached
            class.ensure_schemas_limit_not_reached()?;

            // Ensure both existing and new properties for future Schema are not empty
            Self::ensure_non_empty_schema(&existing_properties, &new_properties)?;

            // Ensure max number of properties per Class limit not reached
            class.ensure_properties_limit_not_reached(&new_properties)?;

            // Complete all checks to ensure all provided new_properties are valid
            Self::ensure_all_properties_are_valid(&new_properties)?;

            // Id of next Class Schema being added
            let schema_id = class.schemas.len() as SchemaId;

            let class_properties = class.get_properties();

            // Ensure all Property names are unique within Class
            Self::ensure_all_property_names_are_unique(&class_properties, &new_properties)?;

            // Ensure existing_properties are valid indices of properties, corresponding to chosen Class
            Self::ensure_schema_properties_are_valid_indices(&existing_properties, &class_properties)?;

            //
            // == MUTATION SAFE ==
            //

            // Create `Schema` instance from existing and new property ids
            let schema = Self::create_class_schema(existing_properties, &class_properties, &new_properties);

            // Update class properties after new `Schema` added
            let updated_class_properties = Self::make_updated_class_properties(class_properties, new_properties);

            // Update Class properties and schemas
            <ClassById<T>>::mutate(class_id, |class| {
                class.properties = updated_class_properties;
                class.schemas.push(schema);
            });

            // Trigger event
            Self::deposit_event(RawEvent::ClassSchemaAdded(class_id, schema_id));

            Ok(())
        }

        /// Update `schema_status` under specific `schema_id` in `Class`
        pub fn update_class_schema_status(
            origin,
            class_id: T::ClassId,
            schema_id: SchemaId,
            schema_status: bool
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Class under given id exists, return corresponding one
            let class = Self::ensure_known_class_id(class_id)?;

            // Ensure Class already contains schema under provided schema_id
            class.ensure_schema_id_exists(schema_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Update class schema status
            <ClassById<T>>::mutate(class_id, |class| {
                class.update_schema_status(schema_id, schema_status)
            });

            // Trigger event
            Self::deposit_event(RawEvent::ClassSchemaStatusUpdated(class_id, schema_id, schema_status));
            Ok(())
        }

        /// Update entity permissions
        pub fn update_entity_permissions(
            origin,
            entity_id: T::EntityId,
            updated_frozen_for_controller: Option<bool>,
            updated_referenceable: Option<bool>
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Entity under given id exists, return corresponding one
            let entity = Self::ensure_known_entity_id(entity_id)?;

            //
            // == MUTATION SAFE ==
            //

            // Make updated entity_permissions from parameters provided
            let entity_permissions = entity.get_permissions();

            let updated_entity_permissions =
                Self::make_updated_entity_permissions(entity_permissions, updated_frozen_for_controller, updated_referenceable);

            // Update entity permissions under given entity id
            if let Some(updated_entity_permissions) = updated_entity_permissions {

                <EntityById<T>>::mutate(entity_id, |entity| {
                    entity.update_permissions(updated_entity_permissions)
                });

                // Trigger event
                Self::deposit_event(RawEvent::EntityPermissionsUpdated(entity_id));
            }
            Ok(())
        }

        /// Transfer ownership to new `EntityController` for `Entity` under given `entity_id`
        /// `new_property_value_references_with_same_owner_flag_set` should be provided manually
        pub fn transfer_entity_ownership(
            origin,
            entity_id: T::EntityId,
            new_controller: EntityController<T>,
            new_property_value_references_with_same_owner_flag_set: BTreeMap<PropertyId, PropertyValue<T>>
        ) -> dispatch::Result {

            // Ensure given origin is lead
            ensure_is_lead::<T>(origin)?;

            // Ensure Entity under given entity_id exists, retrieve corresponding Entity & Class
            let (entity, class) = Self::ensure_known_entity_and_class(entity_id)?;

            // Ensure provided new_entity_controller is not equal to current one
            entity.get_permissions_ref().ensure_controllers_are_not_equal(&new_controller)?;

            // Ensure any inbound PropertyValue::Reference with same_owner flag set points to the given Entity
            entity.ensure_inbound_same_owner_rc_is_zero()?;

            let class_properties = class.properties;

            let entity_property_values = entity.values;

            // Create wrapper structure from provided entity_property_values and their corresponding Class properties
            let values_for_existing_properties = ValuesForExistingProperties::from(&class_properties, &entity_property_values);

            // Filter provided values_for_existing_properties, leaving only `Reference`'s with `SameOwner` flag set
            // Retrieve the set of corresponding property ids
            let entity_property_id_references_with_same_owner_flag_set =
                Self::get_property_id_references_with_same_owner_flag_set(values_for_existing_properties);

            // Ensure provided `new_property_value_references_with_same_owner_flag_set` are valid references with `SameOwner` flag set
            Self::ensure_only_references_with_same_owner_flag_set_provided(
                &entity_property_id_references_with_same_owner_flag_set,
                &new_property_value_references_with_same_owner_flag_set
            )?;

            let unused_schema_property_ids = Self::compute_unused_property_ids(
                &new_property_value_references_with_same_owner_flag_set, &entity_property_id_references_with_same_owner_flag_set
            );

            // Perform all checks to ensure all required property_values under provided unused_schema_property_ids provided
            Self::ensure_all_required_properties_provided(&class_properties, unused_schema_property_ids)?;

            // Create wrapper structure from provided new_property_value_references_with_same_owner_flag_set and their corresponding Class properties
            let values_for_existing_properties = ValuesForExistingProperties::from(
                &class_properties, &new_property_value_references_with_same_owner_flag_set
            );

            // Validate all values, provided in values_for_existing_properties,
            // against the type of its Property and check any additional constraints
            Self::ensure_property_values_are_valid(&new_controller, &values_for_existing_properties)?;

            // Make updated entity_property_values from parameters provided
            let entity_property_values_updated =
                if let Some(entity_property_values_updated) =
                    Self::make_updated_property_value_references_with_same_owner_flag_set(
                        entity_property_id_references_with_same_owner_flag_set, &entity_property_values,
                        &new_property_value_references_with_same_owner_flag_set,
                    ) {

                        // Create wrapper structure from provided entity_property_values_updated and their corresponding Class properties
                        let updated_values_for_existing_properties = ValuesForExistingProperties::from(
                            &class_properties, &entity_property_values_updated
                        );

                        // Traverse all updated_values_for_existing_properties to ensure unique property satisfied (if required)
                        Self::ensure_property_values_unique_option_satisfied(updated_values_for_existing_properties)?;
                        Some(entity_property_values_updated)
                    } else {
                        None
                    };

            //
            // == MUTATION SAFE ==
            //

            // Transfer entity ownership
            if let Some(entity_property_values_updated) = entity_property_values_updated {

                // Calculate entities reference counter side effects for current operation
                let entities_inbound_rcs_delta =
                    Self::get_updated_inbound_rcs_delta(
                        class_properties, entity_property_values, &new_property_value_references_with_same_owner_flag_set
                    );

                // Update InboundReferenceCounter, based on previously calculated ReferenceCounterSideEffects, for each Entity involved
                entities_inbound_rcs_delta.update_entities_rcs();

                <EntityById<T>>::mutate(entity_id, |entity| {

                    // Update current Entity property values with updated ones
                    entity.values = entity_property_values_updated;

                    // Set up new controller for the current Entity instance
                    entity.get_permissions_mut().set_conroller(new_controller.clone());
                });
            } else {
                // Set up new controller for the current Entity instance
                <EntityById<T>>::mutate(entity_id, |entity| {
                    entity.get_permissions_mut().set_conroller(new_controller.clone());
                });
            };

            // Trigger event
            Self::deposit_event(RawEvent::EntityOwnershipTransfered(entity_id, new_controller));

            Ok(())
        }

        // ======
        // The next set of extrinsics can be invoked by anyone who can properly sign for provided value of `Actor<T>`.
        // ======

        /// Create an entity.
        /// If someone is making an entity of this class for first time,
        /// then a voucher is also added with the class limit as the default limit value.
        pub fn create_entity(
            origin,
            class_id: T::ClassId,
            actor: Actor<T>,
        ) -> dispatch::Result {

            let account_id = ensure_signed(origin)?;

            // Ensure Class under given id exists, return corresponding one
            let class = Self::ensure_class_exists(class_id)?;

            // Ensure maximum entities limit per class not reached
            class.ensure_maximum_entities_count_limit_not_reached()?;

            let class_permissions = class.get_permissions_ref();

            // Ensure entities creation is not blocked on Class level
            class_permissions.ensure_entity_creation_not_blocked()?;

            // Ensure actor can create entities
            class_permissions.ensure_can_create_entities(&account_id, &actor)?;

            let entity_controller = EntityController::from_actor(&actor);

            // Check if entity creation voucher exists
            let voucher_exists = if <EntityCreationVouchers<T>>::exists(class_id, &entity_controller) {

                // Ensure voucher limit not reached
                Self::entity_creation_vouchers(class_id, &entity_controller).ensure_voucher_limit_not_reached()?;
                true
            } else {
                false
            };

            //
            // == MUTATION SAFE ==
            //

            // Create voucher, update if exists

            if voucher_exists {

                // Increment number of created entities count, if specified voucher already exist
                <EntityCreationVouchers<T>>::mutate(class_id, &entity_controller, |entity_creation_voucher| {
                    entity_creation_voucher.increment_created_entities_count()
                });
            } else {

                // Create new voucher for given entity creator with default limit
                let mut entity_creation_voucher = EntityCreationVoucher::new(class.get_default_entity_creation_voucher_upper_bound());

                // Increase created entities count by 1 to maintain valid entity_creation_voucher state after following Entity added
                entity_creation_voucher.increment_created_entities_count();
                <EntityCreationVouchers<T>>::insert(class_id, entity_controller.clone(), entity_creation_voucher);
            }

            // Create new entity

            let entity_id = Self::next_entity_id();

            let new_entity = Entity::<T>::new(
                entity_controller,
                class_id,
                BTreeSet::new(),
                BTreeMap::new(),
            );

            // Save newly created entity:
            EntityById::insert(entity_id, new_entity);

            // Increment the next entity id:
            <NextEntityId<T>>::mutate(|n| *n += T::EntityId::one());

            // Increment number of entities, associated with this class
            <ClassById<T>>::mutate(class_id, |class| {
                class.increment_entities_count();
            });

            // Trigger event
            Self::deposit_event(RawEvent::EntityCreated(actor, entity_id));
            Ok(())
        }

        /// Remove `Entity` under provided `entity_id`
        pub fn remove_entity(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
        ) -> dispatch::Result {

            // Retrieve Entity and EntityAccessLevel for the actor, attemting to perform operation
            let (_, entity, access_level) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure actor with given EntityAccessLevel can remove entity
            EntityPermissions::<T>::ensure_group_can_remove_entity(access_level)?;

            // Ensure any inbound PropertyValue::Reference points to the given Entity
            entity.ensure_rc_is_zero()?;

            //
            // == MUTATION SAFE ==
            //

            // Remove entity
            <EntityById<T>>::remove(entity_id);

            // Decrement class entities counter
            <ClassById<T>>::mutate(entity.class_id, |class| class.decrement_entities_count());

            let entity_controller = EntityController::<T>::from_actor(&actor);

            // Decrement entity_creation_voucher after entity removal perfomed
            <EntityCreationVouchers<T>>::mutate(entity.class_id, entity_controller, |entity_creation_voucher| {
                entity_creation_voucher.decrement_created_entities_count();
            });

            // Trigger event
            Self::deposit_event(RawEvent::EntityRemoved(actor, entity_id));
            Ok(())
        }

        /// Add schema support to entity under given `shema_id` and provided `property_values`
        pub fn add_schema_support_to_entity(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
            schema_id: SchemaId,
            property_values: BTreeMap<PropertyId, PropertyValue<T>>
        ) -> dispatch::Result {

            // Retrieve Class, Entity and ensure given have access to the Entity under given entity_id
            let (class, entity, _) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure Schema under given id is not added to given Entity yet
            entity.ensure_schema_id_is_not_added(schema_id)?;

            // Ensure provided property_values are not added to the Entity values map yet
            entity.ensure_property_values_are_not_added(&property_values)?;

            // Ensure Class Schema under given index exist, return corresponding Schema
            let schema = class.ensure_schema_exists(schema_id)?.to_owned();

            // Ensure provided schema can be added to the Entity
            schema.ensure_is_active()?;

            // Ensure all provided property values are for properties in the given schema
            schema.ensure_has_properties(&property_values)?;

            // Retrieve Schema property ids, which are not provided in property_values
            let unused_schema_property_ids = Self::compute_unused_property_ids(&property_values, schema.get_properties());
            let class_properties = class.properties;

            // Perform all checks to ensure all required property_values under provided unused_schema_property_ids provided
            Self::ensure_all_required_properties_provided(&class_properties, unused_schema_property_ids)?;

            // Ensure all property_values under given Schema property ids are valid
            let entity_controller = entity.get_permissions_ref().get_controller();

            // Create wrapper structure from provided property_values and their corresponding Class properties
            let values_for_existing_properties = ValuesForExistingProperties::from(&class_properties, &property_values);

            // Validate all values, provided in values_for_existing_properties,
            // against the type of its Property and check any additional constraints
            Self::ensure_property_values_are_valid(&entity_controller, &values_for_existing_properties)?;

            // Calculate entities reference counter side effects for current operation
            let entities_inbound_rcs_delta = Self::calculate_entities_inbound_rcs_delta(
                values_for_existing_properties, DeltaMode::Increment
            );

            // Update InboundReferenceCounter, based on previously calculated ReferenceCounterSideEffects, for each Entity involved
            entities_inbound_rcs_delta.update_entities_rcs();

            let entity_property_values = entity.get_values();

            // Compute updated entity values, after new schema support added
            let entity_values_updated = Self::make_updated_entity_property_values(
                schema, entity_property_values, property_values
            );

            // Create wrapper structure from updated entity values and their corresponding Class properties
            let updated_values_for_existing_properties = ValuesForExistingProperties::from(
                &class_properties, &entity_values_updated
            );

            // Traverse all updated_values_for_existing_properties to ensure unique property satisfied (if required)
            Self::ensure_property_values_unique_option_satisfied(
                updated_values_for_existing_properties
            )?;


            //
            // == MUTATION SAFE ==
            //

            // Add schema support to `Entity` under given `entity_id`
            <EntityById<T>>::mutate(entity_id, |entity| {

                // Add a new schema to the list of schemas supported by this entity.
                entity.supported_schemas.insert(schema_id);

                // Update entity values only if new properties have been added.
                if entity_values_updated.len() > entity.values.len() {
                    entity.values = entity_values_updated;
                }
            });

            // Trigger event
            Self::deposit_event(RawEvent::EntitySchemaSupportAdded(actor, entity_id, schema_id));
            Ok(())
        }

        /// Update `Entity` `PropertyValue`'s with provided ones
        pub fn update_entity_property_values(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
            new_property_values: BTreeMap<PropertyId, PropertyValue<T>>
        ) -> dispatch::Result {

            // Retrieve Class, Entity and EntityAccessLevel for the actor, attemting to perform operation
            let (class, entity, access_level) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure property values were not locked on Class level
            class.ensure_property_values_unlocked()?;

            // Filter new_property_values, that are identical to entity_property_values.
            let new_property_values = Self::try_filter_identical_property_values(&entity.values, new_property_values);

            // Ensure all provided new_property_values are already added to the current Entity instance
            Self::ensure_all_property_values_are_already_added(&entity.values, &new_property_values)?;

            let class_properties = class.properties;

            // Create wrapper structure from new_property_values and their corresponding Class properties
            let new_values_for_existing_properties = ValuesForExistingProperties::from(&class_properties, &new_property_values);

            // Ensure all provided property values are unlocked for the actor with given access_level
            Self::ensure_all_property_values_are_unlocked_from(&new_values_for_existing_properties, access_level)?;

            let entity_controller = entity.get_permissions_ref().get_controller();

            // Validate all values, provided in values_for_existing_properties,
            // against the type of its Property and check any additional constraints
            Self::ensure_property_values_are_valid(&entity_controller, &new_values_for_existing_properties)?;

            // Get current property values of an entity,
            // so we can update them if new values provided present in new_property_values.

            let entity_property_values = entity.values;

            // Make updated entity_property_values from current entity_property_values and new_property_values
            let entity_property_values_updated = if let Some(entity_property_values_updated) =
                Self::make_updated_values(&entity_property_values, &new_property_values) {

                    // Create wrapper structure from new_property_values and their corresponding Class properties
                    let updated_values_for_existing_properties = ValuesForExistingProperties::from(
                        &class_properties, &new_property_values
                    );

                    // Traverse all values_for_updated_properties to ensure unique property satisfied (if required)
                    Self::ensure_property_values_unique_option_satisfied(updated_values_for_existing_properties)?;
                    Some(entity_property_values_updated)
                } else {
                    None
                };

            //
            // == MUTATION SAFE ==
            //

            // If property values should be updated
            if let Some(entity_property_values_updated) = entity_property_values_updated {
                // Update entity property values
                <EntityById<T>>::mutate(entity_id, |entity| {
                    entity.values = entity_property_values_updated;
                });

                // Calculate entities reference counter side effects for current operation
                let entities_inbound_rcs_delta =
                    Self::get_updated_inbound_rcs_delta(class_properties, entity_property_values, &new_property_values);

                // Update InboundReferenceCounter, based on previously calculated ReferenceCounterSideEffects, for each Entity involved
                entities_inbound_rcs_delta.update_entities_rcs();

                // Trigger event
                Self::deposit_event(RawEvent::EntityPropertyValuesUpdated(actor, entity_id));
            }

            Ok(())
        }

        /// Clear `PropertyValueVec` under given `entity_id` & `in_class_schema_property_id`
        pub fn clear_entity_property_vector(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
            in_class_schema_property_id: PropertyId
        ) -> dispatch::Result {

            // Retrieve Class, Entity and EntityAccessLevel for the actor, attemting to perform operation
            let (class, entity, access_level) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure PropertyValue under given in_class_schema_property_id is Vector
            let property_value_vector =
                entity.ensure_property_value_is_vec(in_class_schema_property_id)?;

            // Ensure Property under given PropertyId is unlocked from actor with given EntityAccessLevel
            // Retrieve corresponding Property by value
            let property = class.ensure_class_property_type_unlocked_from(
                in_class_schema_property_id,
                access_level,
            )?;

            // Decrease reference counters of involved entities (if some)
            if let Some(entity_ids_to_decrease_rcs) = property_value_vector.get_vec_value().get_involved_entities() {

                // Calculate `ReferenceCounterSideEffects`, based on entity_ids involved, same_controller_status and chosen `DeltaMode`
                let same_controller_status = property.property_type.same_controller_status();
                let entities_inbound_rcs_delta = Self::perform_entities_inbound_rcs_delta_calculation(
                    ReferenceCounterSideEffects::<T>::default(), entity_ids_to_decrease_rcs,
                    same_controller_status, DeltaMode::Decrement
                );

                // Update InboundReferenceCounter, based on previously calculated ReferenceCounterSideEffects, for each Entity involved
                entities_inbound_rcs_delta.update_entities_rcs();
            }

            // Clear property_value_vector.
            let empty_property_value_vector = Self::clear_property_vector(property_value_vector);

            // Insert empty_property_value_vector into entity_property_values mapping at in_class_schema_property_id.
            // Retrieve updated entity_property_values
            let entity_values_updated = Self::insert_at_in_class_schema_property_id(
                entity.values, in_class_schema_property_id, empty_property_value_vector
            );

            //
            // == MUTATION SAFE ==
            //

            // Update entity property values
            <EntityById<T>>::mutate(entity_id, |entity| {
                entity.values = entity_values_updated
            });

            // Trigger event
            Self::deposit_event(RawEvent::EntityPropertyValueVectorCleared(actor, entity_id, in_class_schema_property_id));

            Ok(())
        }

        /// Remove value at given `index_in_property_vector`
        /// from `PropertyValueVec` under in_class_schema_property_id
        pub fn remove_at_entity_property_vector(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
            in_class_schema_property_id: PropertyId,
            index_in_property_vector: VecMaxLength,
            nonce: T::Nonce
        ) -> dispatch::Result {

            // Retrieve Class, Entity and EntityAccessLevel for the actor, attemting to perform operation
            let (class, entity, access_level) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure PropertyValue under given in_class_schema_property_id is Vector
            let property_value_vector =
                entity.ensure_property_value_is_vec(in_class_schema_property_id)?;

            // Ensure Property under given PropertyId is unlocked from actor with given EntityAccessLevel
            // Retrieve corresponding Property by value
            let property = class.ensure_class_property_type_unlocked_from(
                in_class_schema_property_id,
                access_level,
            )?;

            // Ensure `VecPropertyValue` nonce is equal to the provided one.
            // Used to to avoid possible data races, when performing vector specific operations
            property_value_vector.ensure_nonce_equality(nonce)?;

            // Ensure, provided index_in_property_vec is valid index of VecValue
            property_value_vector
                .ensure_index_in_property_vector_is_valid(index_in_property_vector)?;

            //
            // == MUTATION SAFE ==
            //

            // Decrease reference counter of involved entity (if some)
            let involved_entity_id = property_value_vector
                .get_vec_value()
                .get_involved_entities()
                .map(|involved_entities| involved_entities[index_in_property_vector as usize]);

            if let Some(involved_entity_id) = involved_entity_id {
                let same_controller_status = property.property_type.same_controller_status();
                let rc_delta = EntityReferenceCounterSideEffect::one(same_controller_status, DeltaMode::Decrement);

                // Update InboundReferenceCounter of involved entity, based on previously calculated ReferenceCounterSideEffect
                Self::update_entity_rc(involved_entity_id, rc_delta);
            }

            // Remove value at in_class_schema_property_id in property value vector
            let property_value_vector_updated = Self::remove_at_index_in_property_vector(
                property_value_vector, index_in_property_vector
            );

            // Insert updated propery value into entity_property_values mapping at in_class_schema_property_id.
            let entity_values_updated = Self::insert_at_in_class_schema_property_id(
                entity.values, index_in_property_vector, property_value_vector_updated
            );

            // Update entity property values
            <EntityById<T>>::mutate(entity_id, |entity| {
                entity.values = entity_values_updated;
            });

            // Trigger event
            Self::deposit_event(
                RawEvent::RemovedAtEntityPropertyValueVectorIndex(
                    actor, entity_id, in_class_schema_property_id, index_in_property_vector, nonce
                )
            );

            Ok(())
        }

        /// Insert `SinglePropertyValue` at given `index_in_property_vector`
        /// into `PropertyValueVec` under `in_class_schema_property_id`
        pub fn insert_at_entity_property_vector(
            origin,
            actor: Actor<T>,
            entity_id: T::EntityId,
            in_class_schema_property_id: PropertyId,
            index_in_property_vector: VecMaxLength,
            property_value: SinglePropertyValue<T>,
            nonce: T::Nonce
        ) -> dispatch::Result {

            // Retrieve Class, Entity and EntityAccessLevel for the actor, attemting to perform operation
            let (class, entity, access_level) = Self::ensure_class_entity_and_access_level(origin, entity_id, &actor)?;

            // Ensure PropertyValue under given in_class_schema_property_id is Vector
            let property_value_vector =
                entity.ensure_property_value_is_vec(in_class_schema_property_id)?;

            // Ensure `VecPropertyValue` nonce is equal to the provided one.
            // Used to to avoid possible data races, when performing vector specific operations
            property_value_vector.ensure_nonce_equality(nonce)?;

            let entity_controller = entity.get_permissions_ref().get_controller();

            // Ensure Property under given PropertyId is unlocked from actor with given EntityAccessLevel
            // Retrieve corresponding Property by value
            let class_property = class.ensure_class_property_type_unlocked_from(
                in_class_schema_property_id,
                access_level,
            )?;

            // Ensure property_value type is equal to the property_value_vector type and check all constraints
            class_property.ensure_property_value_can_be_inserted_at_property_vector(
                &property_value,
                &property_value_vector,
                index_in_property_vector,
                entity_controller,
            )?;

            //
            // == MUTATION SAFE ==
            //

            let value = property_value.get_value();

            // Increase reference counter of involved entity (if some)
            if let Some(entity_rc_to_increment) = value.get_involved_entity() {
                let same_controller_status = class_property.property_type.same_controller_status();
                let rc_delta = EntityReferenceCounterSideEffect::one(same_controller_status, DeltaMode::Increment);

                // Update InboundReferenceCounter of involved entity, based on previously calculated ReferenceCounterSideEffect
                Self::update_entity_rc(entity_rc_to_increment, rc_delta);
            }

            // Insert SinglePropertyValue at in_class_schema_property_id into property value vector
            let property_value_updated = Self::insert_at_index_in_property_vector(
                property_value_vector, index_in_property_vector, value
            );

            // Insert updated propery value into entity_property_values mapping at in_class_schema_property_id.
            // Retrieve updated entity_property_values
            let entity_values_updated = Self::insert_at_in_class_schema_property_id(
                entity.values, in_class_schema_property_id, property_value_updated
            );

            // Update entity property values
            <EntityById<T>>::mutate(entity_id, |entity| {
                entity.values = entity_values_updated
            });

            // Trigger event
            Self::deposit_event(
                RawEvent::InsertedAtEntityPropertyValueVectorIndex(
                    actor, entity_id, in_class_schema_property_id, index_in_property_vector, nonce
                )
            );

            Ok(())
        }

        pub fn transaction(origin, actor: Actor<T>, operations: Vec<OperationType<T>>) -> dispatch::Result {

            // Ensure maximum number of operations during atomic batching limit not reached
            Self::ensure_number_of_operations_during_atomic_batching_limit_not_reached(&operations)?;

            //
            // == MUTATION SAFE ==
            //

            // This Vec holds the T::EntityId of the entity created as a result of executing a `CreateEntity` `Operation`
            let mut entity_created_in_operation = vec![];

            // Create raw origin
            let raw_origin = origin.into().map_err(|_| ERROR_ORIGIN_CANNOT_BE_MADE_INTO_RAW_ORIGIN)?;

            for operation_type in operations.into_iter() {
                let origin = T::Origin::from(raw_origin.clone());
                let actor = actor.clone();
                match operation_type {
                    OperationType::CreateEntity(create_entity_operation) => {
                        Self::create_entity(origin, create_entity_operation.class_id, actor)?;

                        // entity id of newly created entity
                        let entity_id = Self::next_entity_id() - T::EntityId::one();
                        entity_created_in_operation.push(entity_id);
                    },
                    OperationType::UpdatePropertyValues(update_property_values_operation) => {
                        let entity_id = operations::parametrized_entity_to_entity_id(
                            &entity_created_in_operation, update_property_values_operation.entity_id
                        )?;
                        let property_values = operations::parametrized_property_values_to_property_values(
                            &entity_created_in_operation, update_property_values_operation.new_parametrized_property_values
                        )?;
                        Self::update_entity_property_values(origin, actor, entity_id, property_values)?;
                    },
                    OperationType::AddSchemaSupportToEntity(add_schema_support_to_entity_operation) => {
                        let entity_id = operations::parametrized_entity_to_entity_id(
                            &entity_created_in_operation, add_schema_support_to_entity_operation.entity_id
                        )?;
                        let schema_id = add_schema_support_to_entity_operation.schema_id;
                        let property_values = operations::parametrized_property_values_to_property_values(
                            &entity_created_in_operation, add_schema_support_to_entity_operation.parametrized_property_values
                        )?;
                        Self::add_schema_support_to_entity(origin, actor, entity_id, schema_id, property_values)?;
                    }
                }
            }

            // Trigger event
            Self::deposit_event(RawEvent::TransactionCompleted(actor));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Updates corresponding `Entity` `reference_counter` by `reference_counter_delta`.
    fn update_entity_rc(
        entity_id: T::EntityId,
        reference_counter_delta: EntityReferenceCounterSideEffect,
    ) {
        // Update both total and same owner number of inbound references for the Entity instance under given entity_id
        <EntityById<T>>::mutate(entity_id, |entity| {
            let entity_inbound_rc = entity.get_reference_counter_mut();
            entity_inbound_rc.total =
                (entity_inbound_rc.total as i32 + reference_counter_delta.total) as u32;
            entity_inbound_rc.same_owner =
                (entity_inbound_rc.same_owner as i32 + reference_counter_delta.same_owner) as u32;
        })
    }

    /// Returns the stored `Class` if exist, error otherwise.
    fn ensure_class_exists(class_id: T::ClassId) -> Result<Class<T>, &'static str> {
        ensure!(<ClassById<T>>::exists(class_id), ERROR_CLASS_NOT_FOUND);
        Ok(Self::class_by_id(class_id))
    }

    /// Update `entity_property_values` with `property_values`
    /// Return updated `entity_property_values`
    fn make_updated_entity_property_values(
        schema: Schema,
        entity_property_values: BTreeMap<PropertyId, PropertyValue<T>>,
        property_values: BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> BTreeMap<PropertyId, PropertyValue<T>> {
        // Concatenate existing `entity_property_values` with `property_values`, provided, when adding `Schema` support.
        let updated_entity_property_values: BTreeMap<PropertyId, PropertyValue<T>> =
            entity_property_values
                .into_iter()
                .chain(property_values.into_iter())
                .collect();

        // Write all missing non required `Schema` `property_values` as PropertyValue::default()
        let non_required_property_values: BTreeMap<PropertyId, PropertyValue<T>> = schema
            .get_properties()
            .iter()
            .filter_map(|property_id| {
                if !updated_entity_property_values.contains_key(property_id) {
                    Some((*property_id, PropertyValue::default()))
                } else {
                    None
                }
            })
            .collect();

        // Extend updated_entity_property_values with given Schema non_required_property_values
        updated_entity_property_values
            .into_iter()
            .chain(non_required_property_values.into_iter())
            .collect()
    }

    /// Update `ReferenceCounterSideEffects`, based on `involved_entity_ids`, `same_controller_status` provided and chosen `DeltaMode`
    fn perform_entities_inbound_rcs_delta_calculation(
        mut inbound_rcs_delta: ReferenceCounterSideEffects<T>,
        involved_entity_ids: Vec<T::EntityId>,
        same_controller_status: bool,
        delta_mode: DeltaMode,
    ) -> ReferenceCounterSideEffects<T> {
        for involved_entity_id in involved_entity_ids {
            // If inbound_rcs_delta already contains entry for the given involved_entity_id, increment it
            // with atomic EntityReferenceCounterSideEffect instance, based on same_owner flag provided and DeltaMode,
            // otherwise create new atomic EntityReferenceCounterSideEffect instance
            *inbound_rcs_delta
                .entry(involved_entity_id)
                .or_insert_with(|| {
                    EntityReferenceCounterSideEffect::one(same_controller_status, delta_mode)
                }) += EntityReferenceCounterSideEffect::one(same_controller_status, delta_mode);
        }
        inbound_rcs_delta
    }

    /// Calculate `ReferenceCounterSideEffects`, based on values provided and chosen `DeltaMode`
    fn calculate_entities_inbound_rcs_delta(
        values_for_existing_properties: ValuesForExistingProperties<T>,
        delta_mode: DeltaMode,
    ) -> ReferenceCounterSideEffects<T> {
        values_for_existing_properties
            .values()
            .map(|value_for_existing_property| value_for_existing_property.unzip())
            .filter_map(|(property, value)| {
                value.get_involved_entities().map(|involved_entity_ids| {
                    (
                        involved_entity_ids,
                        property.property_type.same_controller_status(),
                    )
                })
            })
            .fold(
                ReferenceCounterSideEffects::default(),
                |inbound_rcs_delta, (involved_entity_ids, same_controller_status)| {
                    Self::perform_entities_inbound_rcs_delta_calculation(
                        inbound_rcs_delta,
                        involved_entity_ids,
                        same_controller_status,
                        delta_mode,
                    )
                },
            )
    }

    /// Get `ReferenceCounterSideEffects`, based on entities involved into update process
    pub fn get_updated_inbound_rcs_delta(
        class_properties: Vec<Property<T>>,
        entity_property_values: BTreeMap<PropertyId, PropertyValue<T>>,
        new_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> ReferenceCounterSideEffects<T> {
        // Entities, which rcs should be updated
        let entity_property_values_to_update: BTreeMap<PropertyId, PropertyValue<T>> =
            entity_property_values
                .into_iter()
                .filter(|(entity_id, _)| new_property_values.contains_key(entity_id))
                .collect();

        // Calculate entities reference counter side effects for current operation
        Self::calculate_entities_inbound_rcs_delta(
            ValuesForExistingProperties::from(&class_properties, &entity_property_values_to_update),
            DeltaMode::Decrement,
        )
        .update(Self::calculate_entities_inbound_rcs_delta(
            ValuesForExistingProperties::from(&class_properties, new_property_values),
            DeltaMode::Increment,
        ))
    }

    /// Used to update `class_permissions` with parameters provided.
    /// Returns `Some(ClassPermissions<T>)` if update performed and `None` otherwise
    pub fn make_updated_class_permissions(
        class_permissions: ClassPermissions<T>,
        updated_any_member: Option<bool>,
        updated_entity_creation_blocked: Option<bool>,
        updated_all_entity_property_values_locked: Option<bool>,
        updated_maintainers: Option<BTreeSet<T::CuratorGroupId>>,
    ) -> Option<ClassPermissions<T>> {
        // Used to check if update performed
        let mut updated_class_permissions = class_permissions.clone();

        if let Some(updated_any_member) = updated_any_member {
            updated_class_permissions.set_any_member_status(updated_any_member);
        }

        if let Some(updated_entity_creation_blocked) = updated_entity_creation_blocked {
            updated_class_permissions.set_entity_creation_blocked(updated_entity_creation_blocked);
        }

        if let Some(updated_all_entity_property_values_locked) =
            updated_all_entity_property_values_locked
        {
            updated_class_permissions
                .set_all_entity_property_values_locked(updated_all_entity_property_values_locked);
        }

        if let Some(updated_maintainers) = updated_maintainers {
            updated_class_permissions.set_maintainers(updated_maintainers);
        }

        if updated_class_permissions != class_permissions {
            Some(updated_class_permissions)
        } else {
            None
        }
    }

    /// Used to update `entity_permissions` with parameters provided.
    /// Returns `Some(EntityPermissions<T>)` if update performed and `None` otherwise
    pub fn make_updated_entity_permissions(
        entity_permissions: EntityPermissions<T>,
        updated_frozen_for_controller: Option<bool>,
        updated_referenceable: Option<bool>,
    ) -> Option<EntityPermissions<T>> {
        // Used to check if update performed
        let mut updated_entity_permissions = entity_permissions.clone();

        if let Some(updated_frozen_for_controller) = updated_frozen_for_controller {
            updated_entity_permissions.set_frozen(updated_frozen_for_controller);
        }

        if let Some(updated_referenceable) = updated_referenceable {
            updated_entity_permissions.set_referencable(updated_referenceable);
        }

        if updated_entity_permissions != entity_permissions {
            Some(updated_entity_permissions)
        } else {
            None
        }
    }

    /// Returns `Class` and `Entity` under given id, if exists, and `EntityAccessLevel` corresponding to `origin`, if permitted
    fn ensure_class_entity_and_access_level(
        origin: T::Origin,
        entity_id: T::EntityId,
        actor: &Actor<T>,
    ) -> Result<(Class<T>, Entity<T>, EntityAccessLevel), &'static str> {
        let account_id = ensure_signed(origin)?;

        // Ensure Entity under given id exists, retrieve corresponding one
        let entity = Self::ensure_known_entity_id(entity_id)?;

        // Ensure Class under given id exists, retrieve corresponding one
        let class = Self::class_by_id(entity.class_id);

        // Derive EntityAccessLevel for the actor, attempting to act.
        let access_level = EntityAccessLevel::derive(
            &account_id,
            entity.get_permissions_ref(),
            class.get_permissions_ref(),
            actor,
        )?;

        Ok((class, entity, access_level))
    }

    /// Ensure `Entity` under given `entity_id` exists, retrieve corresponding `Entity` & `Class`
    pub fn ensure_known_entity_and_class(
        entity_id: T::EntityId,
    ) -> Result<(Entity<T>, Class<T>), &'static str> {
        // Ensure Entity under given id exists, retrieve corresponding one
        let entity = Self::ensure_known_entity_id(entity_id)?;

        let class = ClassById::get(entity.class_id);
        Ok((entity, class))
    }

    /// Filter `provided values_for_existing_properties`, leaving only `Reference`'s with `SameOwner` flag set
    /// Returns the set of corresponding property ids
    pub fn get_property_id_references_with_same_owner_flag_set(
        values_for_existing_properties: ValuesForExistingProperties<T>,
    ) -> BTreeSet<PropertyId> {
        values_for_existing_properties
            // Iterate over the PropertyId's
            .keys()
            // Filter provided values_for_existing_properties, leaving only `Reference`'s with `SameOwner` flag set
            .filter(|&property_id| {
                values_for_existing_properties[property_id]
                    .get_property()
                    .property_type
                    .same_controller_status()
            })
            .copied()
            .collect()
    }

    /// Ensure all provided `new_property_value_references_with_same_owner_flag_set` are references with `SameOwner` flag set
    pub fn ensure_only_references_with_same_owner_flag_set_provided(
        entity_property_id_references_with_same_owner_flag_set: &BTreeSet<PropertyId>,
        new_property_value_references_with_same_owner_flag_set: &BTreeMap<
            PropertyId,
            PropertyValue<T>,
        >,
    ) -> dispatch::Result {
        let new_property_value_id_references_with_same_owner_flag_set: BTreeSet<PropertyId> =
            new_property_value_references_with_same_owner_flag_set
                .keys()
                .copied()
                .collect();

        ensure!(
            new_property_value_id_references_with_same_owner_flag_set
                .is_subset(entity_property_id_references_with_same_owner_flag_set),
            ERROR_ALL_PROVIDED_PROPERTY_VALUES_MUST_BE_REFERENCES_WITH_SAME_OWNER_FLAG_SET
        );
        Ok(())
    }

    /// Used to update entity_property_values with parameters provided.
    /// Returns Some(BTreeMap<PropertyId, PropertyValue<T>>) if update performed and None otherwise
    pub fn make_updated_property_value_references_with_same_owner_flag_set(
        entity_property_id_references_with_same_owner_flag_set: BTreeSet<PropertyId>,
        entity_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
        new_property_value_references_with_same_owner_flag_set: &BTreeMap<
            PropertyId,
            PropertyValue<T>,
        >,
    ) -> Option<BTreeMap<PropertyId, PropertyValue<T>>> {
        // Used to check if update performed
        let mut entity_property_values_updated = entity_property_values.clone();

        for entity_property_id_reference in entity_property_id_references_with_same_owner_flag_set {
            // If new_property_value_reference_with_same_owner_flag_set under provided entity_property_id_reference exists,
            if let Some(new_property_value_reference_with_same_owner_flag_set) =
                new_property_value_references_with_same_owner_flag_set
                    .get(&entity_property_id_reference)
            {
                // Update entity_property_values with new_property_value_reference_with_same_owner_flag_set
                entity_property_values_updated.insert(
                    entity_property_id_reference,
                    new_property_value_reference_with_same_owner_flag_set.to_owned(),
                );
            } else {
                // Throw away old non required property value references with same owner flag set
                // and replace them with Default ones
                entity_property_values_updated
                    .insert(entity_property_id_reference, PropertyValue::default());
            }
        }

        if *entity_property_values != entity_property_values_updated {
            Some(entity_property_values_updated)
        } else {
            None
        }
    }

    /// Compute property ids, that are not in `property_values`
    pub fn compute_unused_property_ids(
        property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
        property_ids: &BTreeSet<PropertyId>,
    ) -> BTreeSet<PropertyId> {
        let property_value_indices: BTreeSet<PropertyId> =
            property_values.keys().cloned().collect();

        property_ids
            .difference(&property_value_indices)
            .copied()
            .collect()
    }

    /// Perform all checks to ensure all required `property_values` under provided `unused_schema_property_ids` provided
    pub fn ensure_all_required_properties_provided(
        class_properties: &[Property<T>],
        unused_schema_property_ids: BTreeSet<PropertyId>,
    ) -> dispatch::Result {
        for unused_schema_property_id in unused_schema_property_ids {
            // Indexing is safe, Class should always maintain such constistency
            let class_property = &class_properties[unused_schema_property_id as usize];

            // All required property values should be provided
            ensure!(!class_property.required, ERROR_MISSING_REQUIRED_PROP);
        }
        Ok(())
    }

    /// Ensure all `updated_values_for_existing_properties` satisfy unique option, if required
    pub fn ensure_property_values_unique_option_satisfied(
        updated_values_for_existing_properties: ValuesForExistingProperties<T>,
    ) -> dispatch::Result {
        for updated_value_for_existing_property in updated_values_for_existing_properties.values() {
            let (property, value) = updated_value_for_existing_property.unzip();

            // Ensure all PropertyValue's with unique option set are unique, except of null non required ones
            property
                .ensure_unique_option_satisfied(value, &updated_values_for_existing_properties)?;
        }
        Ok(())
    }

    /// Validate all values, provided in `values_for_existing_properties`, against the type of its `Property`
    /// and check any additional constraints
    pub fn ensure_property_values_are_valid(
        entity_controller: &EntityController<T>,
        values_for_existing_properties: &ValuesForExistingProperties<T>,
    ) -> dispatch::Result {
        for value_for_existing_property in values_for_existing_properties.values() {
            let (property, value) = value_for_existing_property.unzip();

            // Validate new PropertyValue against the type of this Property and check any additional constraints
            property.ensure_property_value_to_update_is_valid(value, entity_controller)?;
        }

        Ok(())
    }

    /// Ensure all provided `new_property_values` are already exist in `entity_property_values` map
    pub fn ensure_all_property_values_are_already_added(
        entity_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
        new_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> dispatch::Result {
        ensure!(
            new_property_values
                .keys()
                .all(|key| entity_property_values.contains_key(key)),
            ERROR_UNKNOWN_ENTITY_PROP_ID
        );
        Ok(())
    }

    /// Ensure `new_values_for_existing_properties` are accessible for actor with given `access_level`
    pub fn ensure_all_property_values_are_unlocked_from(
        new_values_for_existing_properties: &ValuesForExistingProperties<T>,
        access_level: EntityAccessLevel,
    ) -> dispatch::Result {
        for value_for_new_property in new_values_for_existing_properties.values() {
            // Ensure Property is unlocked from Actor with given EntityAccessLevel
            value_for_new_property
                .get_property()
                .ensure_unlocked_from(access_level)?;
        }
        Ok(())
    }

    /// Filter `new_property_values` identical to `entity_property_values`.
    pub fn try_filter_identical_property_values(
        entity_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
        new_property_values: BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> BTreeMap<PropertyId, PropertyValue<T>> {
        new_property_values
            .into_iter()
            .filter(|(id, new_property_value)| {
                matches!(
                    entity_property_values.get(id),
                    Some(entity_property_value) if *entity_property_value != *new_property_value
                )
            })
            .collect()
    }

    /// Update `entity_property_values` with `new_property_values`.
    /// Returns `Some(BTreeMap<PropertyId, PropertyValue<T>>)`,
    /// if `entity_property_values` have been updated, and `None` otherwise.
    pub fn make_updated_values(
        entity_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
        new_property_values: &BTreeMap<PropertyId, PropertyValue<T>>,
    ) -> Option<BTreeMap<PropertyId, PropertyValue<T>>> {
        // Used to check if updated performed
        let mut entity_property_values_updated = entity_property_values.to_owned();

        new_property_values
            .iter()
            .for_each(|(id, new_property_value)| {
                if let Some(entity_property_value) = entity_property_values_updated.get_mut(&id) {
                    entity_property_value.update(new_property_value.to_owned());
                }
            });

        if entity_property_values_updated != *entity_property_values {
            Some(entity_property_values_updated)
        } else {
            None
        }
    }

    /// Insert `Value` into `VecPropertyValue` at `index_in_property_vector`.
    /// Returns `PropertyValue`
    pub fn insert_at_index_in_property_vector(
        mut property_value_vector: VecPropertyValue<T>,
        index_in_property_vector: VecMaxLength,
        value: Value<T>,
    ) -> PropertyValue<T> {
        property_value_vector.insert_at(index_in_property_vector, value);
        PropertyValue::Vector(property_value_vector)
    }

    /// Remove `Value` at `index_in_property_vector` in `VecPropertyValue`.
    /// Returns `PropertyValue`
    pub fn remove_at_index_in_property_vector(
        mut property_value_vector: VecPropertyValue<T>,
        index_in_property_vector: VecMaxLength,
    ) -> PropertyValue<T> {
        property_value_vector.remove_at(index_in_property_vector);
        PropertyValue::Vector(property_value_vector)
    }

    /// Clear `VecPropertyValue`. Returns empty `PropertyValue`
    pub fn clear_property_vector(
        mut property_value_vector: VecPropertyValue<T>,
    ) -> PropertyValue<T> {
        property_value_vector.clear();
        PropertyValue::Vector(property_value_vector)
    }

    /// Insert `PropertyValue` into `entity_property_values` mapping at `in_class_schema_property_id`.
    /// Returns updated `entity_property_values`
    pub fn insert_at_in_class_schema_property_id(
        mut entity_property_values: BTreeMap<PropertyId, PropertyValue<T>>,
        in_class_schema_property_id: PropertyId,
        property_value: PropertyValue<T>,
    ) -> BTreeMap<PropertyId, PropertyValue<T>> {
        entity_property_values.insert(in_class_schema_property_id, property_value);
        entity_property_values
    }

    /// Ensure `Class` under given id exists, return corresponding one
    pub fn ensure_known_class_id(class_id: T::ClassId) -> Result<Class<T>, &'static str> {
        ensure!(<ClassById<T>>::exists(class_id), ERROR_CLASS_NOT_FOUND);
        Ok(Self::class_by_id(class_id))
    }

    /// Ensure `Entity` under given id exists, return corresponding one
    pub fn ensure_known_entity_id(entity_id: T::EntityId) -> Result<Entity<T>, &'static str> {
        ensure!(<EntityById<T>>::exists(entity_id), ERROR_ENTITY_NOT_FOUND);
        Ok(Self::entity_by_id(entity_id))
    }

    /// Ensure `CuratorGroup` under given id exists
    pub fn ensure_curator_group_under_given_id_exists(
        curator_group_id: &T::CuratorGroupId,
    ) -> dispatch::Result {
        ensure!(
            <CuratorGroupById<T>>::exists(curator_group_id),
            ERROR_CURATOR_GROUP_DOES_NOT_EXIST
        );
        Ok(())
    }

    /// Ensure `CuratorGroup` under given id exists, return corresponding one
    pub fn ensure_curator_group_exists(
        curator_group_id: &T::CuratorGroupId,
    ) -> Result<CuratorGroup<T>, &'static str> {
        Self::ensure_curator_group_under_given_id_exists(curator_group_id)?;
        Ok(Self::curator_group_by_id(curator_group_id))
    }

    /// Ensure `MaxNumberOfMaintainersPerClass` constraint satisfied
    pub fn ensure_maintainers_limit_not_reached(
        curator_groups: &BTreeSet<T::CuratorGroupId>,
    ) -> Result<(), &'static str> {
        ensure!(
            curator_groups.len() < T::MaxNumberOfMaintainersPerClass::get() as usize,
            ERROR_NUMBER_OF_MAINTAINERS_PER_CLASS_LIMIT_REACHED
        );
        Ok(())
    }

    /// Ensure all `CuratorGroup`'s under given ids exist
    pub fn ensure_curator_groups_exist(
        curator_groups: &BTreeSet<T::CuratorGroupId>,
    ) -> dispatch::Result {
        for curator_group in curator_groups {
            // Ensure CuratorGroup under given id exists
            Self::ensure_curator_group_exists(curator_group)?;
        }
        Ok(())
    }

    /// Perform security checks to ensure provided `class_maintainers` are valid
    pub fn ensure_class_maintainers_are_valid(
        class_maintainers: &BTreeSet<T::CuratorGroupId>,
    ) -> dispatch::Result {
        // Ensure max number of maintainers per Class constraint satisfied
        Self::ensure_maintainers_limit_not_reached(class_maintainers)?;

        // Ensure all curator groups provided are already exist in runtime
        Self::ensure_curator_groups_exist(class_maintainers)?;
        Ok(())
    }

    /// Ensure new `Schema` is not empty
    pub fn ensure_non_empty_schema(
        existing_properties: &BTreeSet<PropertyId>,
        new_properties: &[Property<T>],
    ) -> dispatch::Result {
        // Schema is empty if both existing_properties and new_properties are empty
        let non_empty_schema = !existing_properties.is_empty() || !new_properties.is_empty();
        ensure!(non_empty_schema, ERROR_NO_PROPS_IN_CLASS_SCHEMA);
        Ok(())
    }

    /// Ensure `ClassNameLengthConstraint` conditions satisfied
    pub fn ensure_class_name_is_valid(text: &[u8]) -> dispatch::Result {
        T::ClassNameLengthConstraint::get().ensure_valid(
            text.len(),
            ERROR_CLASS_NAME_TOO_SHORT,
            ERROR_CLASS_NAME_TOO_LONG,
        )
    }

    /// Ensure `ClassDescriptionLengthConstraint` conditions satisfied
    pub fn ensure_class_description_is_valid(text: &[u8]) -> dispatch::Result {
        T::ClassDescriptionLengthConstraint::get().ensure_valid(
            text.len(),
            ERROR_CLASS_DESCRIPTION_TOO_SHORT,
            ERROR_CLASS_DESCRIPTION_TOO_LONG,
        )
    }

    /// Ensure `MaxNumberOfClasses` constraint satisfied
    pub fn ensure_class_limit_not_reached() -> dispatch::Result {
        ensure!(
            T::MaxNumberOfClasses::get() < <ClassById<T>>::enumerate().count() as MaxNumber,
            ERROR_CLASS_LIMIT_REACHED
        );
        Ok(())
    }

    /// Ensure `MaxNumberOfEntitiesPerClass` constraint satisfied
    pub fn ensure_valid_number_of_entities_per_class(
        maximum_entities_count: T::EntityId,
    ) -> dispatch::Result {
        ensure!(
            maximum_entities_count < T::MaxNumberOfEntitiesPerClass::get(),
            ERROR_ENTITIES_NUMBER_PER_CLASS_CONSTRAINT_VIOLATED
        );
        Ok(())
    }

    /// Ensure `IndividualEntitiesCreationLimit` constraint satisfied
    pub fn ensure_valid_number_of_class_entities_per_actor_constraint(
        number_of_class_entities_per_actor: T::EntityId,
    ) -> dispatch::Result {
        ensure!(
            number_of_class_entities_per_actor < T::IndividualEntitiesCreationLimit::get(),
            ERROR_NUMBER_OF_CLASS_ENTITIES_PER_ACTOR_CONSTRAINT_VIOLATED
        );
        Ok(())
    }

    /// Ensure, that all entities creation limits, defined for a given `Class`, are valid
    pub fn ensure_entities_creation_limits_are_valid(
        maximum_entities_count: T::EntityId,
        default_entity_creation_voucher_upper_bound: T::EntityId,
    ) -> dispatch::Result {
        // Ensure `per_controller_entities_creation_limit` does not exceed
        ensure!(
            default_entity_creation_voucher_upper_bound < maximum_entities_count,
            ERROR_PER_CONTROLLER_ENTITIES_CREATION_LIMIT_EXCEEDS_OVERALL_LIMIT
        );

        // Ensure maximum_entities_count does not exceed MaxNumberOfEntitiesPerClass limit
        Self::ensure_valid_number_of_entities_per_class(maximum_entities_count)?;

        // Ensure default_entity_creation_voucher_upper_bound constraint does not exceed IndividualEntitiesCreationLimit
        Self::ensure_valid_number_of_class_entities_per_actor_constraint(
            default_entity_creation_voucher_upper_bound,
        )
    }

    /// Ensure maximum number of operations during atomic batching constraint satisfied
    pub fn ensure_number_of_operations_during_atomic_batching_limit_not_reached(
        operations: &[OperationType<T>],
    ) -> dispatch::Result {
        ensure!(
            operations.len() <= T::MaxNumberOfOperationsDuringAtomicBatching::get() as usize,
            ERROR_MAX_NUMBER_OF_OPERATIONS_DURING_ATOMIC_BATCHING_LIMIT_REACHED
        );
        Ok(())
    }

    /// Complete all checks to ensure each `Property` is valid
    pub fn ensure_all_properties_are_valid(new_properties: &[Property<T>]) -> dispatch::Result {
        for new_property in new_properties.iter() {
            // Ensure PropertyNameLengthConstraint satisfied
            new_property.ensure_name_is_valid()?;

            // Ensure PropertyDescriptionLengthConstraint satisfied
            new_property.ensure_description_is_valid()?;

            // Ensure Type specific constraints satisfied
            new_property.ensure_property_type_size_is_valid()?;

            // Ensure refers to existing class_id, if If Property Type is Reference,
            new_property.ensure_property_type_reference_is_valid()?;
        }
        Ok(())
    }

    /// Ensure all `Property` names are  unique within `Class`
    pub fn ensure_all_property_names_are_unique(
        class_properties: &[Property<T>],
        new_properties: &[Property<T>],
    ) -> dispatch::Result {
        // Used to ensure all property names are unique within class
        let mut unique_prop_names = BTreeSet::new();

        for property in class_properties.iter() {
            unique_prop_names.insert(property.name.to_owned());
        }

        for new_property in new_properties {
            // Ensure name of a new property is unique within its class.
            ensure!(
                !unique_prop_names.contains(&new_property.name),
                ERROR_PROP_NAME_NOT_UNIQUE_IN_A_CLASS
            );

            unique_prop_names.insert(new_property.name.to_owned());
        }

        Ok(())
    }

    /// Ensure provided indices of `existing_properties`  are valid indices of `Class` properties
    pub fn ensure_schema_properties_are_valid_indices(
        existing_properties: &BTreeSet<PropertyId>,
        class_properties: &[Property<T>],
    ) -> dispatch::Result {
        let has_unknown_properties = existing_properties
            .iter()
            .any(|&prop_id| prop_id >= class_properties.len() as PropertyId);
        ensure!(
            !has_unknown_properties,
            ERROR_CLASS_SCHEMA_REFERS_UNKNOWN_PROP_INDEX
        );
        Ok(())
    }

    /// Create new `Schema` from existing and new property ids
    pub fn create_class_schema(
        existing_properties: BTreeSet<PropertyId>,
        class_properties: &[Property<T>],
        new_properties: &[Property<T>],
    ) -> Schema {
        // Calcualate new property ids
        let properties = new_properties
            .iter()
            .enumerate()
            .map(|(i, _)| (class_properties.len() + i) as PropertyId)
            // Concatenate them with existing ones
            .chain(existing_properties.into_iter())
            .collect();

        Schema::new(properties)
    }

    /// Update existing `Class` properties with new ones provided, return updated ones
    pub fn make_updated_class_properties(
        class_properties: Vec<Property<T>>,
        new_properties: Vec<Property<T>>,
    ) -> Vec<Property<T>> {
        class_properties
            .into_iter()
            .chain(new_properties.into_iter())
            .collect()
    }
}

decl_event!(
    pub enum Event<T>
    where
        CuratorGroupId = <T as ActorAuthenticator>::CuratorGroupId,
        CuratorId = <T as ActorAuthenticator>::CuratorId,
        ClassId = <T as Trait>::ClassId,
        EntityId = <T as Trait>::EntityId,
        EntityController = EntityController<T>,
        EntityCreationVoucher = EntityCreationVoucher<T>,
        Status = bool,
        Actor = Actor<T>,
        Nonce = <T as Trait>::Nonce,
    {
        CuratorGroupAdded(CuratorGroupId),
        CuratorGroupRemoved(CuratorGroupId),
        CuratorGroupStatusSet(Status),
        CuratorAdded(CuratorGroupId, CuratorId),
        CuratorRemoved(CuratorGroupId, CuratorId),
        MaintainerAdded(ClassId, CuratorGroupId),
        MaintainerRemoved(ClassId, CuratorGroupId),
        EntityCreationVoucherUpdated(EntityController, EntityCreationVoucher),
        EntityCreationVoucherCreated(EntityController, EntityCreationVoucher),
        ClassCreated(ClassId),
        ClassPermissionsUpdated(ClassId),
        ClassSchemaAdded(ClassId, SchemaId),
        ClassSchemaStatusUpdated(ClassId, SchemaId, Status),
        EntityPermissionsUpdated(EntityId),
        EntityCreated(Actor, EntityId),
        EntityRemoved(Actor, EntityId),
        EntitySchemaSupportAdded(Actor, EntityId, SchemaId),
        EntityPropertyValuesUpdated(Actor, EntityId),
        EntityPropertyValueVectorCleared(Actor, EntityId, PropertyId),
        RemovedAtEntityPropertyValueVectorIndex(Actor, EntityId, PropertyId, VecMaxLength, Nonce),
        InsertedAtEntityPropertyValueVectorIndex(Actor, EntityId, PropertyId, VecMaxLength, Nonce),
        TransactionCompleted(Actor),
        EntityOwnershipTransfered(EntityId, EntityController),
    }
);
