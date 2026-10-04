use bevy::{color::palettes, platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::asset::GameState;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<DamageAssets>(),
        )
        .add_systems(
            Update,
            (apply_resist, apply_damage_buffers, die)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
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
    Physical,
    Magical,

    Heat,
    Cold,
    Plague,

    Radiant,
    Abyssal,
}

impl DamageVariant {
    pub const VALUES: [Self; 7] = [
        Self::Physical,
        Self::Magical,
        Self::Heat,
        Self::Cold,
        Self::Plague,
        Self::Radiant,
        Self::Abyssal,
    ];

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Physical => "PHY",
            Self::Magical => "MAG",
            Self::Heat => "HOT",
            Self::Cold => "CLD",
            Self::Plague => "PLG",
            Self::Radiant => "RAD",
            Self::Abyssal => "ABY",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Physical => palettes::css::ORANGE.into(),
            Self::Magical => palettes::css::AQUA.into(),
            Self::Heat => palettes::css::RED.into(),
            Self::Cold => palettes::css::BLUE.into(),
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

/// Health. Is there anything more I can say?
///
/// Can't fall below zero.
#[derive(Component, Debug)]
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
pub fn apply_damage_buffers(mut query: Query<(&mut Health, &mut DamageBuffer), Without<Dead>>) {
    for (mut health, mut damage_buf) in query.iter_mut() {
        for damage in damage_buf.0.drain(0..) {
            health.0 = (health.0 - damage.value).max(0.0);
            info!("health: {}", health.0);
        }
    }
}

/// Inserts `Dead` component.
pub fn die(mut commands: Commands, health_query: Query<(Entity, &Health)>) {
    for (entity, health) in health_query.iter() {
        if health.0 == 0.0 {
            commands.entity(entity).insert(Dead);
        }
    }
}

/// PCs and NPCs with this are dead.
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct Dead;
