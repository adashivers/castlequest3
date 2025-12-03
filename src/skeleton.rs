use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController};
use bevy_landmass::{
	Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentState, AgentTarget3d, ArchipelagoRef3d, Character, Island, TargetReachedCondition, coords::ThreeD
};
use super::{Health};

const SKELETON_PATH: &str = "models/skeleton.glb";

#[derive(Resource)]
pub struct Animations {
	animations: Vec<AnimationNodeIndex>,
	graph_handle: Handle<AnimationGraph>,
}

pub fn load_animations(
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	mut graphs: ResMut<Assets<AnimationGraph>>,
) {
	let (graph, node_indices) = AnimationGraph::from_clips([
		asset_server.load(GltfAssetLabel::Animation(0).from_asset(SKELETON_PATH)), // idle
		asset_server.load(GltfAssetLabel::Animation(1).from_asset(SKELETON_PATH)), // swing
		asset_server.load(GltfAssetLabel::Animation(2).from_asset(SKELETON_PATH)), // walk
	]);

	// Keep our animation graph in a Resource so that it can be inserted onto
	// the correct entity once the scene actually loads.
	let graph_handle = graphs.add(graph);
	commands.insert_resource(Animations {
		animations: node_indices,
		graph_handle,
	});

}

// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
pub fn setup_animations_once_loaded(
	mut commands: Commands,
	animations: Res<Animations>,
	mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
	
	for (entity, mut player) in &mut players {
		debug!("setting up transitions");
		let mut transitions = AnimationTransitions::new();

		// Make sure to start the animation via the `AnimationTransitions`
		// component. The `AnimationTransitions` component wants to manage all
		// the animations and will get confused if the animations are started
		// directly via the `AnimationPlayer`.
		transitions
			.play(&mut player, animations.animations[0], Duration::ZERO)
			.repeat();

		commands
			.entity(entity)
			.insert(AnimationGraphHandle(animations.graph_handle.clone()))
			.insert(transitions);
	}
}


#[derive(Component, Default)]
pub struct Skeleton;
#[derive(Component, Default)]
pub struct LastState(AgentState);

pub fn spawn(
  mut commands: Commands,
  mut materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
  island_archipelago_ref: Query<&mut ArchipelagoRef3d, With<Island>>,
  player_query: Query<(Entity, &Character<ThreeD>)>,
) {
  let archipelago_ref = ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity);
  let player_entity = player_query.single().expect("Could not find player while spawning enemy").0;

  commands.spawn((
	Skeleton::default(),
	Health { hp: 100 },
	Transform::from_xyz(-35.0, 3.7, -10.0),
	Visibility::default(),
	Collider::round_cylinder(1.0, 0.1, 0.0),
	MeshMaterial3d(materials.add(StandardMaterial::default())),
	KinematicCharacterController {
		custom_mass: Some(5.0),
		up: Vec3::Y,
		offset: CharacterLength::Absolute(0.01),
		slide: true,
		autostep: Some(CharacterAutostep {
			max_height: CharacterLength::Relative(0.3),
			min_width: CharacterLength::Relative(0.5),
			include_dynamic_bodies: false,
		}),
		// Don’t allow climbing slopes larger than 45 degrees.
		max_slope_climb_angle: 45.0_f32.to_radians(),
		// Automatically slide down on slopes smaller than 30 degrees.
		min_slope_slide_angle: 30.0_f32.to_radians(),
		apply_impulse_to_dynamic_bodies: true,
		snap_to_ground: Some(CharacterLength::Absolute(5.0)),
		..default()
	},
	children![
		(
			// navmesh agent
			Transform::from_xyz(0.0, -1.1, 0.0),
			Agent3dBundle {
				agent: default(),
				settings: AgentSettings {
				radius: 0.3,
				desired_speed: 1.0,
				max_speed: 5.0,
				},
				archipelago_ref,
			},
			AgentTarget3d::Entity(player_entity),
			TargetReachedCondition::Distance(Some(2.0)),
			LastState(AgentState::Idle),
		),
		(
			// skeleton mesh
			Transform::from_xyz(0.0, -1.0, 0.0),
			SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skeleton.glb"))),
		)
	]
  ));
}



pub fn update_skellys(
	mut skelly_query: Query<(&mut KinematicCharacterController, &mut Transform), With<Skeleton>>,
	mut skelly_agents: Query<(&ChildOf, &AgentState, &AgentDesiredVelocity3d, &mut LastState)>,
	mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations: Res<Animations>,
	time: Res<Time>,
) {
	for (
		parent, 
		agent_state, 
		desired_velocity, 
		mut last_state
	) in skelly_agents.iter_mut() {
		
		// set animation depending on agent state
		if *agent_state != last_state.0 {
			debug!("skeleton state: {:?}", agent_state);
			for (
				mut animation_player, 
				mut transitions
			) in animation_players.iter_mut() {
				match agent_state {
					AgentState::Moving => {
						transitions
							.play(
								&mut animation_player, 
								animations.animations[2], 
								Duration::from_millis(250)
							)
							.repeat();
					},
					AgentState::ReachedTarget => {
						// this will change in the future to trigger an "on attack" event
						// instead of playing the attack animation on loop
						transitions
							.play(
								&mut animation_player, 
								animations.animations[1], 
								Duration::from_millis(250)
							)
							.repeat();
					},
					_ => {
						transitions
							.play(
								&mut animation_player, 
								animations.animations[0], 
								Duration::from_millis(250)
							)
							.repeat();
					},
				}
			}
			last_state.0 = *agent_state;
		}
		
		let (mut controller, mut transform) = skelly_query.get_mut(parent.parent()).unwrap();

		// align transform rotation so that skeleton looks where it's going
		if desired_velocity.velocity().length() > 0.1 {
			transform.align(Dir3::X, desired_velocity.velocity().normalize(), Dir3::Y, Dir3::Y);
		}
		
		// set next velocity
		let mut next_velocity: Vec3 = Vec3::new(0.0, -0.1, 0.0); // slight downward tilt so that the collider snaps to the ground.
		next_velocity += desired_velocity.velocity(); // add desired velocity
		controller.translation = Some(next_velocity * time.delta_secs());
	
  }
}