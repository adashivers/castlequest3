use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{actor::{Health, Player, animations::HurtBox}, debris::Invincible};

pub static IFRAMETIME: f32 = 0.5;

pub fn use_attack_collisions(
    mut collision_events: MessageReader<CollisionEvent>,
    mut health_query: Query<(Entity, &mut Health), With<Player>>,
    invincible_query: Query<&Invincible>,
    hurt_collider_query: Query<&HurtBox>,
    mut commands: Commands,
) {
    if !collision_events.is_empty() {
        debug!("checking collisions!");
    }
    
    for collision_event in collision_events.read() {
        debug!("collided!");
        match collision_event {
            CollisionEvent::Started(from, to, flags) => {
                // hurt collider's entity contains a collider
                if flags.contains(CollisionEventFlags::SENSOR) {
                    // we don't know which entity is which, so we check both options.
                    let mut hurt_result = hurt_collider_query.get(*from);
                    let mut health_result = health_query.get_mut(*to);
                    if hurt_result.is_err() || health_result.is_err() {
                        hurt_result = hurt_collider_query.get(*to);
                        health_result = health_query.get_mut(*from);
                        if hurt_result.is_err() || health_result.is_err() { continue };
                    }

                    // damage player if not currently in iframes
                    let (player_entity, mut health) = health_result.unwrap();
                    if !invincible_query.contains(player_entity) {
                        let dmg = hurt_result.unwrap().0;
                        commands.entity(player_entity).insert(Invincible(Timer::from_seconds(IFRAMETIME, TimerMode::Once)));
                        health.hp = health.hp - dmg;
                        debug!("took {} dmg, health down to {}", dmg, health.hp);
                        break; // we want the player to be hit only once per frame
                    }

                    
                }
            },
            _ => {}
        }
    }

}