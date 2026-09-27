use bevy::prelude::*;

use crate::{
    damage::{DamageBuffer, Health, Resist},
    field::{Field, HexCoord, OccupyingCell},
};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, reposition);
    }
}

#[derive(Component, Default)]
#[require(
    Health,
    Resist,
    DamageBuffer,
    Transform,
    OccupyingCell(Entity::PLACEHOLDER)
)]
pub struct Unit;

pub struct UnitDesc {
    pub name: &'static str,
    pub passive: &'static str,
}

#[derive(Resource)]
pub struct ActiveUnits(pub Vec<UnitType>);

pub enum UnitType {
	Maw,
	Symmetra,
}

pub fn reposition(
    field: Res<Field>,
    q_moved_units: Query<(&mut Transform, &OccupyingCell), Changed<OccupyingCell>>,
    q_hexcoord: Query<&HexCoord>,
) {
    for (mut tf, OccupyingCell(e_cell)) in q_moved_units {
        let cell_hexcoord = q_hexcoord.get(*e_cell).unwrap();
        *tf = field.to_world_space(*cell_hexcoord);
        info!("unit moved to {:?}", cell_hexcoord);
    }
}

pub struct Intent {
    pub name: &'static str,
}
