use crate::{asset::GameState, enemy::ai::AIPlugin};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod ai;
pub mod mortal;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AIPlugin).configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading)
                .load_collection::<mortal::MortalAssets>(),
        );
    }
}
