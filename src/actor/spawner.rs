use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_rapier3d::{prelude::{Collider,}};
use bevy_landmass::{
	Agent3dBundle, AgentSettings, AgentState, AgentTarget3d, ArchipelagoRef3d, Character, Island, TargetReachedCondition, coords::ThreeD
};
use super::{Health, SKELETON_PATH};
use crate::actor::*;


// A template struct that contains information about a type of enemy.
// In theory, should be serializable/deserializable. 
// TODO: add serializability from/to .rom files and rename to something more general like "ActorInfo"
pub struct ActorSpawnerTemplate {
	actor_name: String,
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
					.spawn((
						Name::new(self.template.actor_name.clone()),
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
					))
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
						let model_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(self.template.model_path.clone()));
						
						parent.spawn((
							// scene
							Transform::from_xyz(0.0, -self.template.half_height, 0.0),
							SceneRoot(model_scene),
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

// Load a spawner that spawns a skeleton in the northern hallway. This is mainly for testing the game right now.
pub fn load_actor_spawners(mut commands: Commands) {
	commands.spawn(ActorSpawner::new(
		vec![Vec3::new(-35.0, 3.7, -10.0)],
		ActorSpawnerTemplate {
			actor_name: "Skeleton".into(),
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