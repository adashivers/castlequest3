use std::fmt::Debug;
use crate::debris::DebrisPlugin;
use crate::loading_system::{GameScenes, LoadingSystemPlugin};
use crate::player_movement::PlayerMovementPlugin;
use crate::actor::navigation::{ArchipelagoSetup, NavmeshGenerating, NavmeshGenerators};
use crate::debug::{CQ3DebugPlugin};
use crate::actor::{EnemySpawnPlugin, Player, Health};
use crate::ui::UIPlugin;

use bevy::window::CursorGrabMode;
use bevy::window::CursorOptions;
use bevy::{prelude::*, log::LogPlugin};
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
use bevy::color::palettes::basic::GRAY;

use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use bevy_landmass::{prelude::*};

pub mod loading_system;
pub mod player_movement;
pub mod ui;
pub mod debug;
pub mod actor;
pub mod utils;
pub mod debris;

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(
            0xF9 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            0xFF as f32 / 255.0,
        )))
        // external plugins
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                filter: "moving_around=debug,wgpu_core=warn,wgpu_hal=warn".into(),
                level: bevy::log::Level::INFO,
                custom_layer: |_| None,
                ..Default::default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RemotePlugin::default(), RemoteHttpPlugin::default(),
        ))
        .insert_state(MyAppState::Loading)
        // internal plugins
        .add_plugins((
            CQ3DebugPlugin,
            LoadingSystemPlugin,
            PlayerMovementPlugin,
            EnemySpawnPlugin,
            UIPlugin,
            DebrisPlugin,
        ))
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
        .add_systems(OnExit(MyAppState::Loading), (
            spawn_level_map, 
            setup_player.after(ArchipelagoSetup),
        )
        )
        .add_systems(Update, (
            grab_mouse.in_set(GameplaySet),
        ))
        .run();
}

pub fn setup_player(
    mut commands: Commands,
    island_archipelago_ref: Query<&mut ArchipelagoRef3d, With<Island>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    const FOV: f32 = f32::to_radians(60.0);
    let archipelago_ref = ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity);

    commands
        .spawn((
            Name::new("Player"),
            Player::default(),
            Health { hp: 90 },
            Transform::from_xyz(0.0, 5.0, 0.0),
            Visibility::default(),
            Collider::round_cylinder(0.7, 0.1, 0.0),
            CollisionGroups::new(Group::GROUP_1, Group::GROUP_2),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
            Mesh3d(meshes.add(Cylinder::new(0.1, 1.4))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: GRAY.into(),
                ..default()
            })),
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
                Transform::from_xyz(0.0, -0.8, 0.0),
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
    scenes: Res<GameScenes>,
    navmesh_generators: Res<NavmeshGenerators>,
) {
    navmesh_generators.0
        .iter()
        .for_each(|gen_mesh_handle| {
            let gen_mesh = meshes
                .get(gen_mesh_handle.id())
                .expect("couldn't get mesh for generating collider/navmesh");
            let collider_option = Collider::from_bevy_mesh(
                gen_mesh, 
                &ComputedColliderShape::TriMesh(TriMeshFlags::all())
            );
            match collider_option {
                Some(collider) => {
                    commands.spawn((
                        Name::new("Level Collider"),
                        Mesh3d(gen_mesh_handle.clone()),
                        MeshMaterial3d(materials.add(Color::BLACK)), // adding for debug purposes
                        Transform::from_xyz(0.0, 0.0, 0.0),
                        Visibility::Hidden,
                        NavmeshGenerating,
                        collider,
                    ));
                }
                _ => {
                    panic!("Could not generate collider");
                }
            }

        });
    
    scenes.0
        .iter()
        .for_each(|scene| {
            commands.spawn((Name::new("Scene"), SceneRoot(scene.clone())));
        });
    
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
// copied almost directly from bevy example mouse_grab.
fn grab_mouse(
    mut cursor_options_q: Query<&mut CursorOptions>,
    key: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let mut cursor_options = cursor_options_q.single_mut().unwrap();

    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}
