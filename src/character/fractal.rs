use std::f32::consts::TAU;

use bevy::{prelude::*, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;
use bevy_hanabi::{
    Attribute, EffectAsset, EffectMaterial, ExprWriter, OrientMode, OrientModifier, ParticleEffect,
    ParticleTextureModifier, ScalarType, SetAttributeModifier, SimulationSpace,
    SizeOverLifetimeModifier, SpawnerSettings,
};

use crate::{
    actions::{
        Action, ActionType, CardArt, OwnedByUnit, Played, Target,
        tags::{LineOfSight, SingleTarget},
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap},
    field::OccupiedByUnit,
    systems::find_named_node,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitColor, UnitDesc, UnitModelResource, UnitPriority, UnitType},
};

pub struct FractalPlugin;

impl Plugin for FractalPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<FractalAssets>(),
        )
        .init_resource::<HeadGlitchEffect>();
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(4),
    Player,
    UnitDesc {
        name: "Fractal",
        passive: "Position is randomized every turn. Attempts to avoid cells with negative effects.",
        flavor: "WTF????????@^#$&*#$@^%!>>????",
    },
    UnitColor(Srgba::hex("#CBCBCB").unwrap().into()),
    Health(5.0),
    HealthCap(10.0),
)]
pub struct Fractal;

#[derive(AssetCollection, Resource)]
pub struct FractalAssets {
    #[asset(path = "gltf/fractal.glb#Scene0")]
    pub fractal: Handle<WorldAsset>,
    #[asset(path = "gltf/fractal.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "img/particles/triangle.png")]
    pub glitch: Handle<Image>,
}

impl UnitModelResource for FractalAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.fractal.clone()
    }
}

#[derive(Component)]
pub struct FractalAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for FractalAnimations {
    type Assets = FractalAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Fractal {
    type ModelResource = FractalAssets;
    type Animations = FractalAnimations;
}

pub fn spawn_fractal_head(In(result): In<Result<Entity>>, mut commands: Commands) {
    if let Ok(entity) = result {
        commands.entity(entity).observe(
            |trigger: On<WorldInstanceReady>,
             mut commands: Commands,
             children_query: Query<&Children>,
             name_query: Query<&Name>,
             assets: Res<FractalAssets>,
             glitch: Res<HeadGlitchEffect>| {
                let e_head =
                    find_named_node(trigger.entity, "HeadEmpty", &children_query, &name_query)
                        .unwrap();
                commands.entity(e_head).insert((
                    ParticleEffect::new(glitch.0.clone()),
                    EffectMaterial {
                        images: vec![assets.glitch.clone()],
                    },
                ));
            },
        );
    }
}

#[derive(Component)]
#[require(
    Action { name: "Pew pew", description: "BSH 1 twice" },
    Damage,
    SingleTarget,
    LineOfSight,
    CardArt("abyss"),
)]
pub struct Smash;

impl ActionType for Smash {
    fn play() -> Observer {
        Observer::new(
            |trigger: On<Played>,
             action_query: Query<&OwnedByUnit>,
             target_query: Single<&OccupiedByUnit, With<Target>>,
             mut damage_events: MessageWriter<DamageEvent>| {
                let e_unit = action_query.get(trigger.entity).unwrap().0;
                let occupied_by = target_query.into_inner();
                let e_enemy = occupied_by.iter().next().unwrap();

                damage_events.write(DamageEvent {
                    damage: Damage {
                        ty: DamageVariant::Plague,
                        value: 2.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}

#[derive(Resource)]
pub struct HeadGlitchEffect(pub Handle<EffectAsset>);

impl FromWorld for HeadGlitchEffect {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();
        let writer = ExprWriter::new();

        let angle = writer.rand(ScalarType::Float) * writer.lit(TAU);
        let vertical = writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(1.0);
        let random_radius = (writer.rand(ScalarType::Float).log() / writer.lit(3.0)).exp();
        let radius = (writer.lit(1.0) - vertical.clone() * vertical.clone()).sqrt() * random_radius;
        let x = radius.clone() * angle.clone().cos() * writer.lit(0.5);
        let z = radius.clone() * angle.sin() * writer.lit(0.5);
        let y = vertical * radius * writer.lit(0.5);
        let position = x.vec3(y, z);
        let position_expr = position.clone().expr();

        let init_velocity = SetAttributeModifier::new(
            Attribute::VELOCITY,
            position.normalized().mul(writer.lit(0.2)).expr(),
        );
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.5).expr());
        let init_rotation = SetAttributeModifier::new(
            Attribute::F32_0,
            (writer.rand(ScalarType::Float) * writer.lit(TAU)).expr(),
        );
        let rotation = writer.attr(Attribute::F32_0).expr();
        let shade = (writer.rand(ScalarType::Float) * writer.lit(3.0)).floor();
        let shade = writer.lit(1.0) - shade * writer.lit(0.5);
        let shade = shade
            .clone()
            .vec3(shade.clone(), shade)
            .vec4_xyz_w(writer.lit(1.0))
            .pack4x8unorm();
        let init_color = SetAttributeModifier::new(Attribute::COLOR, shade.expr());
        let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(1.0).expr());
        let texture_slot = writer.lit(0u32).expr();
        let mut module = writer.finish();
        module.add_texture_slot("glitch");

        let mut size_gradient = bevy_hanabi::Gradient::new();
        size_gradient.add_key(0.0, Vec3::ONE);
        size_gradient.add_key(1.0, Vec3::ZERO);

        let effect = effects.add(
            EffectAsset::new(1024, SpawnerSettings::rate(100.0.into()), module)
                .with_name("FractalGlitch")
                .init(SetAttributeModifier::new(
                    Attribute::POSITION,
                    position_expr,
                ))
                .init(init_velocity)
                .init(init_age)
                .init(init_lifetime)
                .init(init_rotation)
                .init(init_color)
                .init(init_size)
                .with_simulation_space(SimulationSpace::Local)
                .add_render_modifier(Box::new(
                    OrientModifier::new(OrientMode::FaceCameraPosition).with_rotation(rotation),
                ))
                .add_render_modifier(Box::new(ParticleTextureModifier::new(texture_slot)))
                .add_render_modifier(Box::new(SizeOverLifetimeModifier {
                    gradient: size_gradient,
                    screen_space_size: false,
                })),
        );
        Self(effect)
    }
}
