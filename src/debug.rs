use bevy::{
    prelude::*,
    color::palettes::{css::{BLACK, WHITE}},
};
use super::Player;
use bevy_landmass::debug::EnableLandmassDebug;
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