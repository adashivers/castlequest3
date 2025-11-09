use std::fmt::Debug;
use crate::loading_system::*;
use crate::player_movement::*;
use crate::ui::*;
use crate::actor_navigation::*;
use crate::debug::*;

use bevy::{input::InputSystem, prelude::*, log::LogPlugin};

use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use vleue_navigator::VleueNavigatorPlugin;

pub mod loading_system;
pub mod player_movement;
pub mod ui;
pub mod actor_navigation;
pub mod debug;

// States of the app in general. Could become more complicated in the future
#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MyAppState {
    Loading,
    InGame,
}

// For systems that should only run when app is loading
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadingSet;

// For systems that should only run when app is in game
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameplaySet;

// a list of mesh handles for the levels currently open.
// This should be a Single resource
// Clear vector to unload resources whenever possible
#[derive(Default, Resource)]
pub struct LevelHandles(Vec<Handle<Mesh>>);



fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        .init_resource::<MovementInput>()
        .init_resource::<LookInput>()
        .init_resource::<AssetsLoading>()
        .init_resource::<LevelHandles>()
        .init_resource::<NavMeshPrimitives>()
        .init_resource::<DebugFlags>() // remove this to disable debug stuff
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "moving_around=debug,wgpu_core=warn,wgpu_hal=warn".into(),
                level: bevy::log::Level::INFO,
                custom_layer: |_| None,
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            VleueNavigatorPlugin,
        ))
        .insert_state(MyAppState::Loading)
        .configure_sets(
            PreUpdate,
            (
                GameplaySet.run_if(in_state(MyAppState::InGame)),
                LoadingSet.run_if(in_state(MyAppState::Loading)),
            ),
        )
        .configure_sets(
            Update,
            (
                GameplaySet.run_if(in_state(MyAppState::InGame)),
                LoadingSet.run_if(in_state(MyAppState::Loading)),
            ),
        )
        .configure_sets(
            FixedUpdate,
            (
                GameplaySet.run_if(in_state(MyAppState::InGame)),
                LoadingSet.run_if(in_state(MyAppState::Loading)),
            ),
        )
        .add_systems(Startup, (
            start_loading_assets, 
            start_loading_navmesh, 
            (
                setup_ui, 
                setup_debug_ui.run_if(resource_exists::<DebugFlags>)
            ).chain(),
        ))
        .add_systems(OnEnter(MyAppState::InGame), (spawn_level_map, spawn_navmesh, setup_player, spawn_enemy))
        .add_systems(
            PreUpdate,
            ((handle_input, handle_debug_input.run_if(resource_exists::<DebugFlags>), player_movement, update_enemy)
                .chain()
                .after(InputSystem)
                .in_set(GameplaySet),),
        )
        .add_systems(
            Update,
            (
                (player_look).in_set(GameplaySet),
                (checks_assets_loaded).in_set(LoadingSet),
                update_ui,
                update_debug_ui.run_if(resource_exists::<DebugFlags>),
            ),
        )
        // .add_systems(FixedUpdate, ((player_movement).in_set(GameplaySet),))
        .run();
}

#[derive(Component)]
pub struct Health {
    hp: u32,
}

impl Default for Health {
    fn default() -> Self {
        Health { hp: 100 }
    }
}

#[derive(Component, Default)]
pub struct Player;

pub fn setup_player(mut commands: Commands) {
    const FOV: f32 = f32::to_radians(60.0);

    commands
        .spawn((
            Player::default(),
            Health { hp: 90 },
            Transform::from_xyz(0.0, 5.0, 0.0),
            Visibility::default(),
            Collider::round_cylinder(0.9, 0.3, 0.2),
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
                snap_to_ground: None,
                ..default()
            },
        ))
        .with_children(|b| {
            // FPS Camera
            b.spawn((
                Camera3d::default(), 
                Transform::from_xyz(0.0, 0.2, -0.1), 
                Projection::Perspective(
                    PerspectiveProjection 
                    { 
                        fov: FOV, 
                        ..Default::default()
                    }
                )
            ));
        });
}

// todo: change this to get in a scene map
pub fn spawn_level_map(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    lvl_handles: Res<LevelHandles>,
) {
    let lvl_mesh_handle = &lvl_handles.0[0];
    let lvl_mesh = meshes
        .get(lvl_mesh_handle)
        .expect("Could not find level mesh");

    let lvl_collider = Collider::from_bevy_mesh(
        lvl_mesh,
        &ComputedColliderShape::TriMesh(TriMeshFlags::all()),
    );
    match lvl_collider {
        Some(collider) => {
            commands.spawn((
                Mesh3d(lvl_mesh_handle.clone()),
                MeshMaterial3d(materials.add(Color::WHITE)),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Name::new("Level Mesh"),
                Visibility::Visible, // ALERT: change this back to visible after finished with testing navmesh
                collider,
            ));
        }
        _ => {
            panic!("Could not generate collider for level mesh");
        }
    }
}