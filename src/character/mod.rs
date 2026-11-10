pub mod animus;
pub mod fractal;
pub mod grease;
pub mod impi;
pub mod mary;
pub mod nine;
pub mod presence;
pub mod symmetra;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub struct CharacterPlugins;

impl PluginGroup for CharacterPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(nine::NinePlugin)
            .add(symmetra::SymmetraPlugin)
            .add(impi::ImpiPlugin)
            .add(animus::AnimusPlugin)
            .add(grease::GreasePlugin)
            .add(presence::PresencePlugin)
            .add(mary::MaryPlugin)
            .add(fractal::FractalPlugin)
    }
}
