use bevy::prelude::*;

use crate::actions::target::CanTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct SingleTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct MultiTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct FrontTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct BackTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct EnemyTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct AllyTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct AnyTarget;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct CellTarget;

#[derive(Component, Default)]
pub struct Pierce;

#[derive(Component, Default)]
pub struct LineOfSight;

#[derive(Component, Default)]
pub struct Damage;

#[derive(Component, Default)]
pub struct Heal;

#[derive(Component, Default)]
pub struct Shield;

#[derive(Component, Default)]
pub struct Buff;

#[derive(Component, Default)]
pub struct Debuff;

#[derive(Component, Default)]
pub struct MultiHit;

#[derive(Component, Default)]
pub struct CellEffect;

#[derive(Component, Default)]
pub struct Glyph;

#[derive(Component, Default)]
#[require(CanTarget)]
pub struct HasSecondaryTarget;

#[derive(Component, Default)]
#[require(HasSecondaryTarget)]
pub struct AdjacentSecondaryTarget;
