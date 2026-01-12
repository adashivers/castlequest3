use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::{entity::Entity};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_landmass::coords::ThreeD;
use bevy_landmass::{AgentDesiredVelocity3d, AgentState, AgentTarget3d, Archipelago, Character, PointSampleDistance3d};
use bevy_rapier3d::prelude::{KinematicCharacterController};
use crate::actor::animations::*;
use crate::actor::{Player,};
use crate::debug::{DebugFlags};

use super::ActorType;

#[derive(Component)]
#[require(MoveAgentLastTick(false))]
// Actors will move towards their targets only if this component is on them, and set to true.
pub struct MoveAgent(pub bool);

#[derive(Component, Default)]

// component holding the state of a moveagent in the last tick.
// added automatically with MoveAgent, starting at false.
pub struct MoveAgentLastTick(pub bool);

// This should run every time an actor is added to the scene
pub fn init_actor_behavior(
    mut commands: Commands,
    player_query: Query<(Entity, &Children), With<Player>>,
    actor_type_query: Query<(Entity, &Children, &ActorType), Added<ActorType>>,
    agent_query: Query<Entity, With<AgentState>>,
    character_query: Query<Entity, With<Character<ThreeD>>>,
    debug_flags_query: Option<Res<DebugFlags>>,
    graphs: Res<Assets<AnimationGraph>>,
	animations: Res<Animations>,
	clips: ResMut<Assets<AnimationClip>>,
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
        let graph = graphs.get(animations.graph_handle.id()).unwrap();
        let attack_anim_node = graph.get(animations.animations[1]).unwrap();
        let clip = match &attack_anim_node.node_type {
            AnimationNodeType::Clip(clip_handle) => clips.get(clip_handle.id()),
            _ => unreachable!(),
        }.unwrap();

        let (tree, agent_target) = match actor_type {
            ActorType::Enemy { sight_radius, attack_radius } => {
                debug!("setting up entity {}'s agent entity as an enemy", entity.index());
                let (_, player_children) = player_query.single().unwrap();
                // The navigation mesh Character entity is actually a parent of the top entity that makes up the player. We use this to get it:
                let player_char_entity = player_children.iter().find(|x| {character_query.get(*x).is_ok()}).unwrap();
                let tree = behave! {
                        Behave::Forever => {
                            Behave::Fallback => {
                                // attack action
                                Behave::Sequence => {
                                    Behave::trigger(CheckEntityInSight { entity_from: agent_entity, entity_to: player_char_entity, radius: *attack_radius}),
                                    Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, do_move: false }),
                                    Behave::While => {
                                        Behave::trigger(CheckEntityInSight { entity_from: agent_entity, entity_to: player_char_entity, radius: *attack_radius}),
                                        Behave::spawn_named("Attack", (
                                            Attack { attacking_agent_entity: entity },
                                            BehaveTimeout::new(Duration::from_secs_f32(clip.duration()), true),
                                        )),
                                    }
                                    
                                },
                                // move cycle
                                Behave::Fallback => {
                                    // enemy in sight logic
                                    Behave::Sequence => {
                                        Behave::trigger(CheckEntityInSight { entity_from: agent_entity, entity_to: player_char_entity, radius: *sight_radius }),
                                        // if not already moving, start moving towards player
                                        Behave::Invert => {
                                            Behave::trigger(CheckMoving { agent_entity: agent_entity }), // TODO: implement
                                        },

                                        // TODO: implement on_set_agent_target_entity. 
                                        // when this and SetAgentTargetPosition are done, we can remove the functionality of this method that adds a target entity.
                                        // i.e. delete agent_target
                                        // Behave::trigger(SetAgentTargetEntity { agent_entity: agent_entity, char_entity: player_char_entity } ), 
                                        Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, do_move: true }),
                                    },

                                    // enemy not in sight but still targeted logic
                                    Behave::Sequence => {
                                         Behave::trigger(CheckMoving { agent_entity: agent_entity }),
                                    },

                                    // enemy not in sight logic
                                    // Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, do_move: true }),
                                    // Behave::trigger(SetAgentTargetPosition { agent_entity: agent_entity, target_pos: Vec3::ZERO }), // TODO: implement
                                },
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

pub fn update_agent_animations(
    actor_query: Query<&AnimationEntityLink, With<ActorType>>,
    mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations: Res<Animations>,
    agent_query: Query<(&ChildOf, &MoveAgent, &MoveAgentLastTick)>
) {
    for (child_of, move_agent, move_agent_last_tick) in agent_query {
        let animation_link = actor_query.get(child_of.parent().entity());
        if animation_link.is_err() { continue };
        let animation_link = animation_link.unwrap();

        let (
            mut animation_player, 
            mut animation_transitions
        ) = animation_query.get_mut(animation_link.0).unwrap();

        if move_agent.0 != move_agent_last_tick.0 {
            match move_agent.0 {
                true => {
                    animation_transitions
                        .play(
                            &mut animation_player, 
                            animations.animations[2], 
                            Duration::from_millis(250)
                        )
                        .repeat();
                }
                false => {
                    // this is a case by case basis and should be set by the propagator.
                }
            }
        }
    }
}

pub fn update_moveagent_laststate(mut agent_query: Query<(&MoveAgent, &mut MoveAgentLastTick)>) {
    // updates last state of movable agent at the end of a tick. runs in Last
    // this is used for setting up the animations
    for (moveagent, mut moveagent_last) in agent_query.iter_mut() {
        moveagent_last.0 = moveagent.0;
    }
}

pub fn move_agents(
    agent_query: Query<(&ChildOf, &AgentDesiredVelocity3d, &MoveAgent)>, 
    mut actor_query: Query<(&mut Transform, &mut KinematicCharacterController)>,
    time: Res<Time>,
) {
    for (child_of, desired_velocity, moveagent) in agent_query {
        match moveagent {
            // only move if moveagent true
            &MoveAgent(true) => {
                let (
                    mut transform, 
                    mut controller
                ) = actor_query.get_mut(child_of.parent().entity()).unwrap();

                if desired_velocity.velocity().length() > 0.05 {
                    // align transform rotation so that agent looks where it's going
                    let mut desired_look_dir = desired_velocity.velocity().normalize();
                    desired_look_dir.y = 0.0;
                    // debug!("turning");
                    let angle = -desired_look_dir.z.signum() * Vec3::X.angle_between(desired_look_dir);
                    let desired_rotation = Quat::from_axis_angle(Vec3::Y, angle);

                    transform.rotation = transform.rotation.rotate_towards(desired_rotation, 0.1);
                }
                
                // set next velocity
                let mut next_velocity: Vec3 = Vec3::new(0.0, -0.1, 0.0); // slight downward tilt so that the collider snaps to the ground.
                next_velocity += desired_velocity.velocity(); // add desired velocity
                controller.translation = Some(next_velocity * time.delta_secs());
            },
            _ => {}
        }
        
    }
}

#[derive(Clone)]
// TODO: add vision radius
pub struct CheckEntityInSight { pub entity_from: Entity, pub entity_to: Entity, pub radius: f32 }

pub fn on_check_entity_in_sight(
	trigger: On<BehaveTrigger<CheckEntityInSight>>, 
	mut commands: Commands, 
    archipelago: Query<&Archipelago<ThreeD>>,
    global_transforms: Query<&GlobalTransform>,
) {
	let ctx = trigger.ctx();
    let archipelago = archipelago.single().unwrap();

    let entity_from_transform = global_transforms.get(trigger.inner().entity_from).unwrap();
    let entity_from_pos = entity_from_transform.translation();
    let entity_to_pos = global_transforms.get(trigger.inner().entity_to).unwrap().translation();
    let raw_dist = entity_from_pos.distance(entity_to_pos);
    
    let sample_dist = PointSampleDistance3d { 
        horizontal_distance: 1.0,
        distance_above: 1.0, 
        distance_below: 1.0, 
        vertical_preference_ratio: 1.0, 
        animation_link_max_vertical_distance: 1.0 
    };

    let entity_from_pos_sampled = archipelago.sample_point(entity_from_pos, &sample_dist);
    let entity_to_pos_sampled = archipelago.sample_point(entity_to_pos, &sample_dist);
    match (entity_from_pos_sampled, entity_to_pos_sampled) {
        (Ok(from), Ok(to)) => {
            let path = archipelago.find_path(&from, &to, &HashMap::new(), bevy_landmass::PermittedAnimationLinks::All).unwrap();
            // debug!("path: {:?}", path);
            if path.len() <= 2 && raw_dist <= trigger.inner().radius {
                // debug!("visible");
                commands.trigger(ctx.success());
            } else {
                // debug!("not visible");
                commands.trigger(ctx.failure());
            }
        }
        _ => {
            // debug!("couldn't sample");
            commands.trigger(ctx.failure());
        }
    };
}

#[derive(Clone)]
// this should always return success
pub struct SetMoveTowardsTarget { pub agent_entity: Entity, pub do_move: bool }

pub fn on_set_move_towards_target(
	trigger: On<BehaveTrigger<SetMoveTowardsTarget>>,
    mut agent_query: Query<(&mut MoveAgent, &mut AgentTarget3d), With<AgentState>>,
    mut commands: Commands,
) {
	let ctx = trigger.ctx();
	let event = trigger.event().inner();

    let agent_query_result =  agent_query.get_mut(event.agent_entity);
    match agent_query_result {
        Ok((mut move_agent, target)) => {
            // set moving to true for this agent
            move_agent.0 = event.do_move;
            commands.trigger(ctx.success());
        },
        Err(e) => panic!("{e:?}"),
    }
}

#[derive(Component, Clone)]
pub struct Attack { attacking_agent_entity: Entity }

pub fn on_attack(
    attacks: Query<&Attack, Added<Attack>>,
    actor_query: Query<&AnimationEntityLink, With<ActorType>>,
    mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations: Res<Animations>,
) {
    for attack in attacks {
        let attacking_entity = attack.attacking_agent_entity;
        debug!("entity {} is attacking!", attacking_entity.index());

        // play attack anim
        if let Ok(AnimationEntityLink(anim_entity)) = actor_query.get(attacking_entity) {
            let (mut animation_player, mut animation_transitions) = animation_query.get_mut(*anim_entity).unwrap();
            // play attack animation.
            // the entity should have attack events defined for AnimationTarget at this point, so this should be enough.
            animation_transitions.play(
                &mut animation_player,  
                animations.animations[1], // attack
                Duration::from_millis(250)
            );
        }

    }
}


#[derive(Clone)]
pub struct CheckMoving { pub agent_entity: Entity }
pub fn on_check_moving (
    trigger: On<BehaveTrigger<CheckMoving>>,
    mut commands: Commands,
    agent_query: Query<&MoveAgent>, 
) {
    let ctx = trigger.ctx();
    match agent_query.get(trigger.inner().agent_entity).unwrap().0 {
        true => { commands.trigger(ctx.success()); },
        _ => {  commands.trigger(ctx.failure()); }
    }
}

#[derive(Clone)]
pub struct SetAgentTargetEntity { agent_entity: Entity, char_entity: Entity } 

#[derive(Clone)]
pub struct SetAgentTargetPosition { agent_entity: Entity, target_pos: Vec3 } 
