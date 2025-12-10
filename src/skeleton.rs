use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController};
use bevy_landmass::{
	Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentState, AgentTarget3d, ArchipelagoRef3d, Character, Island, TargetReachedCondition, coords::ThreeD
};
use super::{Health};

const SKELETON_PATH: &str = "models/skeleton.glb";

pub struct EnemySpawnPlugin;
impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(OnExit(super::MyAppState::Loading), 
			(
				load_animations,
				spawn,
			).chain().after(super::setup_player)
		)
		.add_systems(Update, 
			(
				link_animations,
				update_skellys,
			)
			.chain()
			.after(crate::ui::update_ui)
			.in_set(super::GameplaySet)
		);
		
    }
}

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

// next two methods built with reference from here:
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
	mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
	animation_link_query: Query<&AnimationEntityLink>,
) {
	
	for (anim_player_entity, mut player) in &mut players {
		debug!("linking animation players");
		let top_entity = get_top_parent(anim_player_entity, &all_entities_with_parents_query);
        if animation_link_query.get(top_entity).is_ok() {
            warn!("Problem with multiple animation players for the same top parent");
        } else {
			commands.entity(top_entity).insert(AnimationEntityLink(anim_player_entity.clone()));
			//debug!("Top entity:\n{:#?}", world.inspect_entity(top_entity).unwrap().map(|info| info.name()).collect::<Vec<_>>());
		}

		
		debug!("setting up transitions");
		let mut transitions = AnimationTransitions::new();
		transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();
		commands
			.entity(anim_player_entity)
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
  let player_entity = player_query.single().expect("Could not find player while spawning enemy").0;

  commands.spawn((
	Skeleton::default(),
	Health { hp: 100 },
	Transform::from_xyz(-37.0, 3.7, -15.0),
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
				archipelago_ref: ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity),
			},
			AgentTarget3d::Entity(player_entity),
			TargetReachedCondition::Distance(Some(2.0)),
			LastState(AgentState::Idle),
		),
		(
			// scene
			Transform::from_xyz(0.0, -1.0, 0.0),
			SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skeleton.glb"))),
		)
	]
  ));

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
				archipelago_ref: ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity),
			},
			AgentTarget3d::Entity(player_entity),
			TargetReachedCondition::Distance(Some(2.0)),
			LastState(AgentState::Idle),
		),
		(
			// scene
			Transform::from_xyz(0.0, -1.0, 0.0),
			SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/skeleton.glb"))),
		)
	]
  ));
}

pub fn update_skellys(
	mut skelly_query: Query<(&mut KinematicCharacterController, &mut Transform, &AnimationEntityLink), With<Skeleton>>,
	mut skelly_agents: Query<(&ChildOf, &AgentState, &AgentDesiredVelocity3d, &mut LastState)>,
	mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations: Res<Animations>,
	time: Res<Time>,
) {
	for (
		childof, 
		agent_state, 
		desired_velocity, 
		mut last_state
	) in skelly_agents.iter_mut() {
		
		// get other components associated with this skeleton.
		// first get components from top parent
		// animations might not be linked yet, so we put an ok wrapper
		if let Ok((
			mut controller,
			mut transform,
			animation_link,
		)) = skelly_query.get_mut(childof.parent()) {

			// then get animation components using link
			let (
				mut animation_player, 
				mut animation_transitions
			) = animation_query.get_mut(animation_link.0).unwrap();


			// set animation depending on agent state
			if *agent_state != last_state.0 {
				debug!("skeleton state: {:?}", agent_state);
				match agent_state {
					AgentState::Moving => {
						animation_transitions
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
						animation_transitions
							.play(
								&mut animation_player, 
								animations.animations[1], 
								Duration::from_millis(250)
							)
							.repeat();
					},
					_ => {
						animation_transitions
							.play(
								&mut animation_player, 
								animations.animations[0], 
								Duration::from_millis(250)
							)
							.repeat();
					},
				}
				last_state.0 = *agent_state;
			}

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
}