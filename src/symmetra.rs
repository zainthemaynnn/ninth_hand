use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    actions::{Action, CardArt, InPlay, Target, tags::{self, *}},
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::{FieldCell, OccupiedByUnit},
    turn::{Ally, Cpu, Player, Rival, TurnState},
    unit::{Unit, UnitDesc},
};

pub struct SymmetraPlugin;

impl Plugin for SymmetraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(glyph_on_cell_enter::<RedGlyph>)
            .add_observer(glyph_on_cell_enter::<BlueGlyph>)
            .add_observer(glyph_on_cell_enter::<GreenGlyph>)
            .add_observer(place_glyphs_on_action::<SummonRedGlyph, RedGlyph>)
            .add_observer(place_glyphs_on_action::<SummonBlueGlyph, BlueGlyph>)
            .add_observer(place_glyphs_on_action::<SummonGreenGlyph, GreenGlyph>);
    }
}

#[derive(Component)]
#[require(
    Unit,
    Player,
    UnitDesc {
        name: "Symmetra",
        passive: "Every turn, guaranteed to draw a random extra glyph.",
        flavor: "Mortals are stupid. Geometry is not.",
    },
    Health(10.0),
    HealthCap(10.0),
    Resist(HashMap::from([
        (DamageVariant::Bash, 1.25),
        (DamageVariant::Slash, 1.25),
        (DamageVariant::Heat, 0.75),
        (DamageVariant::Cold, 0.75),
        (DamageVariant::Plague, 0.75),
    ])),
)]
pub struct Symmetra;

#[derive(Component)]
#[require(FieldCell)]
pub struct Glyph {
    pub owner: Entity,
}

pub trait SimpleGlyph: Component {
    fn damage() -> Damage;
}

#[derive(Component, Default)]
pub struct RedGlyph;

impl SimpleGlyph for RedGlyph {
    fn damage() -> Damage {
        Damage {
            value: 1.0,
            ty: DamageVariant::Heat,
            source: None,
        }
    }
}

#[derive(Component, Default)]
pub struct BlueGlyph;

impl SimpleGlyph for BlueGlyph {
    fn damage() -> Damage {
        Damage {
            value: 1.0,
            ty: DamageVariant::Cold,
            source: None,
        }
    }
}

#[derive(Component, Default)]
pub struct GreenGlyph;

impl SimpleGlyph for GreenGlyph {
    fn damage() -> Damage {
        Damage {
            value: 1.0,
            ty: DamageVariant::Plague,
            source: None,
        }
    }
}

#[derive(Component)]
#[require(
	Action { name: "Red Glyph", description: "Place red glyph (HOT 1)" },
	CellTarget,
	CellEffect,
	tags::Glyph,
	CardArt("shapes"),
)]
pub struct SummonRedGlyph;

#[derive(Component)]
#[require(
	Action { name: "Blue Glyph", description: "Place blue glyph (CLD 1)" },
	CellTarget,
	CellEffect,
	tags::Glyph,
	CardArt("shapes"),
)]
pub struct SummonBlueGlyph;

#[derive(Component)]
#[require(
	Action { name: "Green Glyph", description: "Place green glyph (PLG 1)" },
	CellTarget,
	CellEffect,
	tags::Glyph,
	CardArt("shapes"),
)]
pub struct SummonGreenGlyph;

pub fn place_glyphs_on_action<A: Component, T: SimpleGlyph + Default>(
    trigger: On<Add, InPlay>,
    mut commands: Commands,
    action_query: Query<(), With<A>>,
    target_query: Query<Entity, With<Target>>,
) {
    if !action_query.contains(trigger.entity) {
        return;
    }

    for e_target in target_query {
    	commands.entity(e_target).insert(T::default());
    }
}

pub fn glyph_on_cell_enter<T: SimpleGlyph>(
    trigger: On<Add, OccupiedByUnit>,
    cell_query: Query<(&Glyph, &T, &OccupiedByUnit), With<T>>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    let Ok((glyph, glyph_ty, occupied_by)) = cell_query.get(trigger.entity) else {
        return;
    };

    let mut damage = T::damage();
    damage.source = Some(glyph.owner);

    damage_events.write(DamageEvent {
        damage,
        target: occupied_by.iter().next().unwrap(),
    });
}
