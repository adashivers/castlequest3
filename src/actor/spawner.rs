use bevy::prelude::*;
use bevy_rapier3d::{prelude::{Collider,}};
use bevy_landmass::{
	Agent3dBundle, AgentSettings, ArchipelagoRef3d, Island, TargetReachedCondition,
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
	max_health: i32,
	agent_settings: AgentSettings,
	target_reached_condition: TargetReachedCondition,
	model_path: String,
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

#[derive(Component)]
// A component that spawns actors.
pub struct ReturnPoint(pub Vec3);

impl ActorSpawner {
	pub fn new(positions: Vec<Vec3>, template: ActorSpawnerTemplate, use_next_tick: bool) -> ActorSpawner {
		debug!("Spawning actor spawner...");
		ActorSpawner {
			positions, template, use_next_tick
		}
	}

	pub fn spawn(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, archipelago_ref: &Entity) {
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
						let offset = Vec3::new(0.0, -(self.template.half_height + self.template.radius), 0.0);
						parent
							// child entity containing navigation components
							.spawn((
								Transform::from_translation(offset),
								Agent3dBundle {
									agent: default(),
									settings: AgentSettings { 
										radius: self.template.agent_settings.radius, 
										desired_speed: self.template.agent_settings.desired_speed, 
										max_speed: self.template.agent_settings.max_speed,
									},
									archipelago_ref: ArchipelagoRef3d::new(*archipelago_ref),
								},
								MoveAgent::Idle,
								ReturnPoint(pos + offset),
								self.template.target_reached_condition,
							));
						
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
) {
	for (entity, spawner) in &spawners {
		if spawner.use_next_tick {
			spawner.spawn(
				&mut commands, 
				&asset_server, 
				&island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity,
			);
		}
		commands.entity(entity).despawn();
	}
}

// Load a spawner that spawns a skeleton in the northern hallway. This is mainly for testing the game right now.
pub fn load_actor_spawners(mut commands: Commands) {
	commands.spawn(ActorSpawner::new(
		vec![Vec3::new(-35.0, 3.7, -10.0), Vec3::new(-22.0, 2.4, 0.0)],
		ActorSpawnerTemplate {
			actor_name: "Skeleton".into(),
			actor_type: ActorType::Enemy { sight_radius: 30.0, attack_radius: 1.3, home_radius: 15.0 },
			half_height: 1.0,
			radius: 0.3,
			visibility: true,
			max_health: 100,
			agent_settings: AgentSettings { radius: 0.3, desired_speed: 1.0, max_speed: 3.0 },
			target_reached_condition: TargetReachedCondition::Distance(Some(0.1)),
		    model_path: SKELETON_PATH.to_string(),
		},
		true
	));
}