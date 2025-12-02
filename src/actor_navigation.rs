use bevy::{prelude::*};

use bevy_landmass::{Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island, PointSampleDistance3d};
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use bevy_rerecast::{Navmesh, generator::NavmeshGenerator,NavmeshSettings};

use crate::player_movement;

// populated during initial load
#[derive(Resource, Default)]
pub struct CurrNavmesh(pub Handle<Navmesh>);


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
    psd.distance_above = 0.3f32;

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

