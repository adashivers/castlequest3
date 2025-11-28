use std::fmt::Debug;
use crate::actor_navigation::generate_navmesh;
use crate::actor_navigation::move_enemy;
use crate::actor_navigation::setup_archipelago;
use crate::actor_navigation::spawn_enemy;
use crate::loading_system::*;
use crate::player_movement::*;
use crate::ui::*;
use crate::actor_navigation::{CurrNavmesh};
use crate::debug::*;

use bevy::{input::InputSystems, prelude::*, log::LogPlugin};
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};

use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use bevy_landmass::{prelude::*, debug::Landmass3dDebugPlugin};

use bevy_rerecast::{prelude::*, Mesh3dBackendPlugin};
use landmass_rerecast::LandmassRerecastPlugin;

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
        .init_resource::<DebugFlags>() // remove this to disable debug stuff completely
        .init_resource::<CurrNavmesh>()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "moving_around=debug,wgpu_core=warn,wgpu_hal=warn".into(),
                level: bevy::log::Level::INFO,
                custom_layer: |_| None,
                ..Default::default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin{
                enabled: SHOW_COLLIDERS,
                ..Default::default()
            },
            RemotePlugin::default(), RemoteHttpPlugin::default(),
            Landmass3dPlugin::default(),
            Landmass3dDebugPlugin {
                draw_on_start: SHOW_NAVMESH,
                ..Default::default()
            },
            LandmassRerecastPlugin::default(),
            NavmeshPlugins::default(),
            Mesh3dBackendPlugin::default(),
            
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
            (
                setup_ui, 
                setup_debug_ui.run_if(resource_exists::<DebugFlags>)
            ).chain(),
        ))
        .add_systems(OnExit(MyAppState::Loading), (spawn_level_map, (generate_navmesh, setup_archipelago, setup_player, spawn_enemy).chain()))
        .add_systems(
            PreUpdate,
            ((handle_input, handle_debug_input.run_if(resource_exists::<DebugFlags>), player_movement)
                .chain()
                .after(InputSystems)
                .in_set(GameplaySet),),
        )
        .add_systems(
            Update,
            (
                (
                    player_look, 
                    move_enemy, 
                    update_ui, 
                    update_debug_ui.run_if(resource_exists::<DebugFlags>)
                ).in_set(GameplaySet),
                (checks_assets_loaded).in_set(LoadingSet),
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

pub fn setup_player(
    mut commands: Commands,
    island_archipelago_ref: Query<&mut ArchipelagoRef3d, With<Island>>,
) {
    const FOV: f32 = f32::to_radians(60.0);
    let archipelago_ref = ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity);

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
            b.spawn((
                Transform::from_xyz(0.0, -0.9, 0.0),
                Character3dBundle {
                    character: default(),
                    settings: CharacterSettings {
                        radius: 0.3
                    },
                    archipelago_ref,
                }
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
                MeshMaterial3d(materials.add(Color::BLACK)),
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