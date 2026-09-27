use bevy::{math::VectorSpace, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::Lens;

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MaterialEmissiveLens {
    pub start: LinearRgba,
    pub end: LinearRgba,
}

impl Lens<StandardMaterial> for MaterialEmissiveLens {
    fn lerp(&mut self, mut target: Mut<StandardMaterial>, ratio: f32) {
        debug!(ratio);
        target.emissive = self.start.lerp(self.end, ratio);
    }
}
