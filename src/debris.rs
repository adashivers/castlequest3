use bevy::prelude::*;

pub struct DebrisPlugin;
impl Plugin for DebrisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, 
            update_debris
        );
    }
}

#[derive(Component, Clone)]
// any entity with this component will be removed after the timer is done.
pub struct Debris(pub Timer);

pub fn update_debris(
    mut commands: Commands,
    debris_query: Query<(Entity, &mut Debris)>,
    time: Res<Time>,
) {
    let mut deleted = 0;
    for (entity, mut debris) in debris_query {
        debris.0.tick(time.delta());
        if debris.0.is_finished() {
            deleted += 1;
            commands.entity(entity).despawn();
        }
    }
    if deleted > 0 {
        debug!("deleted {} debris", deleted);
    }
}