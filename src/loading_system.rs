use crate::actor::navigation::NavmeshGenerators;
use super::MyAppState;
use bevy::{asset::LoadState, prelude::*};
use crate::utils::get_group_load_state;

// A list of assets currently being loaded.
// This should be a Single resource
// Should always be empty if app state is not Loading
#[derive(Default, Resource)]
pub struct AssetsLoading(pub(crate) Vec<UntypedHandle>);

// list of scenes in the game
#[derive(Default, Resource)]
pub struct GameScenes(pub(crate) Vec<Handle<Scene>>);

// This system is responsible for loading everything external in the right order.
pub struct LoadingSystemPlugin;
impl Plugin for LoadingSystemPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<AssetsLoading>()
        .init_resource::<GameScenes>()
        .add_systems(Startup, (
            start_loading_assets,
        ))
        .add_systems(Update, checks_assets_loaded.in_set(super::LoadingSet));
    }
}

// Start loading assets for the level.
pub fn start_loading_assets(
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut navmesh_generators: ResMut<NavmeshGenerators>,
    mut scenes: ResMut<GameScenes>,
) {
    debug!("Loading assets...");
    // load the navmesh template for the castle and add it to the list of navmesh-generating meshes
    let castle_navmesh_gen_mesh: Handle<Mesh> = asset_server.load("models/dungeon.glb#Mesh0/Primitive0");
    navmesh_generators.0.push(castle_navmesh_gen_mesh.clone());

    // load the castle scene and add it to the list of scenes
    let castle_scene: Handle<Scene> = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/dungeontex.glb"),
    );
    scenes.0.push(castle_scene.clone());

    // add everything to loading list
    let new_assets: Vec<UntypedHandle> = vec![
        castle_navmesh_gen_mesh.into(),
        castle_scene.into(),
    ];  
    loading.0.extend(new_assets);
}

// Check if the assets currently being loaded are done loading.
// This runs every frame if app state is Loading, and will transition app state to InGame
// if all assets are loaded.
// todo: find a way to do this faster
pub fn checks_assets_loaded(
    mut commands: Commands,
    mut next_state: ResMut<NextState<MyAppState>>,
    asset_server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
) {
    // get loading state
    let state = get_group_load_state(&asset_server, loading.0.iter().map(|h| h.id()));
    match state {
        LoadState::Failed(err) => {
            // one of our assets had an error
            panic!("couldnt get load state of asset. {}", err);
        }
        LoadState::Loaded => {
            // all assets are now ready
            // transition into in-game state
            debug!("Loaded all assets, switching to ingame state");
            next_state.set(MyAppState::InGame);

            // remove the resource to drop the tracking handles
            commands.remove_resource::<AssetsLoading>();
            // (note: if you don't have any other handles to the assets
            // elsewhere, they will get unloaded after this)
        }
        _ => {
            // NotLoaded/Loading: assets are not fully ready yet, do nothing.
        }
    }
}
