use std::f32::consts::TAU;

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_hanabi::prelude::*;

use crate::{
    actions::{
        Action, ActionType, CardArt, OwnedByUnit, Played, Target,
        tags::{FrontTarget, LineOfSight, SingleTarget},
    },
    asset::{EllipsoidBackfaceDepthModifier, GameState},
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::OccupiedByUnit,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitColor, UnitDesc, UnitModelResource, UnitPriority, UnitType},
};

pub struct PresencePlugin;

impl Plugin for PresencePlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<PresenceAssets>(),
        )
        .init_resource::<PresenceSmokeEffect>();
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(4),
    Player,
    UnitDesc {
        name: "Presence",
        passive: "???",
        flavor: "HATE. ORGANICS.",
    },
    UnitColor(Srgba::hex("#CBC275").unwrap().into()),
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Bash, 1.5),
        (DamageVariant::Slash, 0.5),
        (DamageVariant::Heat, 0.8),
        (DamageVariant::Cold, 0.8),
    ])),
)]
pub struct Presence;

#[derive(Resource)]
pub struct PresenceSmokeEffect(Handle<EffectAsset>);

impl FromWorld for PresenceSmokeEffect {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();
        let writer = ExprWriter::new();

        // Sample an ellipsoid with the same 2x4x2 bounds as the previous cylinder.
        let angle = writer.rand(ScalarType::Float) * writer.lit(TAU);
        let vertical = writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(1.0);
        let radius = (writer.lit(1.0) - vertical.clone() * vertical.clone()).sqrt()
            * writer.rand(ScalarType::Float).sqrt();
        let x = radius.clone() * angle.clone().cos();
        let z = radius * angle.sin();
        let y = vertical * writer.lit(2.0) + writer.lit(3.0);
        let position = x.vec3(y, z).expr();

        let init_velocity = SetAttributeModifier::new(
            Attribute::VELOCITY,
            writer.lit(Vec3::new(0.0, 0.2, 0.0)).expr(),
        );
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(1.5).expr());
        let init_color = SetAttributeModifier::new(
            Attribute::COLOR,
            writer
                .lit(LinearRgba::new(0.35, 0.35, 0.35, 0.5).as_u32())
                .expr(),
        );
        let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(1.0).expr());
        let texture_slot = writer.lit(0u32).expr();
        let mut module = writer.finish();
        module.add_texture_slot("smoke");

        let mut color_gradient = bevy_hanabi::Gradient::new();
        color_gradient.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0));
        color_gradient.add_key(1.0, Vec4::ZERO);

        let effect = effects.add(
            EffectAsset::new(1024, SpawnerSettings::rate(240.0.into()), module)
                .with_name("PresenceSmoke")
                .init(SetAttributeModifier::new(Attribute::POSITION, position))
                .init(init_velocity)
                .init(init_age)
                .init(init_lifetime)
                .init(init_color)
                .init(init_size)
                .with_simulation_space(SimulationSpace::Local)
                .add_render_modifier(Box::new(EllipsoidBackfaceDepthModifier))
                .add_render_modifier(Box::new(OrientModifier::new(
                    OrientMode::FaceCameraPosition,
                )))
                .add_render_modifier(Box::new(ParticleTextureModifier::new(texture_slot)))
                .add_render_modifier(Box::new(ColorOverLifetimeModifier::new(color_gradient))),
        );
        Self(effect)
    }
}

pub fn spawn_presence_smoke(
    In(result): In<Result<Entity>>,
    mut commands: Commands,
    assets: Res<PresenceAssets>,
    smoke: Res<PresenceSmokeEffect>,
) {
    if let Ok(entity) = result {
        commands.entity(entity).insert((
            ParticleEffect::new(smoke.0.clone()),
            EffectMaterial {
                images: vec![assets.smoke.clone()],
            },
        ));
    }
}

#[derive(AssetCollection, Resource)]
pub struct PresenceAssets {
    #[asset(path = "gltf/presence.glb#Scene0")]
    pub presence: Handle<WorldAsset>,
    #[asset(path = "gltf/presence.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "img/particles/kenney/PNG (Transparent)/window_01.png")]
    pub smoke: Handle<Image>,
}

impl UnitModelResource for PresenceAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.presence.clone()
    }
}

#[derive(Component)]
pub struct PresenceAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for PresenceAnimations {
    type Assets = PresenceAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Presence {
    type ModelResource = PresenceAssets;
    type Animations = PresenceAnimations;
}

#[derive(Component)]
#[require(
    Action { name: "Slime", description: "Damage 2 front, Slow" },
    Damage,
    FrontTarget,
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
