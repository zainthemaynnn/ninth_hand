use bevy::prelude::*;

use crate::field::FieldCell;

#[derive(Component)]
pub struct Symmetra;

#[derive(Component)]
#[require(FieldCell)]
pub enum Glyph {
	Red,
	Blue,
	Green,
}
