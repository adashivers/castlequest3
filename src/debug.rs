use bevy::{
    prelude::*,
    color::palettes::css::*,
};
use super::Player;
use bevy_landmass::{AgentState, debug::EnableLandmassDebug};
use bevy_rapier3d::{render::DebugRenderContext};
use crate::ui::BrosOskonFont;

// edit to change initial flags for debug
pub const SHOW_COLLIDERS: bool = false;
pub const SHOW_NAVMESH: bool = true;

/// Debug input vector. Debug systems should only run if this resource exists.
#[derive(Resource)]
pub struct DebugFlags {
    pub show_colliders: bool,
    pub show_navmesh: bool
}

impl Default for DebugFlags {
    fn default() -> Self {
        DebugFlags { 
            show_colliders: SHOW_COLLIDERS,
            show_navmesh: SHOW_NAVMESH,
        }
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AgentStateGizmos; // Gizmos showing agent state

pub fn draw_agent_state_gizmos(
    mut agent_gizmos: Gizmos<AgentStateGizmos>,
    agent_query: Query<(&AgentState, &GlobalTransform)>,
) {
    agent_query
    .iter()
    .for_each(|(agent_state, transform)| {
        let isometry = transform.to_isometry();
        let color = match agent_state {
            AgentState::Idle => GRAY,
            AgentState::AgentNotOnNavMesh => RED,
            AgentState::TargetNotOnNavMesh => MAROON,
            AgentState::Moving => GREEN,
            AgentState::ReachedTarget => LIME,
            AgentState::NoPath => BLACK,
            AgentState::Paused => SILVER,
            AgentState::ReachedAnimationLink => TEAL,
            AgentState::UsingAnimationLink => BLUE,
        };
        agent_gizmos.sphere(isometry, 0.2f32, color);
    });
}

pub fn update_gizmo_configs(
    mut config_store: ResMut<GizmoConfigStore>,
    debug_flags: Option<Res<DebugFlags>>
) {
    let (config, _) = config_store.config_mut::<AgentStateGizmos>();
    match debug_flags {
        Some(flags) => {
            config.enabled = flags.show_navmesh;
            config.line.width = 7.0;
            config.depth_bias = -0.5;
        },
        _ => {
            config.enabled = false;
        }
    }
}

#[derive(Component, Default)]
pub struct DebugCollidersVisibleText;
#[derive(Component, Default)]
pub struct DebugNavmeshVisibleText;
#[derive(Component, Default)]
pub struct DebugPlayerPositionText;
#[derive(Component, Default)]
pub struct DebugNavmeshDisplay;

pub fn handle_debug_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    debug_flags: Option<ResMut<DebugFlags>>,
    debug_render_context: Option<ResMut<DebugRenderContext>>,
    mut landmass_debug: ResMut<EnableLandmassDebug>
) {
    // set debug input stuff
    match debug_flags {
        Some(mut flags) => {
            if keyboard.just_pressed(KeyCode::KeyT) {
                let next = !flags.show_colliders;
                debug!("show_colliders set to {}", next);
                debug_render_context.unwrap().enabled = next;
                flags.show_colliders = next;
            }
            if keyboard.just_pressed(KeyCode::KeyN) {
                let next = !flags.show_navmesh;
                debug!("show_navmesh set to {}", next);
                landmass_debug.0 = next;
                flags.show_navmesh = next;
            }
        },
        _ => {}
    }
}

pub fn setup_debug_ui(
    mut commands: Commands,
    debug_flags: Option<ResMut<DebugFlags>>,
    font: Res<BrosOskonFont>
) {

    // set up some debug stuff
    match debug_flags {
        Some(flags) => {
            commands
                .spawn((
                    Text::default(),
                    TextFont {
                        font: font.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextColor(BLACK.into()),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Percent(50.0),
                        ..default()
                    },
                    BackgroundColor(WHITE.into()),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        TextSpan::new("Player position: "),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                    ));
                    builder.spawn((
                        TextSpan::new(""),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                        DebugPlayerPositionText,
                    ));
                    builder.spawn((
                        TextSpan::new("\n(T) Show colliders: "),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                    ));
                    builder.spawn((
                        TextSpan::new(flags.show_colliders.to_string()),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                        DebugCollidersVisibleText
                    ));
                    builder.spawn((
                        TextSpan::new("\n(N) Show navmesh: "),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                    ));
                    builder.spawn((
                        TextSpan::new(flags.show_navmesh.to_string()),
                        TextFont {
                            font: font.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(BLACK.into()),
                        DebugNavmeshVisibleText
                    ));
                });
        },

        None => {}
    }
}

pub fn update_debug_ui(
    debug_flags: Option<ResMut<DebugFlags>>,
    player_query: Query<&Transform, With<Player>>,
    mut debug_text_query_set: ParamSet<(
        Query<&mut TextSpan, With<DebugCollidersVisibleText>>,
        Query<&mut TextSpan, With<DebugNavmeshVisibleText>>,
        Query<&mut TextSpan, With<DebugPlayerPositionText>>
    )>,
) {
    // update debug text
    match debug_flags {
        Some(flags) => {
            for mut span in debug_text_query_set.p0().iter_mut() {
                **span = flags.show_colliders.to_string();
            }
            for mut span in debug_text_query_set.p1().iter_mut() {
                **span = flags.show_navmesh.to_string();
            }
            for mut span in debug_text_query_set.p2().iter_mut() {
                let player_translation = player_query.single().unwrap().translation;
                let player_translation = (player_translation * 100.0).trunc() / 100.0; // truncate to 2 decimal pts to reduce visual clutter
                **span = player_translation.to_string();
            }
        }
        None => {}
    }
}