use super::*;

#[test]
fn insert_at_entity_property_vector_success() {
    with_test_externalities(|| {
        let actor = Actor::Lead;

        // Add entity schemas support
        let (mut first_entity, mut second_entity) = add_entity_schemas_support();

        // Runtime state before tested call

        // Events number before tested call
        let number_of_events_before_calls = System::events().len();

        // Insert `SinglePropertyValue` at given `index_in_property_vector`
        // into `PropertyValueVec` under `in_class_schema_property_id`
        let nonce = 0;
        let index_in_property_vector = 1;
        let single_property_value = SinglePropertyValue::new(Value::Reference(SECOND_ENTITY_ID));

        assert_ok!(insert_at_entity_property_vector(
            LEAD_ORIGIN,
            actor.clone(),
            FIRST_ENTITY_ID,
            SECOND_PROPERTY_ID,
            index_in_property_vector,
            single_property_value,
            nonce
        ));

        // Runtime tested state after call

        // Ensure first entity properties updated succesfully
        if let Some(second_schema_old_property_value) = first_entity
            .values
            .get_mut(&SECOND_PROPERTY_ID)
            .and_then(|property_value| property_value.as_vec_property_value_mut())
        {
            second_schema_old_property_value
                .insert_at(index_in_property_vector, Value::Reference(SECOND_ENTITY_ID));
        }

        assert_eq!(first_entity, entity_by_id(FIRST_ENTITY_ID));

        // Ensure reference counter of second entity updated succesfully
        let inbound_rc = InboundReferenceCounter::new(4, true);
        *second_entity.get_reference_counter_mut() = inbound_rc.clone();

        assert_eq!(second_entity, entity_by_id(SECOND_ENTITY_ID));

        // Create side-effect
        let side_effect = EntityReferenceCounterSideEffect::new(1, 1);

        let inserted_at_vector_index_event = get_test_event(RawEvent::InsertedAtVectorIndex(
            actor,
            FIRST_ENTITY_ID,
            SECOND_PROPERTY_ID,
            index_in_property_vector,
            nonce + 1,
            Some((SECOND_ENTITY_ID, side_effect)),
        ));

        // Last event checked
        assert_event_success(
            inserted_at_vector_index_event,
            number_of_events_before_calls + 1,
        );
    })
}
