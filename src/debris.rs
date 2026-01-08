use bevy::prelude::*;

pub struct DebrisPlugin;
impl Plugin for DebrisPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, 
            remove_debris
        );
    }
}

#[derive(Component)]
// any entity with this component will be removed after the timer is done.
pub struct Debris(pub Timer);

pub fn remove_debris(
    mut commands: Commands,
    debris_query: Query<(Entity, &Debris)>
) {
    for (entity, debris) in debris_query {
        if debris.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}