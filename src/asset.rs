use std::time::Duration;

use bevy::{
    animation::RepeatAnimation, math::VectorSpace, prelude::*,
    world_serialization::WorldInstanceReady,
};
use bevy_asset_loader::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_tweening::Lens;

/// Moves particle centers to the far ellipsoid intersection along the camera ray.
/// The ellipsoid is centered at (0, 3, 0) with radii (1, 2, 1), matching the
/// Presence smoke spawn volume.
#[derive(Debug, Default, Clone, Copy, Reflect)]
pub struct EllipsoidBackfaceDepthModifier;

impl Modifier for EllipsoidBackfaceDepthModifier {
    fn context(&self) -> ModifierContext {
        ModifierContext::Render
    }

    fn as_render(&self) -> Option<&dyn RenderModifier> {
        Some(self)
    }

    fn as_render_mut(&mut self) -> Option<&mut dyn RenderModifier> {
        Some(self)
    }

    fn into_boxed_render(self: Box<Self>) -> Option<Box<dyn RenderModifier>> {
        Some(self)
    }

    fn attributes(&self) -> &[Attribute] {
        &[Attribute::POSITION]
    }

    fn boxed_clone(&self) -> BoxedModifier {
        Box::new(*self)
    }

    fn apply(&self, _module: &mut Module, context: &mut ShaderWriter) -> Result<(), ExprError> {
        Err(ExprError::InvalidModifierContext(
            context.modifier_context(),
            ModifierContext::Render,
        ))
    }
}

impl RenderModifier for EllipsoidBackfaceDepthModifier {
    fn apply_render(
        &self,
        _module: &mut Module,
        context: &mut RenderContext,
    ) -> Result<(), ExprError> {
        context.vertex_code += r#"
// Convert the camera position into this effect's local space, including translation.
let ellipsoid_inverse_transform = unpack_compressed_transform(
    spawners[batch_info.base_effect + effect_location.effect_index].inverse_transform
);
let ellipsoid_camera = (ellipsoid_inverse_transform * vec4<f32>(view.world_from_view[3].xyz, 1.0)).xyz;
let ellipsoid_particle = position;
let ellipsoid_ray = normalize(ellipsoid_particle - ellipsoid_camera);
let ellipsoid_center = vec3<f32>(0.0, 3.0, 0.0);
let ellipsoid_radii = vec3<f32>(1.0, 2.0, 1.0);
let ellipsoid_origin_scaled = (ellipsoid_camera - ellipsoid_center) / ellipsoid_radii;
let ellipsoid_direction_scaled = ellipsoid_ray / ellipsoid_radii;
let ellipsoid_a = dot(ellipsoid_direction_scaled, ellipsoid_direction_scaled);
let ellipsoid_b = dot(ellipsoid_origin_scaled, ellipsoid_direction_scaled);
let ellipsoid_c = dot(ellipsoid_origin_scaled, ellipsoid_origin_scaled) - 1.0;
let ellipsoid_discriminant = ellipsoid_b * ellipsoid_b - ellipsoid_a * ellipsoid_c;
if ellipsoid_discriminant >= 0.0 && ellipsoid_a > 0.0 {
    let ellipsoid_far_distance = (-ellipsoid_b + sqrt(ellipsoid_discriminant)) / ellipsoid_a;
    let ellipsoid_particle_distance = distance(ellipsoid_camera, ellipsoid_particle);
    if ellipsoid_far_distance > 0.0 && ellipsoid_particle_distance > 0.0 {
        position = ellipsoid_camera + ellipsoid_ray * ellipsoid_far_distance;
        // Keep the billboard's apparent screen size after moving it farther away.
        size *= ellipsoid_far_distance / ellipsoid_particle_distance;
    }
}
"#;
        Ok(())
    }

    fn boxed_render_clone(&self) -> Box<dyn RenderModifier> {
        Box::new(*self)
    }

    fn as_modifier(&self) -> &dyn Modifier {
        self
    }
}

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
