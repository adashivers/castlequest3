use bevy::{prelude::*};
use bevy_rapier3d::prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController};
use super::{Health, Player};
use bevy_landmass::{Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentTarget3d, Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, Character, Character3dBundle, CharacterSettings, FromAgentRadius, Island, NavMesh3d};
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

use bevy_rerecast::{Navmesh as RerecastNavMesh};



// populated during initial load
#[derive(Resource, Default)]
pub struct CurrNavmesh(pub Handle<RerecastNavMesh>);

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

#[derive(Component, Default)]
pub struct Enemy;

pub fn spawn_enemy(
  mut commands: Commands,
  archipelago_query: Query<(Entity, &Archipelago3d)>,
  player_query: Query<(Entity, &Player)>,
) {
  let archipelago_entity = archipelago_query.single().expect("Cound not find single archipelago entity").0;
  let player_entity = player_query.single().expect("Could not find player while spawning enemy").0;

  commands.spawn((
    Enemy::default(),
    Health { hp: 100 },
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
    Agent3dBundle {
      agent: default(),
      settings: AgentSettings {
        radius: 0.3,
        desired_speed: 1.0,
        max_speed: 2.0,
      },
      archipelago_ref: ArchipelagoRef3d::new(archipelago_entity),
    },
    AgentTarget3d::Entity(player_entity),
  ));
}

pub fn move_enemy(
  enemies: Query<(&mut KinematicCharacterController, &AgentDesiredVelocity3d), With<Enemy>>,
  time: Res<Time>,
) {
  for (mut controller, desired_velocity) in enemies {
    let mut next_velocity: Vec3 = Vec3::new(0.0, -9.8, 0.0);
    debug!("enemy desired velocity: {}", desired_velocity.velocity());
    next_velocity += desired_velocity.velocity();
    controller.translation = Some(next_velocity * time.delta_secs());
  }
}
