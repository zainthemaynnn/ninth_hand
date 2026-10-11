use bevy::{mesh::ExtrusionBuilder, prelude::*};
use bevy_tweening::{AnimTarget, RepeatCount, RepeatStrategy, Tween, TweenAnim};
use std::{
    collections::HashMap,
    f32::consts::{FRAC_PI_2, FRAC_PI_6, PI, SQRT_3},
    marker::PhantomData,
    time::Duration,
};

use crate::{
    asset::{GameState, MaterialEmissiveLens},
    systems::{enforce_one, insert_on_hover, toggle_insert_on_click, uninsert_on_unhover},
    turn::{Cpu, Player, Team},
};

pub struct FieldPlugin;

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexMesh>()
            .init_resource::<HighlighterMaterials>()
            .init_resource::<Field<Player>>()
            .init_resource::<Field<Cpu>>()
            .add_systems(
                OnEnter(GameState::AssetLoading),
                animate_highlighter_materials,
            )
            .add_observer(toggle_insert_on_click::<SelectedCell, With<FieldCell>>)
            .add_observer(insert_on_hover::<HoveredCell, With<FieldCell>>)
            .add_observer(uninsert_on_unhover::<HoveredCell, With<FieldCell>>)
            .add_observer(enforce_one::<SelectedCell>)
            // not sure if this is still needed yet; might still be short periods where two exist
            // .add_observer(enforce_one::<HoveredCell>)
            .add_observer(animate_highlighted_cell)
            .add_observer(deanimate_dehighlighted_cell)
            .add_observer(dehighlight_deselected_cell);
    }
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    pub const DIRECTIONS: [Self; 6] = [
        Self { q: 1, r: 0 },
        Self { q: 1, r: -1 },
        Self { q: 0, r: -1 },
        Self { q: -1, r: 0 },
        Self { q: -1, r: 1 },
        Self { q: 0, r: 1 },
    ];

    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn neighbor(&self, direction: HexCoord) -> Self {
        Self {
            q: self.q + direction.q,
            r: self.r + direction.r,
        }
    }

    pub fn p(&self) -> i32 {
        -self.q - self.r
    }

    pub fn row(&self) -> i32 {
        self.r.abs() - (self.q + 1) / 2
    }

    pub fn frac_row(&self) -> f32 {
        self.r.abs() as f32 - self.q as f32 / 2.0
    }

    pub fn directly_behind(&self, other: &HexCoord) -> bool {
        self.q == other.q && self.r < other.r
    }
}

impl std::ops::Add for HexCoord {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

impl<'a, 'b> std::ops::Add<&'b HexCoord> for &'a HexCoord {
    type Output = HexCoord;

    fn add(self, rhs: &'b HexCoord) -> Self::Output {
        HexCoord {
            q: self.q + rhs.q,
            r: self.r + rhs.r,
        }
    }
}

/// The `FieldCell` occupied by this unit.
#[derive(Component, Debug)]
#[relationship(relationship_target = OccupiedByUnit)]
pub struct OccupyingCell(pub Entity);

/// The `Unit` occupying this cell.
#[derive(Component, Debug)]
#[relationship_target(relationship = OccupyingCell)]
pub struct OccupiedByUnit(Entity);

#[derive(Component, Default)]
#[require(HexCoord, Pickable)]
pub struct FieldCell;

#[derive(Resource)]
pub struct HexMesh(pub Handle<Mesh>);

impl FromWorld for HexMesh {
    fn from_world(world: &mut World) -> Self {
        let hex_2d = RegularPolygon::new(0.5, 6);
        let hex_mesh = ExtrusionBuilder::new(&hex_2d, 0.1);

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let handle = meshes.add(hex_mesh);

        HexMesh(handle)
    }
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
#[require(
    HighlightedCell { ty: Highlighter::Red },
)]
pub struct SelectedCell;

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct HoveredCell;

#[derive(Component, Copy, Clone, Default, Hash, Eq, PartialEq)]
pub enum Highlighter {
    #[default]
    White,
    Gray,
    Red,
    Blue,
    Green,
}

impl Highlighter {
    pub const VALUES: [Self; 5] = [Self::White, Self::Gray, Self::Red, Self::Blue, Self::Green];
}

impl From<Highlighter> for LinearRgba {
    fn from(value: Highlighter) -> Self {
        match value {
            Highlighter::White => LinearRgba::WHITE,
            Highlighter::Gray => LinearRgba::gray(0.2),
            Highlighter::Red => LinearRgba::RED,
            Highlighter::Blue => LinearRgba::BLUE,
            Highlighter::Green => LinearRgba::GREEN,
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HighlightedCell {
    pub ty: Highlighter,
}

#[derive(Resource)]
pub struct HighlighterMaterials(pub HashMap<Highlighter, Handle<StandardMaterial>>);

impl FromWorld for HighlighterMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let _handle = materials.add(StandardMaterial::default());
        HighlighterMaterials(HashMap::from(
            Highlighter::VALUES.map(|v| (v, materials.add(StandardMaterial::default()))),
        ))
    }
}

/// Contains all the `FieldCell`s on the field, mapped by `HexCoord`.
#[derive(Resource)]
pub struct Field<T: Team> {
    pub cells: HashMap<HexCoord, Entity>,
    pub origin_transform: Transform,
    pub hex_radius: f32,
    pub phantom_data: PhantomData<T>,
}

impl<T: Team> Default for Field<T> {
    fn default() -> Self {
        Self {
            hex_radius: 2.0,
            cells: HashMap::default(),
            origin_transform: Transform::default().with_rotation(Quat::from_euler(
                EulerRot::YXZ,
                PI,
                0.0,
                0.0,
            )),
            phantom_data: PhantomData::default(),
        }
    }
}

impl<T: Team> Field<T> {
    pub fn add_cell(&mut self, e_cell: Entity, position: HexCoord) {
        self.cells.insert(position, e_cell);
    }

    pub fn adjacent(&self, position: HexCoord) -> Vec<Entity> {
        let mut adj = Vec::new();
        for direction in HexCoord::DIRECTIONS {
            if let Some(e_cell) = self.cells.get(&(position + direction)) {
                adj.push(*e_cell);
            }
        }
        return adj;
    }

    pub fn to_world_space(&self, coord: HexCoord) -> Transform {
        self.origin_transform
            * Transform::from_xyz(
                self.hex_radius * 3. / 2. * coord.q as f32,
                0.0,
                self.hex_radius * (SQRT_3 / 2. * coord.q as f32 + SQRT_3 * coord.r as f32),
            )
    }

    pub fn transform_hex(&self, coord: HexCoord) -> Transform {
        self.to_world_space(coord)
            * Transform::default()
                .with_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_2, 0.0, FRAC_PI_6))
                .with_scale(Vec3::new(
                    self.hex_radius / 0.5 * 0.9,
                    self.hex_radius / 0.5 * 0.9,
                    1.0,
                ))
    }
}

pub fn load_field<T: Team>(
    In((tf, coords)): In<(Transform, Vec<HexCoord>)>,
    mut commands: Commands,
    mut field: ResMut<Field<T>>,
    hex_mesh: Res<HexMesh>,
) {
    field.origin_transform = tf;

    let h_mesh = &hex_mesh.0;
    for coord in &coords {
        let e_cell = commands
            .spawn((
                FieldCell,
                *coord,
                Mesh3d(h_mesh.clone()),
                MeshMaterial3d::<StandardMaterial>::default(),
                field.transform_hex(*coord),
            ))
            .id();
        field.add_cell(e_cell, *coord);
    }

    info!("loaded hex cells {:?}", &coords);
}

fn animate_highlighter_materials(mut commands: Commands, materials: Res<HighlighterMaterials>) {
    for (v, material) in materials.0.iter() {
        let selection_tween = Tween::new(
            EaseFunction::SineInOut,
            Duration::from_millis(1000),
            MaterialEmissiveLens {
                start: LinearRgba::BLACK,
                end: LinearRgba::from(*v),
            },
        )
        .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);

        commands.spawn((
            TweenAnim::new(selection_tween),
            AnimTarget::asset(material.id()),
        ));
    }
}

fn animate_highlighted_cell(
    trigger: On<Add, HighlightedCell>,
    mut commands: Commands,
    materials: Res<HighlighterMaterials>,
    highlight_query: Query<&HighlightedCell>,
) {
    info!("highlighting cell");
    let highlight = highlight_query.get(trigger.entity).unwrap();
    commands.entity(trigger.entity).insert(MeshMaterial3d::from(
        materials.0.get(&highlight.ty).unwrap().clone(),
    ));
}

fn deanimate_dehighlighted_cell(trigger: On<Remove, HighlightedCell>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(MeshMaterial3d::<StandardMaterial>::default());
}

fn dehighlight_deselected_cell(trigger: On<Remove, SelectedCell>, mut commands: Commands) {
    commands.entity(trigger.entity).remove::<HighlightedCell>();
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn adjacency() {
        let mut app = App::new();

        app.insert_resource(Field::<Player>::default());

        let coords = [
            HexCoord { q: 0, r: 0 },
            HexCoord { q: 1, r: 0 },
            HexCoord { q: 0, r: 1 },
            HexCoord { q: 3, r: 0 },
        ];

        for coord in coords {
            let e_cell = app.world_mut().spawn((coord,)).id();

            app.world_mut()
                .resource_mut::<Field<Player>>()
                .add_cell(e_cell, coord);
        }

        let field = app.world_mut().resource::<Field<Player>>();
        assert!(field.adjacent(coords[0]).len() == 2);
        assert!(field.adjacent(coords[1]).len() == 2);
        assert!(field.adjacent(coords[2]).len() == 2);
        assert!(field.adjacent(coords[3]).len() == 0);
    }
}
