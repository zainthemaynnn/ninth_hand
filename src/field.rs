use bevy::{mesh::ExtrusionBuilder, prelude::*};
use bevy_tweening::{AnimTarget, RepeatCount, RepeatStrategy, Tween, TweenAnim};
use std::{
    collections::HashMap, f32::consts::{FRAC_PI_2, FRAC_PI_6, PI, SQRT_3}, time::Duration,
};

use crate::asset::{GameAssetLoadState, MaterialEmissiveLens};

pub struct FieldPlugin;

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HexMesh>()
            .init_resource::<SelectedCellMaterial>()
            .init_resource::<Field>()
            .add_systems(
                OnEnter(GameAssetLoadState::AssetLoading),
                animate_selected_cell_material,
            )
            .add_observer(select_cell_on_click)
            .add_observer(deselect_preselected_cell)
            .add_observer(animate_selected_cell)
            .add_observer(deanimate_deselected_cell);
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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedCell;

#[derive(Resource)]
pub struct SelectedCellMaterial(pub Handle<StandardMaterial>);

impl FromWorld for SelectedCellMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let handle = materials.add(StandardMaterial::default());
        SelectedCellMaterial(handle)
    }
}

/// Contains all the `FieldCell`s on the field, mapped by `HexCoord`.
#[derive(Resource)]
pub struct Field {
    pub cells: HashMap<HexCoord, Entity>,
    pub origin_transform: Transform,
    pub hex_radius: f32,
}

impl Default for Field {
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
        }
    }
}

impl Field {
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

pub fn load_field(
    In(coords): In<Vec<HexCoord>>,
    mut commands: Commands,
    mut field: ResMut<Field>,
    hex_mesh: Res<HexMesh>,
) {
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

fn animate_selected_cell_material(mut commands: Commands, material: Res<SelectedCellMaterial>) {
    error!("tweening material");
    let selection_tween = Tween::new(
        EaseFunction::SineInOut,
        Duration::from_millis(1000),
        MaterialEmissiveLens {
            start: LinearRgba::BLACK,
            end: LinearRgba::RED,
        },
    )
    .with_repeat(RepeatCount::Infinite, RepeatStrategy::MirroredRepeat);

    commands.spawn((
        TweenAnim::new(selection_tween),
        AnimTarget::asset(material.0.id()),
    ));
}

fn select_cell_on_click(
    trigger: On<Pointer<Click>>,
    cell_query: Query<Option<&SelectedCell>, With<FieldCell>>,
    mut commands: Commands,
) {
    debug!("select");
    if let Ok(selected) = cell_query.get(trigger.entity) {
        if selected.is_none() {
            commands.entity(trigger.entity).insert(SelectedCell);
        } else {
            commands.entity(trigger.entity).remove::<SelectedCell>();
        }
    }
}

fn deselect_preselected_cell(
    trigger: On<Add, SelectedCell>,
    mut commands: Commands,
    selected_query: Query<Entity, With<SelectedCell>>,
) {
    debug!("deselect");
    for e_old in selected_query {
        if e_old == trigger.entity {
            continue;
        }

        commands.entity(e_old).remove::<SelectedCell>();
    }
}

fn animate_selected_cell(
    trigger: On<Add, SelectedCell>,
    mut commands: Commands,
    material: Res<SelectedCellMaterial>,
) {
    commands
        .entity(trigger.entity)
        .insert(MeshMaterial3d::from(material.0.clone()));
}

fn deanimate_deselected_cell(trigger: On<Remove, SelectedCell>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(MeshMaterial3d::<StandardMaterial>::default());
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn adjacency() {
        let mut app = App::new();

        app.insert_resource(Field::default());

        let coords = [
            HexCoord { q: 0, r: 0 },
            HexCoord { q: 1, r: 0 },
            HexCoord { q: 0, r: 1 },
            HexCoord { q: 3, r: 0 },
        ];

        for coord in coords {
            let e_cell = app.world_mut().spawn((coord,)).id();

            app.world_mut()
                .resource_mut::<Field>()
                .add_cell(e_cell, coord);
        }

        let field = app.world_mut().resource::<Field>();
        assert!(field.adjacent(coords[0]).len() == 2);
        assert!(field.adjacent(coords[1]).len() == 2);
        assert!(field.adjacent(coords[2]).len() == 2);
        assert!(field.adjacent(coords[3]).len() == 0);
    }
}
