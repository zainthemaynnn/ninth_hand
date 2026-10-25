use std::{marker::PhantomData, time::Duration};

use bevy::{
    asset::RenderAssetUsages,
    ecs::relationship::Relationship,
    gltf::GltfMaterialName,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashMap,
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use bevy_asset_loader::prelude::*;
use bevy_tweening::{AnimTarget, Tween, TweenAnim};
use rand::RngExt;

use crate::{
    actions::{
        Action, ActionType, CardArt, OwnedByUnit, Played, Target,
        tags::{self, *},
    },
    asset::{GameState, MaterialBaseColorLens},
    damage::{Damage, DamageEvent, DamageVariant, Health, HealthCap, Resist},
    field::{FieldCell, OccupiedByUnit},
    turn::Player,
    unit::{Unit, UnitAnimations, UnitColor, UnitDesc, UnitModelResource, UnitPriority, UnitType},
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
        ))
        .add_systems(OnEnter(GameState::InGame), update_symmetra_color);
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
            .init_resource::<HexShape<T>>()
            .add_observer(glyph_on_cell_enter::<T>)
            .add_observer(render_glyphs::<T>)
            .add_observer(derender_glyphs::<T>);
    }
}

#[derive(AssetCollection, Resource)]
pub struct SymmetraAssets {
    #[asset(path = "gltf/symmetra.glb#Scene0")]
    pub symmetra: Handle<WorldAsset>,
    #[asset(path = "gltf/symmetra.glb#Material0/std")]
    pub cloak: Handle<StandardMaterial>,
    #[asset(path = "gltf/symmetra.glb#Animation0")]
    pub idle: Handle<AnimationClip>,
    #[asset(path = "audio/glyph.wav")]
    pub glyph: Handle<AudioSource>,
}

impl UnitModelResource for SymmetraAssets {
    fn model_handle(&self) -> Handle<WorldAsset> {
        self.symmetra.clone()
    }
}

#[derive(Component)]
pub struct SymmetraAnimations {
    pub idle: AnimationNodeIndex,
}

impl UnitAnimations for SymmetraAnimations {
    type Assets = SymmetraAssets;

    fn new(assets: &Self::Assets, graph: &mut AnimationGraph) -> Self {
        Self {
            idle: graph.add_clip(assets.idle.clone(), 1.0, graph.root),
        }
    }

    fn idle(&self) -> Option<Vec<AnimationNodeIndex>> {
        Some(vec![self.idle])
    }
}

#[derive(Component, Default)]
#[require(
    Unit,
    UnitPriority(1),
    Player,
    UnitDesc {
        name: "Symmetra",
        passive: "Every turn, guaranteed to draw a random extra glyph.",
        flavor: "Mortals are stupid. Geometry is not.",
    },
    UnitColor(Srgba::hex("#C23EA4").unwrap().into()), // #780EC2 accent??
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

impl UnitType for Symmetra {
    type ModelResource = SymmetraAssets;
    type Animations = SymmetraAnimations;
}

#[derive(Component)]
pub struct Cloak(pub Entity);

#[derive(Component)]
#[require(FieldCell)]
pub struct Glyph {
    pub owner: Entity,
}

pub trait SimpleGlyph: Component + Default {
    const SHAPE_N: u32;

    fn damage() -> Damage;
}

#[derive(Component, Default)]
pub struct RedGlyph;

impl SimpleGlyph for RedGlyph {
    const SHAPE_N: u32 = 4;

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
    const SHAPE_N: u32 = 32;

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
    const SHAPE_N: u32 = 3;

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

impl ActionType for SummonRedGlyph {
    fn play() -> Observer {
        Observer::new(
            |trigger: On<Played>,
             commands: Commands,
             target_query: Query<Entity, With<Target>>,
             action_query: Query<&OwnedByUnit>,
             chroma_query: Query<(&UnitColor, &Chroma)>| {
                place_glyphs_on_action::<RedGlyph>(
                    trigger,
                    commands,
                    target_query,
                    action_query,
                    chroma_query,
                );
            },
        )
    }
}

#[derive(Component)]
#[require(
	Action { name: "Blue Glyph", description: "Place blue glyph (CLD 1)" },
	CellTarget,
	CellEffect,
	tags::Glyph,
	CardArt("shapes"),
)]
pub struct SummonBlueGlyph;

impl ActionType for SummonBlueGlyph {
    fn play() -> Observer {
        Observer::new(
            |trigger: On<Played>,
             commands: Commands,
             target_query: Query<Entity, With<Target>>,
             action_query: Query<&OwnedByUnit>,
             chroma_query: Query<(&UnitColor, &Chroma)>| {
                place_glyphs_on_action::<BlueGlyph>(
                    trigger,
                    commands,
                    target_query,
                    action_query,
                    chroma_query,
                );
            },
        )
    }
}

#[derive(Component)]
#[require(
	Action { name: "Green Glyph", description: "Place green glyph (PLG 1)" },
	CellTarget,
	CellEffect,
	tags::Glyph,
	CardArt("shapes"),
)]
pub struct SummonGreenGlyph;

impl ActionType for SummonGreenGlyph {
    fn play() -> Observer {
        Observer::new(
            |trigger: On<Played>,
             commands: Commands,
             target_query: Query<Entity, With<Target>>,
             action_query: Query<&OwnedByUnit>,
             chroma_query: Query<(&UnitColor, &Chroma)>| {
                place_glyphs_on_action::<GreenGlyph>(
                    trigger,
                    commands,
                    target_query,
                    action_query,
                    chroma_query,
                );
            },
        )
    }
}

pub fn place_glyphs_on_action<T: SimpleGlyph + Default>(
    trigger: On<Played>,
    mut commands: Commands,
    target_query: Query<Entity, With<Target>>,
    action_query: Query<&OwnedByUnit>,
    chroma_query: Query<(&UnitColor, &Chroma)>,
) {
    let e_owner = action_query.get(trigger.entity).unwrap().get();
    for e_target in target_query {
        commands
            .entity(e_target)
            .insert((T::default(), Glyph { owner: e_owner }));

        let (unit_color, chroma) = chroma_query.get(e_owner).unwrap();

        let tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(100),
            MaterialBaseColorLens {
                start: unit_color.0.to_linear(),
                end: T::damage().ty.color().to_linear(),
            },
        )
        .then(Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(500),
            MaterialBaseColorLens {
                start: T::damage().ty.color().to_linear(),
                end: unit_color.0.to_linear(),
            },
        ));

        commands.entity(chroma.0).insert(TweenAnim::new(tween));
    }
}

pub fn glyph_on_cell_enter<T: SimpleGlyph>(
    trigger: On<Add, OccupiedByUnit>,
    mut commands: Commands,
    cell_query: Query<(Entity, &Glyph, &OccupiedByUnit), With<T>>,
    mut damage_events: MessageWriter<DamageEvent>,
    assets: Res<SymmetraAssets>,
) {
    let Ok((e_glyph, glyph, occupied_by)) = cell_query.get(trigger.entity) else {
        return;
    };

    let mut damage = T::damage();
    damage.source = Some(glyph.owner);

    damage_events.write(DamageEvent {
        damage,
        target: occupied_by.iter().next().unwrap(),
    });

    commands.entity(e_glyph).insert((
        AudioPlayer::new(assets.glyph.clone()),
        PlaybackSettings::REMOVE,
    ));
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
            vertices.push(point.extend(-1.0));

            indices.push(i as u32 * 2);
            indices.push(i as u32 * 2 + 1);

            colors.push([1.0, 1.0, 1.0, 1.0]);
            colors.push([1.0, 1.0, 1.0, 0.0]);
        }

        indices.push(0u32);
        indices.push(1u32);

        colors.push([1.0, 1.0, 1.0, 1.0]);
        colors.push([1.0, 1.0, 1.0, 0.0]);

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices));

        let hex_glow = meshes.add(mesh);
        Self(hex_glow)
    }
}

#[derive(Resource)]
pub struct HexShape<T: SimpleGlyph>(pub Handle<Mesh>, PhantomData<T>);

impl<T: SimpleGlyph> FromWorld for HexShape<T> {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let inner = RegularPolygon::new(0.25, T::SHAPE_N);
        let outer = RegularPolygon::new(0.35, T::SHAPE_N);

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let rot = rand::rng().random_range(0.0..std::f32::consts::PI);

        for (i, (p0, p1)) in inner
            .vertices(rot)
            .into_iter()
            .zip(outer.vertices(rot).into_iter())
            .enumerate()
        {
            vertices.push(p0.extend(0.01));
            vertices.push(p1.extend(0.01));

            indices.push(i as u32 * 2);
            indices.push(i as u32 * 2 + 1);
        }

        indices.push(0u32);
        indices.push(1u32);

        let mesh = Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_indices(Indices::U32(indices));

        let hex_shape = meshes.add(mesh);
        Self(hex_shape, PhantomData)
    }
}

#[derive(Resource)]
pub struct HexGlowColor<T: SimpleGlyph>(pub Handle<StandardMaterial>, pub PhantomData<T>);

impl<T: SimpleGlyph> FromWorld for HexGlowColor<T> {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let h_material = materials.add(StandardMaterial {
            base_color: T::damage().ty.color(),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..Default::default()
        });
        Self(h_material, PhantomData)
    }
}

pub fn render_glyphs<T: SimpleGlyph>(
    trigger: On<Add, T>,
    mut commands: Commands,
    hex_glow: Res<HexGlow>,
    hex_shape: Res<HexShape<T>>,
    hex_glow_color: Res<HexGlowColor<T>>,
) {
    info!("spawning glyph");
    commands.entity(trigger.entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(hex_glow.0.clone()),
            MeshMaterial3d(hex_glow_color.0.clone()),
        ));
        parent.spawn((
            Mesh3d(hex_shape.0.clone()),
            MeshMaterial3d(hex_glow_color.0.clone()),
        ));
    });
}

pub fn derender_glyphs<T: SimpleGlyph>(trigger: On<Remove, T>, mut commands: Commands) {
    // TODO: despawn specifically this child I'm too lazy to write it
    commands.entity(trigger.entity).despawn_children();
}

#[derive(Component, Debug)]
pub struct Chroma(pub Entity);

pub fn setup_symmetra_model(
    trigger: On<WorldInstanceReady>,
    mut commands: Commands,
    children_query: Query<&Children>,
    name_query: Query<&GltfMaterialName>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    let e_cloak =
        crate::systems::find_named_material(trigger.entity, "cloak", &children_query, &name_query)
            .unwrap();
    let e_anim = commands
        .spawn(AnimTarget::asset(material_query.get(e_cloak).unwrap().id()))
        .id();
    commands.entity(trigger.entity).insert(Chroma(e_anim));
}

pub fn update_symmetra_color(
    assets: Res<SymmetraAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("shift");
    let mut material = materials.get_mut(assets.cloak.id()).unwrap();
    material.base_color = Srgba::hex("#C23EA4").unwrap().into();
}
