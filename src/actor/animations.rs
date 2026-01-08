use std::time::Duration;
use bevy::{animation::AnimationTargetId, prelude::*};
use crate::utils::get_top_parent;

use super::SKELETON_PATH;

#[derive(Resource)]
pub struct Animations { // taken from bevy example animations
	pub animations: Vec<AnimationNodeIndex>,
	pub graph_handle: Handle<AnimationGraph>,
}

#[derive(Resource, Clone)]
pub struct SkeletonAnimTargets {
	pub right_hand: AnimationTargetId
}
impl Default for SkeletonAnimTargets {
    fn default() -> Self {
		let right_hand_names = [
			Name::new("mixarmorig:Hips"),
			Name::new("mixarmorig:Spine1"),
			Name::new("mixarmorig:Spine2"),
			Name::new("mixarmorig:Spine2"),
			Name::new("mixarmorig:RightShoulder"),
			Name::new("mixarmorig:RightArm"),
			Name::new("mixarmorig:RightForeArm"),
			Name::new("mixarmorig:RightHand"),
			];
		SkeletonAnimTargets {
			right_hand: AnimationTargetId::from_names(right_hand_names.iter())
		}
		
	}
}


pub fn load_animations(
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	mut graphs: ResMut<Assets<AnimationGraph>>,
) {
	debug!("Loading all required animations...");
	let clips = [
		asset_server.load(GltfAssetLabel::Animation(0).from_asset(SKELETON_PATH)), // idle
		asset_server.load(GltfAssetLabel::Animation(1).from_asset(SKELETON_PATH)), // swing
		asset_server.load(GltfAssetLabel::Animation(2).from_asset(SKELETON_PATH)), // walk
	];

	let (graph, node_indices) = AnimationGraph::from_clips(clips);

	// Keep our animation graph in a Resource so that it can be inserted onto
	// the correct entity once the scene actually loads.
	let graph_handle = graphs.add(graph);
	commands.insert_resource(Animations {
		animations: node_indices,
		graph_handle,
	});

}



#[derive(Component, Debug)]
pub struct AnimationEntityLink(pub Entity);

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// An `AnimationTransitions` component should always be added to the same entity 
// that has the `AnimationPlayer` component it's refering to.
// An `AnimationEntityLink` is on the topmost parent of the entity that has the `AnimationPlayer`.
pub fn link_animations(
	mut commands: Commands,
	animations: Res<Animations>,
	all_entities_with_parents_query: Query<&ChildOf>,
	mut anim_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
	animation_link_query: Query<&AnimationEntityLink>,
) {
	if !anim_players.is_empty() {
		debug!("running link animations system");
	}
	for (anim_player_entity, mut anim_player) in &mut anim_players {

		let top_entity = get_top_parent(anim_player_entity, &all_entities_with_parents_query);
        if animation_link_query.get(top_entity).is_ok() {
            warn!("\tProblem with multiple animation players for the same top parent");
        } else {
			debug!("\tinserting animation link to entity {}", top_entity.row().index());
			commands.entity(top_entity).insert(AnimationEntityLink(anim_player_entity.clone()));
			//debug!("Top entity:\n{:#?}", world.inspect_entity(top_entity).unwrap().map(|info| info.name()).collect::<Vec<_>>());
		}

		
		debug!("\tsetting up transitions");
		let mut transitions = AnimationTransitions::new();
		transitions
            .play(&mut anim_player, animations.animations[0], Duration::ZERO)
            .repeat();
		commands
			.entity(anim_player_entity)
			.insert(AnimationGraphHandle(animations.graph_handle.clone()))
			.insert(transitions);
	}
}