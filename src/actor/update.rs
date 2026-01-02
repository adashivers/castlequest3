use std::time::Duration;
use bevy::prelude::*;
use bevy_behave::prelude::*;
use bevy_rapier3d::{prelude::KinematicCharacterController};
use bevy_landmass::{
	AgentDesiredVelocity3d, 
	AgentState, 
};

use crate::actor::spawner::ActorType;
use crate::actor::*;

#[derive(Component, Default)]
pub struct LastState(pub AgentState);

pub fn update_enemies(
	mut actor_query: Query<(Entity, &ActorType, &mut KinematicCharacterController, &mut Transform, &AnimationEntityLink)>,
	mut agent_query: Query<(Entity, &ChildOf, &AgentState, &AgentDesiredVelocity3d, &mut LastState)>,
	mut animation_query: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	player_query: Query<Entity, With<Player>>,
	animations: Res<Animations>,
	time: Res<Time>,
) {
	let player_entity = player_query.single().unwrap();

	for (
		enemy_agent_entity,
		childof, 
		agent_state, 
		desired_velocity, 
		mut last_state
	) in &mut agent_query {
		if let Ok((
			enemy_entity,
			actor_type,
			mut controller,
			mut transform,
			animation_link,
		)) = actor_query.get_mut(childof.parent()) {

			// if not an enemy, skip over this actor
			match actor_type {
				ActorType::Enemy {radius: _ } => {},
				_ => {continue}
			}

			let (
				mut animation_player, 
				mut animation_transitions
			) = animation_query.get_mut(animation_link.0).unwrap();

			// TODO: add behavior tree action here
			let _tree = behave! {
				Behave::Fallback => {
					Behave::Sequence => {
						Behave::trigger(CheckEntityInSight { entity_from: enemy_entity, entity_to: player_entity }),
						Behave::trigger(MoveTowardsTarget { agent_entity: enemy_agent_entity, actor_entity: enemy_entity }),
					},
					Behave::trigger(SwitchToIdling)
				}
			};




			// set animation depending on agent state
			if *agent_state != last_state.0 {
				debug!("skeleton state: {:?}", agent_state);
				match agent_state {
					AgentState::Moving => {
						animation_transitions
							.play(
								&mut animation_player, 
								animations.animations[2], 
								Duration::from_millis(250)
							)
							.repeat();
					},
					AgentState::ReachedTarget => {
						// this will change in the future to trigger an "on attack" event
						// instead of playing the attack animation on loop
						animation_transitions
							.play(
								&mut animation_player, 
								animations.animations[1], 
								Duration::from_millis(250)
							)
							.repeat();
					},
					_ => {
						animation_transitions
							.play(
								&mut animation_player, 
								animations.animations[0], 
								Duration::from_millis(250)
							)
							.repeat();
					},
				}
				last_state.0 = *agent_state;
			}

			// align transform rotation so that skeleton looks where it's going
			if desired_velocity.velocity().length() > 0.1 {
				transform.align(Dir3::X, desired_velocity.velocity().normalize(), Dir3::Y, Dir3::Y);
			}
			
			// set next velocity
			let mut next_velocity: Vec3 = Vec3::new(0.0, -0.1, 0.0); // slight downward tilt so that the collider snaps to the ground.
			next_velocity += desired_velocity.velocity(); // add desired velocity
			controller.translation = Some(next_velocity * time.delta_secs());
		}


	}
}

#[derive(Clone)]
pub struct CheckEntityInSight { entity_from: Entity, entity_to: Entity }


pub fn on_check_entity_in_sight(
	trigger: On<BehaveTrigger<CheckEntityInSight>>, 
	mut raycast: MeshRayCast,
	mut commands: Commands, 
	transforms: Query<&GlobalTransform>,
) {
	let ctx = trigger.ctx();

	// get entity info from trigger
	let entity_from = trigger.inner().entity_from;
	let entity_to = trigger.inner().entity_to;

	//get direction and distance vectors
	let entity_from_pos = transforms.get(entity_from).unwrap().translation();
	let entity_to_pos = transforms.get(entity_to).unwrap().translation();
	let dist_to_entity = entity_from_pos - entity_to_pos;
	let towards_entity = Dir3::new(dist_to_entity).unwrap();

	// cast ray
	let ray = Ray3d::new(entity_from_pos, towards_entity);
	let settings = MeshRayCastSettings {
		visibility: RayCastVisibility::Visible,
		filter: &|e| e.index() != entity_from.index(), // dont collide with the entity sending the ray
		..default()
	};

	// check if first hit is the entity we seek
	match raycast.cast_ray(ray, &settings).first() {
		Some((first_hit_entity, _)) => {
			if first_hit_entity.index() == entity_to.index() {
				commands.trigger(ctx.success());
			} else {
				commands.trigger(ctx.failure());
			}
		},
		None => { commands.trigger(ctx.failure()); }
	}
}

#[derive(Clone)]
pub struct MoveTowardsTarget { agent_entity: Entity, actor_entity: Entity }

pub fn on_move_towards_target(
	mut commands: Commands,
	trigger: On<BehaveTrigger<MoveTowardsTarget>>,
	desired_velocities: Query<&AgentDesiredVelocity3d>,
	mut actor_query: Query<(&mut Transform, &mut KinematicCharacterController)>,
	time: Res<Time>
) {
	let ctx = trigger.ctx();
	let event = trigger.event().inner();
	let (mut transform, mut controller) = actor_query.get_mut(event.actor_entity).unwrap();

	if let Ok(desired_velocity) = desired_velocities.get(event.agent_entity) {
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
