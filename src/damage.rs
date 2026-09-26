use std::collections::HashMap;

use bevy::prelude::*;

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

/// Assigns resistances against `DamageVariant`s.
///
/// Scales linearly from no resist at `0.0` to full resist at `1.0`. All values default to `0.0`.
#[derive(Component, Debug, Default)]
pub struct Resist(HashMap<DamageVariant, f32>);

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
