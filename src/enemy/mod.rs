use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::asset::GameState;

pub mod ai;
pub mod mortal;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<EnemyAssets>(),
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(path = "gltf/enemy/mortal.glb#Scene0")]
    pub mortal: Handle<WorldAsset>,
}
