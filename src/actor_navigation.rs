use crate::loading_system::AssetsLoading;
use super::Player;
use bevy::{asset::weak_handle, color::palettes, prelude::*};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use bevy_behave::prelude::*;
use vleue_navigator::NavMesh;

const HANDLE_NAVMESH: Handle<NavMesh> = weak_handle!("2bee303e-d39f-479f-8fd3-20babb822ddb");
const ENEMY_SPEED: f32 = 100.;
const ENEMY_VISION_RADIUS: f32 = 300.0;

#[derive(Component, Default)]
#[require(Transform)]
#[require(VisionRadius(ENEMY_VISION_RADIUS))]
pub struct Enemy;

#[derive(Component, Clone)]
struct NavMeshDisplay(Handle<NavMesh>);

#[derive(Resource)]
pub struct CurrentNavMesh(Handle<NavMesh>);

#[derive(Default, Resource)]
pub struct NavMeshPrimitives(Vec<Handle<Mesh>>);

pub fn start_loading_navmesh
(
    mut commands: Commands,
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

    commands.insert_resource(CurrentNavMesh(HANDLE_NAVMESH));
}

pub fn spawn_navmesh(
    mut commands: Commands,
    mut nav_mesh_prim: ResMut<NavMeshPrimitives>,
    meshes: Res<Assets<Mesh>>,
    mut navmeshes: ResMut<Assets<NavMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // navmesh must be loaded when this is called
    // navmesh is loaded in startup, so this should best run in update or at the start of the ingame state of the app
    let mesh = meshes.get(nav_mesh_prim.0[0].id()).expect("Couldn't get navmesh for the level");
    let navmesh = NavMesh::from_bevy_mesh(mesh).unwrap();

    let mut material: StandardMaterial = Color::Srgba(palettes::css::ANTIQUE_WHITE).into();
    material.unlit = true;


    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        MeshMaterial3d(materials.add(material)),
        Name::new("Level NavMesh"),
        Visibility::Visible,
        NavMeshDisplay(HANDLE_NAVMESH),
    ));
    navmeshes.insert(&HANDLE_NAVMESH, navmesh);
    

    // todo: might not fully clear here?
    // drops the nav mesh primitive used here to save memory
    nav_mesh_prim.0.clear();
}

#[derive(Component, Clone)]
struct WaitUntilPlayerIsNear {
    player: Entity,
}

#[derive(Component, Clone)]
struct MoveTowardsPlayer {
    player: Entity,
    speed: f32,
}

#[derive(Component)]
struct VisionRadius(f32);


// --- !!! WIP !!! ---
pub fn move_actors(
    player: Single<Entity, With<Player>>,
    mut agents: Query<
        (
            &mut Transform,
            &mut KinematicCharacterController,
            Option<&KinematicCharacterControllerOutput>,
        ),
        Without<Player>,
    >,
    navmesh: Res<CurrentNavMesh>,
) {
    let tree = tree! {
        Behave::Forever => {
            Behave::Sequence => {
                 Behave::spawn_named("Wait until player is near",
                    WaitUntilPlayerIsNear{player: *player}
                ),
            },
            // found player near
            Behave::Sequence => {
                Behave::spawn_named("Move towards player while in range",
                    MoveTowardsPlayer{player: *player, speed: ENEMY_SPEED}
                ),
            },
        }
    };

    for (agent, controller, controller_output) in agents.iter() {

    }
}