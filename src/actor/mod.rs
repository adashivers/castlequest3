use bevy::{prelude::*};
use bevy_behave::prelude::BehavePlugin;
use bevy_rapier3d::{prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController}};
use bevy_mod_skinned_aabb::SkinnedAabbPlugin;

use animations::*;
use spawner::*;
use navigation::*;
use behavior::*;

const SKELETON_PATH: &str = "models/skeleton.glb";
pub mod animations;
pub mod spawner;
pub mod navigation;
mod behavior;

// -- PLUGIN --
pub struct EnemySpawnPlugin;
impl Plugin for EnemySpawnPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_plugins((
			// needed for cull masking actor models properly.
			// apparently there is a PR for this in Bevy 0.18!
			SkinnedAabbPlugin,
			ActorNavigationPlugin,
			BehavePlugin::default(),
		)) 
		.init_resource::<SkeletonAnimTargets>()
		.add_observer(on_check_entity_in_sight)
		.add_observer(on_set_move_towards_target)
		.add_systems(Startup, load_animations)
		.add_systems(OnExit(super::MyAppState::Loading), 
			
			load_actor_spawners,
			
		)
		.add_systems(Update, (
				(
					link_animations,
					init_actor_behavior,
				)
				.chain()
				.after(crate::ui::update_ui),
				use_actor_spawners,
				move_agents,
				update_agent_animations,
				on_attack,
				
			).in_set(super::GameplaySet)
		)
		.add_systems(Last, (
			update_moveagent_laststate,
		));
		
    }
}

#[derive(Component)]
pub struct Health {
    pub hp: u32,
}

impl Default for Health {
    fn default() -> Self {
        Health { hp: 100 }
    }
}

#[derive(Component, Default)]
pub struct Player;

// Entities with ActorBundles are considered "actors," which could be enemies, players, NPCs etc.

#[derive(Component, Default, Clone, Copy, PartialEq)]
// The type of an actor. 
// Enemies take in a radius parameter which defines their line of sight.
// TODO: Use this enum (and this system in general) for setting up the player.
pub enum ActorType {
	Enemy{sight_radius: f32, attack_radius: f32},
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
			actor_type: ActorType::Enemy { sight_radius: 100.0, attack_radius: 1.0 },
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
