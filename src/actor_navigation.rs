use std::fmt::Debug;

use bevy::{prelude::*};
use bevy_rapier3d::prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController};
use super::{Health};
use bevy_landmass::{Agent3dBundle, AgentDesiredVelocity3d, AgentSettings, AgentState, AgentTarget3d, Archipelago3d, ArchipelagoOptions, ArchipelagoRef3d, Character, FromAgentRadius, Island, TargetReachedCondition, coords::ThreeD};
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

#[derive(Component, Default)]
pub struct Enemy;

pub fn spawn_enemy(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  island_archipelago_ref: Query<&mut ArchipelagoRef3d, With<Island>>,
  player_query: Query<(Entity, &Character<ThreeD>)>,
) {
  let archipelago_ref = ArchipelagoRef3d::new(island_archipelago_ref.single().expect("Cound not find archipelago reference on island").entity);
  let player_entity = player_query.single().expect("Could not find player while spawning enemy").0;

  commands.spawn((
    Enemy::default(),
    Health { hp: 100 },
    Transform::from_xyz(-35.0, 10.0, -10.0),
    Visibility::default(),
    Collider::round_cylinder(0.8, 0.3, 0.2),
    Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
    MeshMaterial3d(materials.add(StandardMaterial::default())),
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
    children![(
      Transform::from_xyz(0.0, -0.9, 0.0),
      Agent3dBundle {
        agent: default(),
        settings: AgentSettings {
          radius: 0.3,
          desired_speed: 3.0,
          max_speed: 5.0,
        },
        archipelago_ref,
      },
      AgentTarget3d::Entity(player_entity),
      TargetReachedCondition::Distance(Some(2.0))
    )]
  ));
}

pub fn move_enemy(
  mut enemy_controllers: Query<&mut KinematicCharacterController, With<Enemy>>,
  enemy_agents: Query<(&ChildOf, &AgentState, &AgentDesiredVelocity3d)>,
  time: Res<Time>,
) {
  for (parent, agent_state, desired_velocity) in enemy_agents {
    let mut controller = enemy_controllers.get_mut(parent.parent()).expect("could not find enemy character controller");
    let mut next_velocity: Vec3 = Vec3::new(0.0, -9.8, 0.0);
    debug!("agent state: {:?}", agent_state);
    next_velocity += desired_velocity.velocity();
    controller.translation = Some(next_velocity * time.delta_secs());
  }
}
