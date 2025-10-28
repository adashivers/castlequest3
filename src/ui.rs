use super::{Health, Player};
use bevy::{
    color::palettes::tailwind::{RED_700, VIOLET_700},
    prelude::*,
};

#[derive(Component, Default)]
pub struct HealthText;

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    health_query: Query<&Health, With<Player>>,
) {
    let player_hp: u32 = match health_query.single() {
        Ok(Health { hp }) => *hp,
        Err(err) => {
            debug!("Health UI found no player health: {}", err);
            0
        }
    };

    let bros_oskon_90s_extltita =
        asset_server.load("fonts/zt_bros_oskon_90s/ZTBrosOskon90s-ExtLtIta.otf");

    commands
        .spawn((
            Text::new("HEALTH: "),
            TextFont {
                font: bros_oskon_90s_extltita.clone(),
                font_size: 275.0, // make it fucking big
                ..default()
            },
            TextColor(VIOLET_700.into()),
        ))
        .with_child((
            TextSpan::new(player_hp.to_string()),
            TextFont {
                font: bros_oskon_90s_extltita,
                font_size: 275.0,
                ..default()
            },
            TextColor(RED_700.into()),
            HealthText,
        ));
}

pub fn update_ui(
    health_query: Query<&Health, With<Player>>,
    mut hp_text_query: Query<&mut TextSpan, With<HealthText>>,
) {
    let player_hp: u32 = match health_query.single() {
        Ok(Health { hp }) => *hp,
        Err(err) => {
            debug!("Health UI found no player health: {}", err);
            0
        }
    };

    for mut span in &mut hp_text_query {
        **span = player_hp.to_string();
    }
}
