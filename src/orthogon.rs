use bevy::prelude::*;

use crate::field::FieldCell;

#[derive(Component)]
#[require(FieldCell)]
pub enum Glyph {
	Red,
	Blue,
	Green,
}
