use crate::{debug::{DebugFlags, DebugNavmeshDisplay}, loading_system::AssetsLoading};
use super::Player;
use bevy::{asset::weak_handle, color::palettes, prelude::*};
use bevy_rapier3d::{control::KinematicCharacterController, prelude::*};
use bevy_behave::prelude::*;
use vleue_navigator::{NavMesh, NavMeshDebug, prelude::ManagedNavMesh};

const ENEMY_SPEED: f32 = 100.;
const ENEMY_VISION_RADIUS: f32 = 300.0;

#[derive(Component, Default)]
#[require(Transform)]
pub struct Enemy;

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
    meshes: Res<Assets<Mesh>>,
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

    let navmesh_handle = navmeshes.add(navmesh);
    commands.insert_resource(CurrentNavMesh(navmesh_handle));

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
                Mesh3d(nav_mesh_prim.0[0].clone()),
                DebugNavmeshDisplay,
            ));
        },
        _ => {}
    }
    

    // todo: might not fully clear here?
    // drops the nav mesh primitive used here to save memory
    nav_mesh_prim.0.clear();
}