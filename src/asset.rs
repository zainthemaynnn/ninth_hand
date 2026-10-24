use std::time::Duration;

use bevy::{
    animation::RepeatAnimation, math::VectorSpace, prelude::*,
    world_serialization::WorldInstanceReady,
};
use bevy_asset_loader::prelude::*;
use bevy_tweening::Lens;

pub struct NineAssetPlugin;

impl Plugin for NineAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::InGame),
            )
            .add_observer(register_animator);
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
        target.emissive = self.start.lerp(self.end, ratio);
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MaterialBaseColorLens {
    pub start: LinearRgba,
    pub end: LinearRgba,
}

impl Lens<StandardMaterial> for MaterialBaseColorLens {
    fn lerp(&mut self, mut target: Mut<StandardMaterial>, ratio: f32) {
        target.base_color = self.start.lerp(self.end, ratio).into();
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

/// Stores the first descendant with an [`AnimationPlayer`] when a world asset
/// instance has finished spawning. Attach this observer to the `WorldAssetRoot`
/// entity; consumers can then read [`AnimatorEntity`] from that root.
pub fn register_animator(
    scene_ready: On<WorldInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mut animator_query: Query<&mut AnimationPlayer>,
    init_query: Query<&AnimatorInitializer>,
) {
    let scene_root = scene_ready.entity;
    let Some(e_animator) = children
        .iter_descendants(scene_root)
        .find(|&entity| animator_query.contains(entity))
    else {
        warn!("no AnimationPlayer descendant found under {scene_root:?}");
        return;
    };

    commands
        .entity(scene_root)
        .insert(AnimatorEntity(e_animator));

    let mut transitions = AnimationTransitions::new();

    if let Ok(AnimatorInitializer { graph, idle }) = init_query.get(scene_root) {
        commands
            .entity(e_animator)
            .insert(AnimationGraphHandle(graph.clone()));

        // lowkey, I just read that only one animation is supported at once...
        // that means only the last in the vec is going to play.
        // however, this is arriving soon in an engine update, so I won't
        // worry about it right now
        if let Some(idle) = idle {
            let mut animator = animator_query.get_mut(e_animator).unwrap();
            for idle_anim in idle {
                transitions
                    .play(&mut animator, idle_anim.clone(), Duration::ZERO)
                    .set_repeat(RepeatAnimation::Forever);
            }
        }
    }

    commands.entity(e_animator).insert(transitions);
}

/// The first descendant `AnimationPlayer` found for a spawned world asset.
#[derive(Component, Clone, Debug)]
pub struct AnimatorEntity(pub Entity);

#[derive(Component, Clone, Debug)]
pub struct AnimatorInitializer {
    pub graph: Handle<AnimationGraph>,
    pub idle: Option<Vec<AnimationNodeIndex>>,
}
