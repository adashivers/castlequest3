use bevy::{prelude::*};

use bevy_landmass::{Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, FromAgentRadius, Island};
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use bevy_rerecast::{Navmesh, generator::NavmeshGenerator,NavmeshSettings};

// populated during initial load
#[derive(Resource, Default)]
pub struct CurrNavmesh(pub Handle<Navmesh>);


pub fn generate_navmesh(
  mut commands: Commands,
  mut generator: NavmeshGenerator,

) {
  let agent_radius = 0.3;
  let agent_height = 1.8;
  let settings = NavmeshSettings::from_agent_3d(agent_radius, agent_height);
  let navmesh_handle = generator.generate(settings);

  commands.insert_resource(CurrNavmesh(navmesh_handle));
}

pub fn setup_archipelago(
  mut commands: Commands,
  curr_navmesh: Res<CurrNavmesh>,
) {
    // spawn archipelago
    let archipelago_id = commands
        .spawn(
            Archipelago3d::new(
                ArchipelagoOptions::from_agent_radius(0.3)
            )
        )
        .id();

    // spawn island
    // we need currnavmesh here, so this system should run right after first load is over
    commands
        .spawn(Island3dBundle {
            island: Island,
            archipelago_ref: ArchipelagoRef3d::new(archipelago_id),
            nav_mesh: NavMeshHandle3d(curr_navmesh.0.clone()),
        });
}

