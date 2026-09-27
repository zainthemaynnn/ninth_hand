use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct NineAssetPlugin;

impl Plugin for NineAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameAssetLoadState>().add_loading_state(
            LoadingState::new(GameAssetLoadState::AssetLoading)
                .continue_to_state(GameAssetLoadState::Next),
        );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameAssetLoadState {
    #[default]
    AssetLoading,
    Next,
}
