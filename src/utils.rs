use bevy::prelude::*;

// https://github.com/SnowdenWintermute/bevy-multiple-characters-animation/blob/main/src/animated_character/link_animations.rs
pub fn get_top_parent(
    mut curr_entity: Entity,
    all_entities_with_parents_query: &Query<&ChildOf>,
) -> Entity {
    // Loop up all the way to the top parent
    loop {
        if let Ok(ref_to_parent) = all_entities_with_parents_query.get(curr_entity) {
            curr_entity = ref_to_parent.parent();
        } else {
            break;
        }
    }
    curr_entity
}