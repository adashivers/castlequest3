use bevy::ecs::{entity::Entity};
use bevy::prelude::*;
use bevy::color::palettes::basic::RED;
use bevy_behave::prelude::*;
use bevy_landmass::coords::ThreeD;
use bevy_landmass::{Agent, AgentDesiredVelocity3d, AgentState, AgentTarget3d, Character};
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
                let (player_entity, player_children) = player_query.single().unwrap();
                // The navigation mesh Character entity is actually a parent of the top entity that makes up the player. We use this to get it:
                let player_char_entity = player_children.iter().find(|x| {character_query.get(*x).is_ok()}).unwrap();
                let tree = behave! {
                        Behave::Forever => {
                            Behave::Fallback => {
                                Behave::Sequence => {
                                    // Behave::trigger(CheckEntityInSight { entity_from: entity, entity_to: player_entity, radius: *radius }),
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
	mut raycast: MeshRayCast,
	mut commands: Commands, 
    mut gizmos: Gizmos<CQ3DebugGizmos>,
	transforms: Query<&GlobalTransform>,
    child_of: Query<&ChildOf>,
) {
	let ctx = trigger.ctx();

	// get entity info from trigger
	let entity_from = trigger.inner().entity_from;
	let entity_to = trigger.inner().entity_to;
	let vision_radius = trigger.inner().radius;

	//get direction and distance vectors
	let entity_from_pos = transforms.get(entity_from).unwrap().translation();
	let entity_to_pos = transforms.get(entity_to).unwrap().translation();
	let dist_to_entity = entity_from_pos - entity_to_pos;
	let towards_entity = Dir3::new(dist_to_entity).unwrap();

	// cast ray
	let ray = Ray3d::new(entity_from_pos, towards_entity);
	let settings = MeshRayCastSettings {
		visibility: RayCastVisibility::Any,
		filter: &|e| e.index() != entity_from.index(), // dont collide with the entity sending the ray
		..default()
	};

	// check if first hit is the entity we seek
	match raycast.cast_ray(ray, &settings).first() {
		Some((first_hit_entity, hit)) => {
            gizmos.line(entity_from_pos, hit.point, RED);
            gizmos.sphere(entity_from_pos, 3.0, RED);
            gizmos.sphere(hit.point, 3.0, RED);
            
            let parent_entity_to = get_top_parent(entity_to, &child_of);
            let parent_hit_entity = get_top_parent(*first_hit_entity, &child_of);
            debug!("hit info\n\thit entity: {}\ttarget entity:{}\n\thit distance:{}\tvision radius:{}\n\thit point: {}", first_hit_entity.index(), entity_to.index(), hit.distance, vision_radius, hit.point);
			if parent_entity_to.index() == parent_hit_entity.index() && hit.distance <= vision_radius {
                debug!("hit target");
				commands.trigger(ctx.success());
			} else {
				commands.trigger(ctx.failure());
			}
		},
		None => { debug!("no hit"); commands.trigger(ctx.failure()); }
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
