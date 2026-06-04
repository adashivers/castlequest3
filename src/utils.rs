use bevy::{asset::{LoadState, UntypedAssetId}, prelude::*};

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

// get a cumulative load state for a list of handles (only success if all of them are loaded)
// apparently this used to be a library method but was removed during a revamp and never added back
pub fn get_group_load_state(
    server: &AssetServer,
    handles: impl IntoIterator<Item = UntypedAssetId>,
) -> LoadState {
    let mut load_state = LoadState::Loaded;
    for handle_id in handles {
        match server.get_load_state(handle_id) {
            Some(LoadState::Loaded) => continue,
            Some(LoadState::Loading) => {
                load_state = LoadState::Loading;
            }
            Some(LoadState::Failed(x)) => return LoadState::Failed(x),
            Some(LoadState::NotLoaded) => return LoadState::NotLoaded,
            None => return LoadState::NotLoaded,
        }
    }

    load_state
}