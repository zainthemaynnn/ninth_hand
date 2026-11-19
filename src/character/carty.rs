use std::f32::consts::TAU;

use bevy::{prelude::*, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;
use bevy_hanabi::prelude::*;

use crate::{
    actions::{
        action::{Action, ActionType, OwnedByUnit, Played},
        tags::{FrontTarget, LineOfSight, SingleTarget},
        target::Target,
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap},
    field::OccupiedByUnit,
    turn::Player,
    unit::{Unit, UnitAnimations, UnitAssetResource, UnitColor, UnitDesc, UnitPriority, UnitType},
};

pub struct CartyPlugin;

impl Plugin for CartyPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<CartyAssets>(),
        )
        .init_resource::<CartyInnerFlameEffect>();
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(5),
    Player,
    UnitDesc {
        name: "Miss Incartilage",
        passive: "When an enemy dies, gain a soul.",
        flavor: "I've collected so many souls, but I still haven't found the meaning of life.",
    },
    UnitColor(Srgba::hex("#0900FF").unwrap().into()),
    Health(5.0),
    HealthCap(10.0),
)]
pub struct Carty;

#[derive(AssetCollection, Resource)]
pub struct CartyAssets {
    #[asset(path = "gltf/carty.glb#Scene0")]
    pub carty: Handle<WorldAsset>,
    #[asset(path = "gltf/carty.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "img/particles/kenney/PNG (Transparent)/scorch_01.png")]
    pub ember: Handle<Image>,
}

impl UnitAssetResource for CartyAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.carty.clone()
    }
}

#[derive(Component)]
pub struct CartyAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for CartyAnimations {
    type Assets = CartyAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

impl UnitType for Carty {
    type ModelResource = CartyAssets;
    type Animations = CartyAnimations;
}

#[derive(Debug, Default, Clone, Copy, Reflect)]
struct ParticleColorAttributeModifier;

impl Modifier for ParticleColorAttributeModifier {
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
        &[Attribute::HDR_COLOR, Attribute::AGE, Attribute::LIFETIME]
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

impl RenderModifier for ParticleColorAttributeModifier {
    fn apply_render(
        &self,
        _module: &mut Module,
        context: &mut RenderContext,
    ) -> Result<(), ExprError> {
        context.vertex_code.push_str(
            "let color_age = clamp(particle.age / particle.lifetime, 0.0, 1.0);\n\
             color = vec4<f32>(particle.hdr_color.rgb, 1.0 - color_age);\n",
        );
        Ok(())
    }

    fn boxed_render_clone(&self) -> Box<dyn RenderModifier> {
        Box::new(*self)
    }

    fn as_modifier(&self) -> &dyn Modifier {
        self
    }
}

pub fn spawn_flame(In(result): In<Result<Entity>>, mut commands: Commands) {
    if let Ok(entity) = result {
        commands.entity(entity).observe(
            |trigger: On<WorldInstanceReady>,
             mut commands: Commands,
             children_query: Query<&Children>,
             name_query: Query<&Name>,
             inner_flame: Res<CartyInnerFlameEffect>,
             assets: Res<CartyAssets>| {
                let e_skull = crate::systems::find_named_node(
                    trigger.entity,
                    "Skull",
                    &children_query,
                    &name_query,
                )
                .unwrap();
                commands.entity(e_skull).with_children(|parent| {
                    parent.spawn((
                        ParticleEffect::new(inner_flame.0.clone()),
                        EffectMaterial {
                            images: vec![assets.ember.clone()],
                        },
                    ));
                });
            },
        );
    }
}

#[derive(Component)]
#[require(
    Action { name: "Slash", description: "Damage 4 front" },
    Damage,
    FrontTarget,
    SingleTarget,
    LineOfSight,
)]
pub struct Reap;

impl ActionType for Reap {
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
                        ty: DamageVariant::Slash,
                        value: 4.0,
                        source: Some(e_unit),
                    },
                    target: e_enemy,
                });
            },
        )
    }
}

#[derive(Resource)]
pub struct CartyFlameEffect(pub Handle<EffectAsset>);

impl FromWorld for CartyFlameEffect {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();

        let writer = ExprWriter::new();

        let angle = writer.rand(ScalarType::Float) * writer.lit(TAU);

        let max_radius = writer.lit(0.12);
        let r = writer.rand(ScalarType::Float).sqrt() * max_radius.clone();

        let x = r.clone() * angle.clone().cos();
        let y = writer.lit(-0.4);
        let z = r.clone() * angle.sin();
        let position = x.vec3(y, z).expr();

        let speed = writer.lit(0.8) + writer.rand(ScalarType::Float) * writer.lit(0.8);
        let sideways = (writer.rand(ScalarType::Float) - writer.lit(0.5)) * writer.lit(0.5);
        let depth_drift = (writer.rand(ScalarType::Float) - writer.lit(0.5)) * writer.lit(0.4);
        let diagonal = writer.lit(std::f32::consts::FRAC_1_SQRT_2);
        let velocity_x = speed.clone() * diagonal.clone() * writer.lit(-1.0) + sideways;
        let velocity_y = speed * diagonal;
        let velocity = velocity_x.vec3(velocity_y, depth_drift).expr();

        let init_pos = SetAttributeModifier::new(Attribute::POSITION, position);
        let init_velocity = SetAttributeModifier::new(Attribute::VELOCITY, velocity);
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        let lifetime = writer.lit(0.6) + writer.rand(ScalarType::Float) * writer.lit(0.6);
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr());
        let flame_color = LinearRgba::from(Srgba::hex("#0900FF").unwrap());
        let bright_core = LinearRgba::from(Srgba::hex("#667BFF").unwrap());
        let init_color =
            SetAttributeModifier::new(Attribute::COLOR, writer.lit(flame_color.as_u32()).expr());
        let lift = AccelModifier::new(writer.lit(Vec3::new(0.0, 0.35, 0.0)).expr());

        let texture_slot = writer.lit(0u32).expr();
        let mut module = writer.finish();
        module.add_texture_slot("ember");

        let mut color_gradient = bevy_hanabi::Gradient::new();
        color_gradient.add_key(
            0.0,
            Vec4::new(
                bright_core.red,
                bright_core.green,
                bright_core.blue,
                bright_core.alpha,
            ),
        );
        color_gradient.add_key(
            0.2,
            Vec4::new(
                flame_color.red,
                flame_color.green,
                flame_color.blue,
                flame_color.alpha,
            ),
        );
        color_gradient.add_key(
            1.0,
            Vec4::new(flame_color.red, flame_color.green, flame_color.blue, 0.0),
        );

        let h_effect = effects.add(
            EffectAsset::new(4096, SpawnerSettings::rate(80.0.into()), module)
                .with_name("CartyEmber")
                .init(init_pos)
                .init(init_velocity)
                .init(init_age)
                .init(init_lifetime)
                .init(init_color)
                .with_simulation_space(SimulationSpace::Local)
                .update(lift)
                .add_render_modifier(Box::new(ParticleTextureModifier::new(texture_slot)))
                .add_render_modifier(Box::new(ColorOverLifetimeModifier::new(color_gradient)))
                .add_render_modifier(Box::new(SizeOverLifetimeModifier {
                    gradient: {
                        let mut gradient = bevy_hanabi::Gradient::new();
                        gradient.add_key(0.0, Vec3::splat(0.28));
                        gradient.add_key(0.25, Vec3::splat(0.4));
                        gradient.add_key(1.0, Vec3::ZERO);
                        gradient
                    },
                    screen_space_size: false,
                })),
        );

        Self(h_effect)
    }
}

#[derive(Resource)]
pub struct CartyInnerFlameEffect(pub Handle<EffectAsset>);

impl FromWorld for CartyInnerFlameEffect {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();
        let writer = ExprWriter::new();

        let angle = writer.rand(ScalarType::Float) * writer.lit(TAU);
        let vertical = writer.rand(ScalarType::Float) * writer.lit(2.0) - writer.lit(1.0);
        let sphere_radius =
            (writer.rand(ScalarType::Float).log() / writer.lit(3.0)).exp() * writer.lit(0.7);
        let horizontal_radius = (writer.lit(1.0) - vertical.clone() * vertical.clone()).sqrt();
        let x = sphere_radius.clone() * horizontal_radius.clone() * angle.clone().cos();
        let y = sphere_radius.clone() * vertical;
        let z = sphere_radius * horizontal_radius * angle.sin();
        let position = x.vec3(y, z).expr();
        let velocity = writer.lit(Vec3::ZERO).expr();
        let init_position = SetAttributeModifier::new(Attribute::POSITION, position);
        let init_velocity = SetAttributeModifier::new(Attribute::VELOCITY, velocity);
        let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.0).expr());
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(3.0).expr());
        let flame_color = LinearRgba::from(Srgba::hex("#0900FF").unwrap());
        let random_color = writer.rand(ScalarType::Float);
        let color_factor = random_color.clone() * random_color.clone() * random_color;
        let red = writer.lit(flame_color.red) * color_factor.clone();
        let green = writer.lit(flame_color.green) * color_factor.clone();
        let blue = writer.lit(flame_color.blue) * color_factor;
        let init_color = SetAttributeModifier::new(
            Attribute::HDR_COLOR,
            red.vec3(green, blue).vec4_xyz_w(writer.lit(1.0)).expr(),
        );
        let init_size = SetAttributeModifier::new(Attribute::SIZE, writer.lit(2.0).expr());
        let texture_slot = writer.lit(0u32).expr();
        let mut module = writer.finish();
        module.add_texture_slot("ember");

        let effect = effects.add(
            EffectAsset::new(4096, SpawnerSettings::rate((64.0_f32 / 3.0).into()), module)
                .with_name("CartyInnerFlame")
                .init(init_position)
                .init(init_velocity)
                .init(init_age)
                .init(init_lifetime)
                .init(init_color)
                .init(init_size)
                .with_simulation_space(SimulationSpace::Local)
                .add_render_modifier(Box::new(OrientModifier::new(
                    OrientMode::FaceCameraPosition,
                )))
                .add_render_modifier(Box::new(ParticleTextureModifier {
                    texture_slot,
                    sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
                }))
                .add_render_modifier(Box::new(ParticleColorAttributeModifier)),
        );

        Self(effect)
    }
}
