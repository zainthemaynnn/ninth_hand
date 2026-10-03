use bevy::{math::VectorSpace, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::Lens;

pub struct NineAssetPlugin;

impl Plugin for NineAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::InGame),
        );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    InGame,
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct UiRightLens {
    /// Start position.
    pub start: Val,
    /// End position.
    pub end: Val,
}

fn lerp_val(start: &Val, end: &Val, ratio: f32) -> Val {
    match (start, end) {
        (Val::Percent(start), Val::Percent(end)) => {
            Val::Percent((end - start).mul_add(ratio, *start))
        }
        (Val::Px(start), Val::Px(end)) => Val::Px((end - start).mul_add(ratio, *start)),
        (Val::Vw(start), Val::Vw(end)) => Val::Vw((end - start).mul_add(ratio, *start)),
        (Val::Vh(start), Val::Vh(end)) => Val::Vh((end - start).mul_add(ratio, *start)),
        (Val::VMin(start), Val::VMin(end)) => Val::VMin((end - start).mul_add(ratio, *start)),
        (Val::VMax(start), Val::VMax(end)) => Val::VMax((end - start).mul_add(ratio, *start)),
        _ => *start,
    }
}

impl Lens<Node> for UiRightLens {
    fn lerp(&mut self, mut target: Mut<Node>, ratio: f32) {
        target.right = lerp_val(&self.start, &self.end, ratio);
    }
}
