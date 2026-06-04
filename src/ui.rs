use super::{Health, Player};
use bevy::{
    color::palettes::{tailwind::*},
    prelude::*,
    
};

#[derive(Component, Default)]
pub struct HealthText;

#[derive(Resource, Deref, DerefMut)]
pub struct BrosOskonFont(Handle<Font>);

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, 
            setup_ui.after(crate::loading_system::start_loading_assets)
        )
        .add_systems(Update,
            update_ui
            .after(crate::player_movement::player_look)
            .in_set(super::GameplaySet)
        );
    }
}


pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    health_query: Query<&Health, With<Player>>,
) {
    let player_hp: i32 = match health_query.single() {
        Ok(Health { hp }) => *hp,
        Err(err) => {
            debug!("Health UI found no player health: {}", err);
            0
        }
    };

    // load font
    let bros_oskon_90s_extltita =
        asset_server.load("fonts/zt_bros_oskon_90s/ZTBrosOskon90s-ExtLtIta.otf");

    // set up health ui
    commands
        .spawn((
            Text::new("HEALTH: "),
            TextFont {
                font: bros_oskon_90s_extltita.clone(),
                font_size: 50.0, // make it fucking big
                ..default()
            },
            TextColor(VIOLET_700.into()),
        ))
        .with_child((
            TextSpan::new(player_hp.to_string()),
            TextFont {
                font: bros_oskon_90s_extltita.clone(),
                font_size: 50.0,
                ..default()
            },
            TextColor(RED_700.into()),
            HealthText,
        ));
    
    // note: debug::setup_debug_ui searches for this resource, so run this system before running that one.
    commands.insert_resource(BrosOskonFont(bros_oskon_90s_extltita));
    
}

pub fn update_ui(
    health_query: Query<&Health, With<Player>>,
    
    mut hp_text_query: Query<&mut TextSpan, With<HealthText>>,
    
) {
    // update hp text
    let player_hp: i32 = match health_query.single() {
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


