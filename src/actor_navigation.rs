use crate::{debug::{DebugFlags, DebugNavmeshDisplay}, loading_system::AssetsLoading};
use bevy::{color::palettes, math::VectorSpace, prelude::*};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use vleue_navigator::{NavMesh};

pub const GRAVITY: f32 = -9.81;
const ENEMY_SPEED: f32 = 100.;
const ENEMY_VISION_RADIUS: f32 = 300.0;

#[derive(Resource)]
pub struct CurrentNavMesh(Handle<NavMesh>);

#[derive(Default, Resource)]
pub struct NavMeshPrimitives(Vec<Handle<Mesh>>);

pub fn start_loading_navmesh
(
    mut loading: ResMut<AssetsLoading>,
    mut nav_mesh_prim: ResMut<NavMeshPrimitives>,
    asset_server: Res<AssetServer>
)
{
    let level_navmesh: Handle<Mesh> =
        asset_server.load("models/dungeon_navmesh.glb#Mesh0/Primitive0");

    // this adds to AssetsLoading, defined in loading_system. Thus, this entire system must run during startup.
    // (at least until the loading system has functionality to rerun initializing after startup)
    // TODO: run this through an event that gets sent to loading_system
    loading.0.push(level_navmesh.clone().untyped());
    nav_mesh_prim.0.push(level_navmesh);

    
}

pub fn spawn_navmesh(
    mut commands: Commands,
    mut nav_mesh_prim: ResMut<NavMeshPrimitives>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    debug_flags: Option<ResMut<DebugFlags>>,
) {
    // navmesh primitive must be loaded when this is called
    // navmesh primitive is loaded in startup, so this should best run in update or at the start of the ingame state of the app
    let mesh = meshes.get(nav_mesh_prim.0[0].id()).expect("Couldn't get navmesh for the level");
    let navmesh = NavMesh::from_bevy_mesh(mesh).unwrap();

    let mut material: StandardMaterial = Color::Srgba(palettes::css::DARK_BLUE).into();
    material.unlit = true;

    // set up navmesh display for debugging
    match debug_flags {
        Some(flags) => {
            commands.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                MeshMaterial3d(materials.add(material)),
                Name::new("Debug NavMesh Display"),
                match flags.show_navmesh {
                    true => Visibility::Visible,
                    _ => Visibility::Hidden
                },
                Mesh3d(meshes.add(navmesh.to_wireframe_mesh())),
                DebugNavmeshDisplay,
            ));
        },
        _ => {}
    }

    let navmesh_handle = navmeshes.add(navmesh);
    commands.insert_resource(CurrentNavMesh(navmesh_handle));
    

    // todo: might not fully clear here?
    // drops the nav mesh primitive used here to save memory
    nav_mesh_prim.0.clear();
}

#[derive(Component)]
pub struct Enemy;

pub fn spawn_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut material: StandardMaterial = Color::Srgba(palettes::css::DARK_BLUE).into();
    material.unlit = true;
    commands.spawn((
        Enemy,
        Transform::from_xyz(0.0, 5.0, 0.0),
        MeshMaterial3d(materials.add(material)),
        Mesh3d(meshes.add(Capsule3d::default())),
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

    ));
}

pub fn update_enemy(
    time: Res<Time>,
    mut enemy_query: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        With<Enemy>,
    >,
) {
    for (_, mut controller, _) in enemy_query.iter_mut() {
        let mut next_translation: Vec3 = Vec3::ZERO;

        // calculate translation
        // TODO: add pathfinding
        next_translation.y += GRAVITY * time.delta_secs() * controller.custom_mass.unwrap_or(1.0);

        controller.translation = Some(next_translation);
    }
}