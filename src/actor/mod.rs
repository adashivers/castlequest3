use bevy::prelude::*;
use bevy_mod_skinned_aabb::SkinnedAabbPlugin;
use animations::*;
use spawner::*;
use update::*;
use navigation::*;

const SKELETON_PATH: &str = "models/skeleton.glb";
pub mod animations;
pub mod spawner;
pub mod update;
pub mod navigation;

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
		)) 
		.add_systems(OnExit(super::MyAppState::Loading), 
			(
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
