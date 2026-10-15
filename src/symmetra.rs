use std::marker::PhantomData;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_asset_loader::prelude::*;

use crate::{
    actions::{
        Action, CardArt, InPlay, Target,
        tags::{self, *},
    },
    asset::GameState,
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::{FieldCell, OccupiedByUnit},
    turn::Player,
    unit::{Unit, UnitDesc},
};

pub struct SymmetraPlugin;

impl Plugin for SymmetraPlugin {
    fn build(&self, app: &mut App) {
        app.configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<SymmetraAssets>(),
        )
        .init_resource::<HexGlow>()
        .add_plugins((
            GlyphPlugin::<SummonRedGlyph, RedGlyph>::default(),
            GlyphPlugin::<SummonBlueGlyph, BlueGlyph>::default(),
            GlyphPlugin::<SummonGreenGlyph, GreenGlyph>::default(),
        ));
    }
}

/// Implements glyph behavior for `T`.
///
/// This includes damage, cell rendering, and a corresponding basic placement card `A`.
pub struct GlyphPlugin<A: Component, T: SimpleGlyph> {
    pub phantom_data0: PhantomData<A>,
    pub phantom_data1: PhantomData<T>,
}

impl<A: Component, T: SimpleGlyph> Default for GlyphPlugin<A, T> {
    fn default() -> Self {
        Self {
            phantom_data0: PhantomData,
            phantom_data1: PhantomData,
        }
    }
}

impl<A: Component, T: SimpleGlyph> Plugin for GlyphPlugin<A, T> {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexGlowColor<T>>()
            .add_observer(glyph_on_cell_enter::<T>)
            .add_observer(place_glyphs_on_action::<A, T>)
            .add_observer(render_glyphs::<T>)
            .add_observer(derender_glyphs::<T>);
    }
}

#[derive(AssetCollection, Resource)]
struct SymmetraAssets {
    #[asset(path = "gltf/symmetra.glb")]
    pub _model: Handle<Gltf>,
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

pub trait SimpleGlyph: Component + Default {
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
    let Ok((glyph, _glyph_ty, occupied_by)) = cell_query.get(trigger.entity) else {
        return;
    };

    let mut damage = T::damage();
    damage.source = Some(glyph.owner);

    damage_events.write(DamageEvent {
        damage,
        target: occupied_by.iter().next().unwrap(),
    });
}

#[derive(Resource)]
pub struct HexGlow(pub Handle<Mesh>);

impl FromWorld for HexGlow {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let polygon = RegularPolygon::new(0.5, 6);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut colors = Vec::new();

        for (i, point) in polygon.vertices(0.0).into_iter().enumerate() {
            vertices.push(point.extend(0.0));
            vertices.push(point.extend(0.1));

            indices.push(i as u32 * 2);
            indices.push(i as u32 * 2 + 1);

            colors.push([1.0, 1.0, 1.0, 1.0]);
            colors.push([1.0, 1.0, 1.0, 0.0]);
        }

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices));

        let hex_glow = meshes.add(mesh);
        HexGlow(hex_glow)
    }
}

#[derive(Resource)]
pub struct HexGlowColor<T: SimpleGlyph>(pub Handle<StandardMaterial>, pub PhantomData<T>);

impl<T: SimpleGlyph> FromWorld for HexGlowColor<T> {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let h_material = materials.add(StandardMaterial {
            base_color: T::damage().ty.color(),
            unlit: true,
            ..Default::default()
        });
        Self(h_material, PhantomData)
    }
}

pub fn render_glyphs<T: SimpleGlyph>(
    trigger: On<Add, T>,
    mut commands: Commands,
    hex_glow: Res<HexGlow>,
    hex_glow_color: Res<HexGlowColor<T>>,
) {
    commands.entity(trigger.entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(hex_glow.0.clone()),
            MeshMaterial3d(hex_glow_color.0.clone()),
        ));
    });
}

pub fn derender_glyphs<T: SimpleGlyph>(trigger: On<Remove, T>, mut commands: Commands) {
    // TODO: despawn specifically this child I'm too lazy to write it
    commands.entity(trigger.entity).despawn_children();
}
