use std::time::Duration;

use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_rapier3d::{prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController}};
use bevy_landmass::{
	Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentState, AgentTarget3d, ArchipelagoRef3d, Character, Island, TargetReachedCondition, coords::ThreeD
};
use super::{Health};

const SKELETON_PATH: &str = "models/skeleton.glb";

// -- PLUGIN --
pub struct EnemySpawnPlugin;
impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(OnExit(super::MyAppState::Loading), 
			(
				//
				load_animations.after(super::setup_player),
				load_actor_spawners
			)
		)
		.add_systems(Update, (
				(
					link_animations,
					update_enemies,
				)
				.chain()
				.after(crate::ui::update_ui),
				use_actor_spawners,
			).in_set(super::GameplaySet)
		);
		
    }
}

// -- animation setup --

#[derive(Resource)]
pub struct Animations { // taken from bevy example animations
	animations: Vec<AnimationNodeIndex>,
	graph_handle: Handle<AnimationGraph>,
}

pub fn load_animations(
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	mut graphs: ResMut<Assets<AnimationGraph>>,
) {
	debug!("Loading all required animations...");
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

// -- actor spawning system --
// Entities with ActorBundles are considered "actors," which could be enemies, players, NPCs etc.
// TODO: move to own file

#[derive(Component, Default, Clone, Copy, PartialEq)]
// The type of an actor. 
// Enemies take in a radius parameter which defines their line of sight.
// TODO: Use this enum (and this system in general) for setting up the player.
pub enum ActorType {
	Enemy{radius: f32},
	Player,
	#[default]
	Neutral
}

#[derive(Bundle)]
// A bundle containing all the components necessary to have an actor.
pub struct ActorBundle {
	actor_type: ActorType,
	health: Health,
	transform: Transform,
	visibility: Visibility,
	collider: Collider,
	character_controller: KinematicCharacterController,
}

impl Default for ActorBundle {
	fn default() -> Self {
		Self { 
			actor_type: ActorType::Enemy { radius: 100.0 },
			health: Health { hp: 100 }, 
			transform: Transform::default(), 
			visibility: Visibility::Visible,
			collider: Collider::round_cylinder(1.0, 0.1, 0.0),
			character_controller: KinematicCharacterController {
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
				apply_impulse_to_dynamic_bodies: false,
				snap_to_ground: Some(CharacterLength::Absolute(5.0)),
				..default()
			},
		}
	}
}

// A template struct that contains information about a type of enemy.
// In theory, should be serializable/deserializable. 
// TODO: add serializability from/to .rom files and rename to something more general like "ActorInfo"
pub struct ActorSpawnerTemplate {
	actor_type: ActorType,
	half_height: f32,
	radius: f32,
	visibility: bool,
	max_health: u32,
	agent_settings: AgentSettings,
	target_reached_condition: TargetReachedCondition,
	model_path: String,
	actor_ai: Option<Tree<Behave>>,
}

#[derive(Component)]
// A component that spawns actors.
pub struct ActorSpawner {
	// list of positions to spawn actors at. The length of this Vec determines how many actors this will spawn.
	positions: Vec<Vec3>, 
	// template to spawn actors according to.
	template: ActorSpawnerTemplate,
	// turning this true during runtime will use this spawner and delete it
	// see UseActorSpawners event
	use_next_tick: bool, 
}

impl ActorSpawner {
	pub fn new(positions: Vec<Vec3>, template: ActorSpawnerTemplate, use_next_tick: bool) -> ActorSpawner {
		debug!("Spawning actor spawner...");
		ActorSpawner {
			positions, template, use_next_tick
		}
	}

	pub fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, archipelago_ref: &Entity, player_entity: Option<&Entity>) {
		debug!("Spawning {} actors...", self.positions.len());
		self.positions.iter().for_each(|&pos| 
			{ 
				commands
					.spawn(
						ActorBundle {
							actor_type: self.template.actor_type.clone(),
							visibility: match self.template.visibility {
								true => Visibility::Visible,
								false => Visibility::Hidden,
							},
							health: Health { hp: self.template.max_health },
							transform: Transform::from_translation(pos),
							collider: Collider::round_cylinder(self.template.half_height, self.template.radius, 0.0),
							..default()
						},
					)
					.with_children(|parent| {
						parent
							// child entity containing navigation components
							.spawn((
								Transform::from_xyz(0.0, -(self.template.half_height + self.template.radius), 0.0),
								Agent3dBundle {
									agent: default(),
									settings: AgentSettings { 
										radius: self.template.agent_settings.radius, 
										desired_speed: self.template.agent_settings.desired_speed, 
										max_speed: self.template.agent_settings.max_speed,
									},
									archipelago_ref: ArchipelagoRef3d::new(*archipelago_ref),
								},
								self.template.target_reached_condition,
								LastState(AgentState::Idle),
							))
							// insert behavior tree if agent ai is specified in template
							.insert_if(
								BehaveTree::new(self.template.actor_ai.clone().unwrap()),
								|| {self.template.actor_ai.is_some()}
							)
							// insert player as agent target if template is an enemy
							.insert_if(
								AgentTarget3d::Entity(*player_entity.unwrap()),
								|| {player_entity.is_some()}
							);
						
						// child entity containing enemy model
						parent.spawn((
							// scene
							Transform::from_xyz(0.0, -self.template.half_height, 0.0),
							SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(self.template.model_path.clone()))),
						));
					}); 
			}
		);
	}
}

// Use any actor spawners with use_next_tick set to true, and despawn them.
// This system should run on update.
pub fn use_actor_spawners(
	spawners: Query<(Entity, &ActorSpawner)>,
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	island_archipelago_ref: Query<&mut ArchipelagoRef3d, With<Island>>,
	player: Query<Entity, With<Character<ThreeD>>>,
) {
	let player = player.single().unwrap();
	for (entity, spawner) in &spawners {
		if spawner.use_next_tick {
			let player_entity = match spawner.template.actor_type {
				ActorType::Enemy { radius: _ } => Some(&player),
				_ => None
			};

			spawner.spawn(
				&mut commands, 
				&asset_server, 
				&island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity,
				player_entity,
			);
		}
		commands.entity(entity).despawn();
	}
}

// Load a skeleton. This is mainly for testing the game right now.
pub fn load_actor_spawners(mut commands: Commands) {
	commands.spawn(ActorSpawner::new(
		vec![Vec3::new(-35.0, 3.7, -10.0)],
		ActorSpawnerTemplate {
			actor_type: ActorType::Enemy { radius: 100.0 },
			half_height: 1.0,
			radius: 0.3,
			visibility: true,
			max_health: 100,
			agent_settings: AgentSettings { radius: 0.3, desired_speed: 1.0, max_speed: 3.0 },
			target_reached_condition: TargetReachedCondition::Distance(Some(1.0)),
		    model_path: SKELETON_PATH.to_string(),
			actor_ai: Some(tree!{Behave::AlwaysFail})
		},
		true
	));
}

// -- actor updating system
// TODO: fix animations not working
// TODO: move to own file

#[derive(Component, Default)]
pub struct LastState(AgentState);

pub fn update_enemies(
	mut actor_query: Query<(&ActorType, &mut KinematicCharacterController, &mut Transform, &AnimationEntityLink), With<ActorType>>,
	mut agent_query: Query<(&ChildOf, &AgentState, &AgentDesiredVelocity3d, &mut LastState)>,
	mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations: Res<Animations>,
	time: Res<Time>,
) {
	for (
		childof, 
		agent_state, 
		desired_velocity, 
		mut last_state
	) in &mut agent_query {
		if let Ok((
			actor_type,
			mut controller,
			mut transform,
			animation_link,
		)) = actor_query.get_mut(childof.parent()) {

			// if not an enemy, skip over this actor
			match actor_type {
				ActorType::Enemy {radius: _ } => {},
				_ => {continue}
			}

			let (
				mut animation_player, 
				mut animation_transitions
			) = animation_query.get_mut(animation_link.0).unwrap();

			// TODO: add behavior tree action here
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