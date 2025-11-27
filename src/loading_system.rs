use super::LevelHandles;
use super::MyAppState;
use crate::actor_navigation::CurrNavmesh;
use bevy::{asset::LoadState, asset::UntypedAssetId, prelude::*};
use bevy_rerecast::Navmesh;

// A list of assets currently being loaded.
// This should be a Single resource
// Should always be empty if app state is not Loading
#[derive(Default, Resource)]
pub struct AssetsLoading(pub(crate) Vec<UntypedHandle>);

// Start loading assets for the level.
pub fn start_loading_assets(
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    mut lvl_handles: ResMut<LevelHandles>,
    mut curr_navmesh: ResMut<CurrNavmesh>,
) {
    debug!("Loading assets...");

    let level_mesh: Handle<Mesh> = asset_server.load("models/dungeon.glb#Mesh0/Primitive0");
    let level_navmesh: Handle<Navmesh> = asset_server.load("navmeshes/castle_navmesh.nav");
    lvl_handles.0.push(level_mesh.clone());
    curr_navmesh.0 = level_navmesh.clone();
    
    // add everything to loading list
    let new_assets: Vec<UntypedHandle> = vec![
        level_mesh.into(),
        level_navmesh.into(),
    ];  
    loading.0.extend(new_assets);
}

// get a cumulative load state for a list of handles (only success if all of them are loaded)
// apparently this used to be a library method but was removed during a revamp and never added back
pub fn get_group_load_state(
    server: &AssetServer,
    handles: impl IntoIterator<Item = UntypedAssetId>,
) -> LoadState {
    let mut load_state = LoadState::Loaded;
    for handle_id in handles {
        match server.get_load_state(handle_id) {
            Some(LoadState::Loaded) => continue,
            Some(LoadState::Loading) => {
                load_state = LoadState::Loading;
            }
            Some(LoadState::Failed(x)) => return LoadState::Failed(x),
            Some(LoadState::NotLoaded) => return LoadState::NotLoaded,
            None => return LoadState::NotLoaded,
        }
    }

    load_state
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
    let state = get_group_load_state(&asset_server, loading.0.iter().map(|h| h.id()));
    match state {
        LoadState::Failed(err) => {
            // one of our assets had an error
            panic!("couldnt get load state of asset. {}", err);
        }
        LoadState::Loaded => {
            // all assets are now ready

            // this might be a good place to transition into your in-game state
            debug!("Loaded all assets, switching to ingame state");
            next_state.set(MyAppState::InGame);
            // remove the resource to drop the tracking handles
            commands.remove_resource::<AssetsLoading>();
            // (note: if you don't have any other handles to the assets
            // elsewhere, they will get unloaded after this)
        }
        _ => {
            // NotLoaded/Loading: not fully ready yet
        }
    }
}
