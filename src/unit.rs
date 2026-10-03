use bevy::prelude::*;

use crate::{
    asset::GameState,
    damage::{DamageAssets, DamageBuffer, DamageVariant, Health, HealthCap, Resist},
    field::{Field, HexCoord, OccupyingCell},
    turn::{Cpu, Player, Team},
};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_unit_details_window)
            .add_systems(Update, (reposition::<Player>, reposition::<Cpu>))
            .add_observer(show_unit_details);
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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedUnit;

#[derive(Resource)]
pub struct ActiveUnits(pub Vec<Entity>);

pub fn reposition<T: Team + Component>(
    field: Res<Field<T>>,
    q_moved_units: Query<(&mut Transform, &OccupyingCell), (Changed<OccupyingCell>, With<T>)>,
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
