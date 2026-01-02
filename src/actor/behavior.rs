use std::collections::HashMap;

use bevy::ecs::{entity::Entity};
use bevy::prelude::*;
use bevy::color::palettes::basic::RED;
use bevy_behave::prelude::*;
use bevy_landmass::coords::ThreeD;
use bevy_landmass::{PathStep::Waypoint, Agent, AgentDesiredVelocity3d, AgentState, AgentTarget3d, Archipelago, Character, PointSampleDistance3d};
use bevy_rapier3d::prelude::KinematicCharacterController;
use crate::actor::{Player,};
use crate::debug::{CQ3DebugGizmos, DebugFlags};
use crate::utils::get_top_parent;

use super::ActorType;


// This should run every time an actor is added to the scene
pub fn init_actor_behavior(
    mut commands: Commands,
    player_query: Query<(Entity, &Children), With<Player>>,
    actor_type_query: Query<(Entity, &Children, &ActorType), Added<ActorType>>,
    agent_query: Query<Entity, With<Agent<ThreeD>>>,
    character_query: Query<Entity, With<Character<ThreeD>>>,
    debug_flags_query: Option<Res<DebugFlags>>,
) {
    let btree_logging = match debug_flags_query {
        Some(flags) => { flags.actor_behavior_tree_logs },
        _ => { false }
    };

    for (entity, children, actor_type) in actor_type_query {
        debug!("actortype component added to entity {}", entity.index());
        let agent_entity = children.iter().find(|x| {agent_query.get(*x).is_ok()});
        if agent_entity.is_none() {
            debug!("agent entity not found for actor!");
            continue;
        }

        let agent_entity = agent_entity.unwrap();

        let (tree, agent_target) = match actor_type {
            ActorType::Enemy { radius } => {
                debug!("setting up entity {}'s agent entity as an enemy", entity.index());
                let (_, player_children) = player_query.single().unwrap();
                // The navigation mesh Character entity is actually a parent of the top entity that makes up the player. We use this to get it:
                let player_char_entity = player_children.iter().find(|x| {character_query.get(*x).is_ok()}).unwrap();
                let tree = behave! {
                        Behave::Forever => {
                            Behave::Fallback => {
                                Behave::Sequence => {
                                    Behave::trigger(CheckEntityInSight { entity_from: agent_entity, entity_to: player_char_entity, radius: *radius }),
                                    Behave::trigger(MoveTowardsTarget { agent_entity: agent_entity, actor_entity: entity }),
                                },
                            // Behave::trigger(SwitchToIdling)
                        }
                    }
                };
                let target = AgentTarget3d::Entity(player_char_entity);
                (BehaveTree::new(tree).with_logging(btree_logging), target)
            }

            _ => {
                (
                    BehaveTree::new(behave! {
                        Behave::AlwaysFail
                    }),
                    AgentTarget3d::None
                )
            }
        };

        commands.get_entity(agent_entity).unwrap().insert((
            tree, agent_target
        ));
    }
    
}

#[derive(Clone)]
pub struct CheckEntityInSight { pub entity_from: Entity, pub entity_to: Entity, pub radius: f32 }


pub fn on_check_entity_in_sight(
	trigger: On<BehaveTrigger<CheckEntityInSight>>, 
	mut commands: Commands, 
    archipelago: Query<&Archipelago<ThreeD>>,
    global_transform: Query<&GlobalTransform>,
) {
    // TODO: this does not work! fix it using the implementation at
    // https://github.com/andriyDev/landmass/blob/3c12842f7620c60a710e8483a2b152229ef4b00c/crates/landmass/src/agent.rs#L343
	let ctx = trigger.ctx();
    let archipelago = archipelago.single().unwrap();

    let entity_from_pos = global_transform.get(trigger.inner().entity_from).unwrap().translation();
    let entity_to_pos = global_transform.get(trigger.inner().entity_to).unwrap().translation();
    
    let entity_from_pos_sampled = archipelago.sample_point(entity_from_pos, &archipelago.get_agent_options().point_sample_distance).unwrap();
    let entity_to_pos_sampled = archipelago.sample_point(entity_to_pos, &archipelago.get_agent_options().point_sample_distance).unwrap();

    let path = archipelago.find_path(&entity_from_pos_sampled, &entity_to_pos_sampled, &HashMap::new(), bevy_landmass::PermittedAnimationLinks::All).unwrap();
    if path.len() == 1 {
        commands.trigger(ctx.success());
    } else {
        commands.trigger(ctx.failure());
    }
}

#[derive(Clone)]
pub struct MoveTowardsTarget { pub agent_entity: Entity, pub actor_entity: Entity }

pub fn on_move_towards_target(
	trigger: On<BehaveTrigger<MoveTowardsTarget>>,
    agent_query: Query<(&AgentState, &AgentDesiredVelocity3d)>,
    mut commands: Commands,
	mut actor_query: Query<(&mut Transform, &mut KinematicCharacterController)>,
	time: Res<Time>
) {
	let ctx = trigger.ctx();
	let event = trigger.event().inner();
	let (mut transform, mut controller) = actor_query.get_mut(event.actor_entity).unwrap();

	if let Ok((agent_state, desired_velocity)) = agent_query.get(event.agent_entity) {
        debug!("agent state:{:?}\tdesired velocity: {}", agent_state, desired_velocity.velocity());
		if desired_velocity.velocity().length() > 0.1 {
			// align transform rotation so that agent looks where it's going
			transform.align(Dir3::X, desired_velocity.velocity().normalize(), Dir3::Y, Dir3::Y);
		}
		
		// set next velocity
		let mut next_velocity: Vec3 = Vec3::new(0.0, -0.1, 0.0); // slight downward tilt so that the collider snaps to the ground.
		next_velocity += desired_velocity.velocity(); // add desired velocity
		controller.translation = Some(next_velocity * time.delta_secs());
		commands.trigger(ctx.success());
	} else {
		commands.trigger(ctx.failure());
	}
	
}

#[derive(Clone)]
pub struct SwitchToIdling;
