use std::collections::HashMap;
use std::time::Duration;

use bevy::ecs::{entity::Entity};
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_landmass::coords::ThreeD;
use bevy_landmass::{AgentDesiredVelocity3d, AgentState, AgentTarget, AgentTarget3d, Archipelago, Character, PathStep, PointSampleDistance3d};
use bevy_rapier3d::prelude::{KinematicCharacterController};
use crate::actor::animations::*;
use crate::actor::spawner::ReturnPoint;
use crate::actor::{Player,};
use crate::debug::{DebugFlags};

use super::ActorType;

#[derive(Component)]
// Represents what the agent of an actor should be doing when called.
pub enum MoveAgent {
    Moving,
    Idle,
    Turning
}

// This should run every time an actor is added to the scene
pub fn init_actor_behavior(
    mut commands: Commands,
    player_query: Query<(Entity, &Children), With<Player>>,
    actor_type_query: Query<(Entity, &Children, &ActorType), Added<ActorType>>,
    agent_query: Query<(Entity, &ReturnPoint), With<AgentState>>,
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
        // this will only give Ok if the agent is an enemy.
        let return_point = agent_query.get(agent_entity).and_then(|res| Ok(res.1.0));

        let graph = graphs.get(animations.graph_handle.id()).unwrap();
        let attack_anim_node = graph.get(animations.animations[1]).unwrap();
        let clip = match &attack_anim_node.node_type {
            AnimationNodeType::Clip(clip_handle) => clips.get(clip_handle.id()),
            _ => unreachable!(),
        }.unwrap();

        let (tree, agent_target) = match actor_type {
            ActorType::Enemy { sight_radius, attack_radius, home_radius } => {
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
                                    Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, new_move_agent: MoveAgent::Turning }),
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
                                        Behave::trigger(CheckWalkingDistanceBelow { agent_entity: agent_entity, threshold: *sight_radius } ),
                                        Behave::trigger(SetAgentTarget { agent_entity: agent_entity, target: AgentTarget3d::Entity(player_char_entity) } ), 
                                        Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, new_move_agent: MoveAgent::Moving }),
                                    },

                                    // enemy not in sight but still targeted logic
                                    Behave::Sequence => {
                                        Behave::trigger(CheckMoving { agent_entity: agent_entity }),
                                        Behave::trigger(CheckWalkingDistanceBelow { agent_entity: agent_entity, threshold: *home_radius }),
                                        Behave::trigger(CheckWalkingDistanceBelow { agent_entity: agent_entity, threshold: *attack_radius }),
                                        Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, new_move_agent: MoveAgent::Idle }),
                                    },

                                    // enemy not in sight logic
                                    Behave::Sequence => {
                                        Behave::Invert => {
                                            Behave::trigger(CheckWalkingDistanceBelow { agent_entity: agent_entity, threshold: *attack_radius }),
                                        },
                                        Behave::trigger(SetMoveTowardsTarget { agent_entity: agent_entity, new_move_agent: MoveAgent::Moving }),
                                        Behave::trigger(SetAgentTarget { agent_entity: agent_entity, target: AgentTarget3d::Point(return_point.unwrap()) }), // TODO: implement
                                    },
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
    agent_query: Query<(&ChildOf, &MoveAgent)>
) {
    for (child_of, move_agent) in agent_query {
        let animation_link = actor_query.get(child_of.parent().entity());
        if animation_link.is_err() { continue };
        let animation_link = animation_link.unwrap();

        let (
            mut animation_player, 
            mut animation_transitions
        ) = animation_query.get_mut(animation_link.0).unwrap();

        match move_agent {
            MoveAgent::Moving => {
                if !animation_player.is_playing_animation(animations.animations[2]) {
                    animation_transitions
                        .play(
                            &mut animation_player, 
                            animations.animations[2], 
                            Duration::from_millis(250)
                        )
                        .repeat();
                }
            },
            MoveAgent::Turning => {},
            MoveAgent::Idle => {
                if !animation_player.is_playing_animation(animations.animations[0]) {
                    animation_transitions
                        .play(
                            &mut animation_player, 
                            animations.animations[0], 
                            Duration::from_millis(250)
                        )
                        .repeat();
                }
            }
            _ => { unimplemented!(); }
        }

        
    }
}

fn turn(mut transform: Mut<'_, Transform>, desired_velocity: &AgentDesiredVelocity3d) {
    // align transform rotation so that agent looks where it's going
    let mut desired_look_dir = desired_velocity.velocity().normalize();
    desired_look_dir.y = 0.0;
    // debug!("turning");
    let angle = -desired_look_dir.z.signum() * Vec3::X.angle_between(desired_look_dir);
    let desired_rotation = Quat::from_axis_angle(Vec3::Y, angle);

    transform.rotation = transform.rotation.rotate_towards(desired_rotation, 0.03);
}

pub fn update_agents(
    agent_query: Query<(&ChildOf, &AgentDesiredVelocity3d, &MoveAgent)>, 
    mut actor_query: Query<(&mut Transform, &mut KinematicCharacterController)>,
    time: Res<Time>,
) {
    for (child_of, desired_velocity, moveagent) in agent_query {
        let (
            transform, 
            mut controller
        ) = actor_query.get_mut(child_of.parent().entity()).unwrap();
        match moveagent {
            // only move if moveagent true
            &MoveAgent::Moving => {
                if desired_velocity.velocity().length() > 0.05 {
                    turn(transform, desired_velocity);
                }
                
                // set next velocity
                let mut next_velocity: Vec3 = Vec3::new(0.0, -0.1, 0.0); // slight downward tilt so that the collider snaps to the ground.
                next_velocity += desired_velocity.velocity(); // add desired velocity
                controller.translation = Some(next_velocity * time.delta_secs());
            },

            &MoveAgent::Turning => {
                turn(transform, desired_velocity);
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
        distance_above: 5.0, 
        distance_below: 5.0, 
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

// this should always return success
pub struct SetMoveTowardsTarget { pub agent_entity: Entity, pub new_move_agent: MoveAgent }
impl Clone for SetMoveTowardsTarget {
    fn clone(&self) -> Self {
        SetMoveTowardsTarget {
            agent_entity: self.agent_entity.clone(),
            new_move_agent: match self.new_move_agent {
                MoveAgent::Idle => MoveAgent::Idle,
                MoveAgent::Moving => MoveAgent::Moving,
                MoveAgent::Turning => MoveAgent::Turning,
            }
        }
    }
}
pub fn on_set_move_towards_target(
	trigger: On<BehaveTrigger<SetMoveTowardsTarget>>,
    mut agent_query: Query<&mut MoveAgent, With<AgentState>>,
    mut commands: Commands,
) {
	let ctx = trigger.ctx();
	let event = trigger.event().inner().clone();

    let agent_query_result =  agent_query.get_mut(event.agent_entity);
    match agent_query_result {
        Ok(mut move_agent) => {
            // set moving to true for this agent
            *move_agent = event.new_move_agent;
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
    match agent_query.get(trigger.inner().agent_entity).unwrap() {
        MoveAgent::Moving => { commands.trigger(ctx.success()); },
        _ => {  commands.trigger(ctx.failure()); }
    }
}

pub struct SetAgentTarget { agent_entity: Entity, target: AgentTarget3d } 
impl Clone for SetAgentTarget {
    fn clone(&self) -> Self {
        SetAgentTarget {
            agent_entity: self.agent_entity.clone(),
            target: match self.target {
                AgentTarget3d::Entity(e) => AgentTarget3d::Entity(e),
                AgentTarget3d::Point(p) => AgentTarget::Point(p),
                AgentTarget3d::None => AgentTarget::None,
            }
        }
    }
}
pub fn on_set_agent_target (
    trigger: On<BehaveTrigger<SetAgentTarget>>,
    mut commands: Commands,
) {
    let trigger_params = trigger.inner().clone();
    commands
        .get_entity(trigger_params.agent_entity)
        .unwrap()
        .insert(trigger_params.target);

    
    commands.trigger(trigger.ctx().success());
}
#[derive(Clone)]
pub struct CheckWalkingDistanceBelow { agent_entity: Entity, threshold: f32 }
pub fn on_check_walking_distance_below(
	trigger: On<BehaveTrigger<CheckWalkingDistanceBelow>>, 
	mut commands: Commands, 
    agent_target_query: Query<&AgentTarget3d>,
    archipelago: Query<&Archipelago<ThreeD>>,
    global_transforms: Query<&GlobalTransform>,
) {
	let ctx = trigger.ctx();
    let archipelago = archipelago.single().unwrap();

    let trigger_params = trigger.inner();
    let agent_target = agent_target_query.get(trigger_params.agent_entity).unwrap();
    let entity_from_pos = global_transforms.get(trigger_params.agent_entity).unwrap().translation();
    let entity_to_pos = match agent_target {
        AgentTarget3d::Entity(e) => global_transforms.get(*e).unwrap().translation(),
        AgentTarget3d::Point(p) => *p,
        AgentTarget3d::None => unreachable!(),
    };

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
            let mut walking_distance = 0.0;
            match path.first() {
                Some(PathStep::Waypoint(first_point)) => {
                    let mut last_point = first_point.clone();
                    for path_step in path.iter().skip(1) {
                        match path_step {
                            bevy_landmass::PathStep::Waypoint(curr_point) => {
                                walking_distance += last_point.distance(*curr_point);
                                last_point = curr_point.clone();
                            },
                            _ => { unimplemented!(); }
                        }
                    }
                    if walking_distance <= trigger_params.threshold {
                        commands.trigger(ctx.success());
                    } else {
                        commands.trigger(ctx.failure());
                    }
                }
                _ => { unimplemented!(); }
            }
        }
        _ => {
            // debug!("couldn't sample");
            commands.trigger(ctx.failure());
        }
    };
}