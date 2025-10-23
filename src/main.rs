use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<DidFixedTimestepRunThisFrame>()
        .add_systems(Startup, (setup_physics, spawn_player))
        // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(PreUpdate, clear_fixed_timestep_flag)
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
        .add_systems(FixedUpdate, move_player)
        .add_systems(
            RunFixedMainLoop,
            (
                accumulate_input.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
                clear_input
                    .run_if(did_fixed_timestep_run_this_frame)
                    .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop),
            ),
        )
        .run();
}

/// A vector representing the player's input, accumulated over all frames that ran
/// since the last time the physics simulation was advanced.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct AccumulatedInput {
    // The player's movement input (WASD).
    movement: Vec2,
    // Other input that could make sense would be e.g.
    // boost: bool
}

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec3); // speed in units per second

#[derive(Bundle)]
struct PlayerInputBundle {
    accumulated_input: AccumulatedInput,
    velocity: Velocity,
}

impl Default for PlayerInputBundle {
    fn default() -> Self {
        Self {
            accumulated_input: AccumulatedInput::default(),
            velocity: Velocity::default(),
        }
    }
}

fn setup_physics(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // insert ground
    commands.spawn((
        Transform::from_translation(Vec3::ZERO),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        RigidBody::Fixed,
        Collider::cuboid(15.0, 0.1, 15.0),
    ));

    // insert random ball
    commands.spawn((
        Transform::from_xyz(1.0, 1.0, 0.0),
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb_u8(50, 50, 255))),
        RigidBody::Dynamic,
        Collider::ball(0.5),
    ));
}

fn spawn_player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // insert player
    commands
        .spawn((
            Name::new("Player"),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(0.5, 0.5),
            PlayerInputBundle {
                ..PlayerInputBundle::default()
            },
            KinematicCharacterController {
                ..KinematicCharacterController::default()
            },
        ))
        .with_children(|parent| {
            // spawn camera following player
            parent.spawn((
                Camera3d::default(),
                Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });
}

fn accumulate_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut AccumulatedInput, &mut Velocity)>,
) {
    const SPEED: f32 = 4.0;
    let (mut input, mut velocity) = player.into_inner();

    // reset the input to zero before reading the new input.
    input.movement = Vec2::ZERO;
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        input.movement.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        input.movement.x += 1.0;
    }

    // remap 2D input to bevy world coordinates
    // -Z is forward in Bevy, so that maps to W
    let input_3d = Vec3 {
        x: input.movement.x,  // left - right movement (strafing)
        y: 0.0,               // no upwards or downwards movement
        z: -input.movement.y, // forward movement along -Z
    };

    velocity.0 = input_3d.clamp_length_max(1.0) * SPEED;
}

/// A simple resource that tells us whether the fixed timestep ran this frame.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct DidFixedTimestepRunThisFrame(bool);

/// Reset the flag at the start of every frame.
fn clear_fixed_timestep_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = false;
}

/// Set the flag during each fixed timestep.
fn set_fixed_time_step_flag(
    mut did_fixed_timestep_run_this_frame: ResMut<DidFixedTimestepRunThisFrame>,
) {
    did_fixed_timestep_run_this_frame.0 = true;
}

fn did_fixed_timestep_run_this_frame(
    did_fixed_timestep_run_this_frame: Res<DidFixedTimestepRunThisFrame>,
) -> bool {
    did_fixed_timestep_run_this_frame.0
}

// Clear the input after it was processed in the fixed timestep.
fn clear_input(mut input: Single<&mut AccumulatedInput>) {
    **input = AccumulatedInput::default();
}

fn move_player(
    fixed_time: Res<Time<Fixed>>,
    velocity: Single<&Velocity>,
    mut controller: Single<&mut KinematicCharacterController>,
) {
    controller.translation = Some(velocity.0 * fixed_time.delta_secs());
}
