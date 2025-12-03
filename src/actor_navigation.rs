use bevy::{prelude::*};

use bevy_landmass::{Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island, PointSampleDistance3d};
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use bevy_rerecast::{Navmesh, generator::NavmeshGenerator,NavmeshSettings};
use bevy_rerecast::rerecast::TriMesh;
use bevy_rerecast::{NavmeshApp as _, TriMeshFromBevyMesh};
use crate::player_movement;

// a list of mesh handles for the meshes we want to generate the navmesh with.
#[derive(Default, Resource)]
pub struct NavmeshGenerators(pub Vec<Handle<Mesh>>);

// populated during initial load
#[derive(Resource, Default)]
pub struct CurrNavmesh(pub Handle<Navmesh>);

// put this on an entity to generate a navmesh from its mesh
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct NavmeshGenerating;

fn navmeshgenerating_backend(
    input: In<NavmeshSettings>,
    meshes: Res<Assets<Mesh>>,
    obstacles: Query<(Entity, &GlobalTransform, &Mesh3d), With<NavmeshGenerating>>,
) -> TriMesh {
    obstacles
        .iter()
        .filter_map(|(entity, transform, mesh)| {
            if input
                .filter
                .as_ref()
                .is_some_and(|entities| !entities.contains(&entity))
            {
                return None;
            }
            let transform = transform.compute_transform();
            let mesh = meshes.get(mesh)?.clone().transformed_by(transform);
            TriMesh::from_mesh(&mesh)
        })
        .fold(TriMesh::default(), |mut acc, t| {
            acc.extend(t);
            acc
        })
}

#[derive(Debug, Default)]
#[non_exhaustive]
pub struct NavmeshGeneratingBackendPlugin;

impl Plugin for NavmeshGeneratingBackendPlugin {
    fn build(&self, app: &mut App) {
        app.set_navmesh_backend(navmeshgenerating_backend);
        app.register_type::<NavmeshGenerating>();
    }
}

pub fn generate_navmesh(
  mut commands: Commands,
  mut generator: NavmeshGenerator,

) {
  let agent_radius = 0.1;
  let agent_height = 1.8;
  let settings = NavmeshSettings::from_agent_3d(agent_radius, agent_height);
  let navmesh_handle = generator.generate(settings);

  commands.insert_resource(CurrNavmesh(navmesh_handle));
}

pub fn setup_archipelago(
  mut commands: Commands,
  curr_navmesh: Res<CurrNavmesh>,
) {
    
    // set archipelago options
    let mut archipelago_options = ArchipelagoOptions::from_agent_radius(0.3);
    let psd: &mut PointSampleDistance3d = &mut archipelago_options.point_sample_distance;
    // setting distance below to highest point a player jump can achieve, so that the navmesh keeps tracking while player is jumping
    psd.distance_below = player_movement::highest_point();
    psd.distance_above = 0.5f32;

    // spawn archipelago
    let archipelago_id = commands.spawn(
      Archipelago3d::new(
          archipelago_options
      )
    ).id();

    // spawn island
    // we need currnavmesh here, so this system should run right after first load is over
    commands
        .spawn(Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
            nav_mesh: NavMeshHandle3d(curr_navmesh.0.clone()),
        });
}

