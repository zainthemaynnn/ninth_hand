use bevy::{color::palettes, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{asset::GameState, ui::FloatyTextEvent};

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<DamageAssets>(),
        )
        .add_message::<DamageEvent>()
        .add_systems(
            Update,
            (
                (apply_damage_events, apply_resist, apply_damage_buffers).chain(),
                show_damage_events,
            )
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Message)]
pub struct DamageEvent {
    pub damage: Damage,
    pub target: Entity,
}

pub fn apply_damage_events(
    mut events: MessageReader<DamageEvent>,
    mut buf_query: Query<&mut DamageBuffer>,
) {
    for DamageEvent {
        damage,
        target: e_target,
    } in events.read()
    {
        let mut damage_buf = buf_query.get_mut(*e_target).unwrap();
        damage_buf.0.push(damage.clone());
    }
}

pub fn show_damage_events(
    mut events: MessageReader<DamageEvent>,
    mut text_events: MessageWriter<FloatyTextEvent>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    transform_query: Query<&GlobalTransform, Without<Camera3d>>,
) {
    let (cam, cam_tf) = camera_query.single().unwrap();

    for DamageEvent {
        damage,
        target: e_target,
    } in events.read()
    {
        let target_tf = transform_query.get(*e_target).unwrap();
        let Ok(screen_pos) = cam.world_to_viewport(cam_tf, target_tf.translation()) else {
            continue;
        };

        text_events.write(FloatyTextEvent {
            text: format!("{0} {1}", damage.ty.symbol(), damage.value),
            location: screen_pos,
            color_start: damage.ty.color(),
            ..Default::default()
        });
    }
}

#[derive(AssetCollection, Resource)]
pub struct DamageAssets {
    #[asset(path = "img/damage_icons", collection(typed, mapped))]
    pub icons: HashMap<AssetFileStem, Handle<Image>>,
}

impl DamageAssets {
    pub fn icon_from_damage_variant(&self, variant: DamageVariant) -> Option<&Handle<Image>> {
        return self.icons.get(variant.symbol().to_lowercase().as_str());
    }
}

/// Damage variant.
#[derive(Component, Hash, Eq, PartialEq, Copy, Clone, Debug, Default)]
pub enum DamageVariant {
    #[default]
    Bash,
    Slash,
    Heat,
    Cold,
    Plague,
    Radiant,
    Abyssal,
}

impl DamageVariant {
    pub const VALUES: [Self; 7] = [
        Self::Bash,
        Self::Slash,
        Self::Heat,
        Self::Cold,
        Self::Plague,
        Self::Radiant,
        Self::Abyssal,
    ];

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Bash => "BSH",
            Self::Slash => "SLA",
            Self::Heat => "HOT",
            Self::Cold => "CLD",
            Self::Plague => "PLG",
            Self::Radiant => "RAD",
            Self::Abyssal => "ABY",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Bash => palettes::css::DARK_ORANGE.into(),
            Self::Slash => palettes::css::ORANGE.into(),
            Self::Heat => palettes::css::RED.into(),
            Self::Cold => palettes::css::AQUA.into(),
            Self::Plague => palettes::css::GREEN.into(),
            Self::Radiant => palettes::css::WHITE.into(),
            Self::Abyssal => palettes::css::BLACK.into(),
        }
    }
}

/// Damage.
#[derive(Component, Copy, Clone, Debug, Default)]
pub struct Damage {
    /// Damage type.
    pub ty: DamageVariant,
    /// Damage amount.
    pub value: f32,
    /// The `Entity` that dealt the damage.
    pub source: Option<Entity>,
}

impl std::ops::MulAssign<f32> for Damage {
    fn mul_assign(&mut self, rhs: f32) {
        self.value *= rhs;
    }
}

impl std::ops::Mul<f32> for Damage {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

/// Health. Is there anything more I can say?
///
/// Can't fall below zero.
#[derive(Component, Debug)]
#[require(HealthCap)]
pub struct Health(pub f32);

impl Default for Health {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Debug)]
pub struct HealthCap(pub f32);

impl Default for HealthCap {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Assigns resistances against `DamageVariant`s.
///
/// Scales linearly from no resist at `0.0` to full resist at `1.0`. All values default to `0.0`.
#[derive(Component, Debug, Default)]
pub struct Resist(pub HashMap<DamageVariant, f32>);

/// Applies damage to adjacent `Health` components.
///
/// If there is no adjacent `Health` component, applies to the first ancestor containing one.
#[derive(Component, Default)]
pub struct DamageBuffer(pub Vec<Damage>);

/// Applies resistance by scaling `Damage` values.
pub fn apply_resist(mut query: Query<(&mut DamageBuffer, &Resist)>) {
    for (mut damage_buf, Resist(resist)) in query.iter_mut() {
        for damage in damage_buf.0.iter_mut() {
            let r = resist.get(&damage.ty).unwrap_or(&0.0);
            damage.value *= 1.0 - r;
        }
    }
}

/// Applies damage values from `DamageBuffer`.
pub fn apply_damage_buffers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, &mut DamageBuffer), Without<Dead>>,
) {
    for (e_health, mut health, mut damage_buf) in query.iter_mut() {
        for damage in damage_buf.0.drain(0..) {
            health.0 = (health.0 - damage.value).max(0.0);
            info!("health: {}", health.0);

            if health.0 == 0.0 {
                commands.entity(e_health).insert(Dead {
                    killed_by: damage.source,
                });

                if let Some(e_killer) = damage.source {
                    commands.trigger(KilledSomething {
                        entity: e_killer,
                        killed: e_health,
                    });
                }

                return;
            }
        }
    }
}

/// PCs and NPCs with this are dead.
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct Dead {
    pub killed_by: Option<Entity>,
}

#[derive(EntityEvent)]
pub struct KilledSomething {
    pub entity: Entity,
    pub killed: Entity,
}
