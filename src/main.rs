#![feature(more_float_constants)]

use bevy::{
    color::palettes::css::SILVER,
    gltf::{GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::{
    actions::{ActionPlugin, AvailableAction, OwnedByUnit},
    asset::{GameState, NineAssetPlugin},
    damage::{DamagePlugin, Health},
    director::DirectorPlugin,
    enemy::{EnemyAssets, EnemyPlugin, mortal::Mortal},
    field::{Field, FieldPlugin, HexCoord, OccupyingCell, load_field},
    nine::Nine,
    status::StatusPlugin,
    symmetra::SymmetraPlugin,
    turn::{Cpu, Player, TurnPlugin},
    ui::NineUiPlugin,
    unit::UnitPlugin,
};

pub mod actions;
pub mod asset;
pub mod damage;
pub mod director;
pub mod enemy;
pub mod field;
pub mod nine;
pub mod status;
pub mod symmetra;
pub mod systems;
pub mod turn;
pub mod ui;
pub mod unit;

#[derive(AssetCollection, Resource)]
struct NineAssets {
    #[asset(path = "gltf/nine.glb")]
    nine: Handle<Gltf>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(GltfPlugin {
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    rotate_meshes: false,
                },
                ..default()
            }),
            NineAssetPlugin,
            FieldPlugin,
            UnitPlugin,
            ActionPlugin,
            TweeningPlugin,
            MeshPickingPlugin,
            TurnPlugin,
            NineUiPlugin,
            DamagePlugin,
            EnemyPlugin,
            SymmetraPlugin,
            StatusPlugin,
            DirectorPlugin,
        ))
        .configure_loading_state(
            LoadingStateConfig::new(GameState::AssetLoading).load_collection::<NineAssets>(),
        )
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ray_cast_visibility: RayCastVisibility::Any,
        })
        .add_systems(
            OnEnter(GameState::InGame),
            (
                (|| {
                    (
                        Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
                        field_coords(),
                    )
                })
                .pipe(load_field::<Player>),
                (|| {
                    (
                        Transform::from_xyz(0.0, 0.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
                        field_coords(),
                    )
                })
                .pipe(load_field::<Cpu>),
                entry,
            )
                .chain(),
        )
        .run();
}

fn field_coords() -> Vec<HexCoord> {
    vec![
        HexCoord::new(-3, 1),
        HexCoord::new(-3, 0),
        HexCoord::new(-3, -1),
        HexCoord::new(-3, -2),
        HexCoord::new(-2, 1),
        HexCoord::new(-2, 0),
        HexCoord::new(-2, -1),
        HexCoord::new(-2, -2),
        HexCoord::new(-1, 0),
        HexCoord::new(-1, -1),
        HexCoord::new(-1, -2),
        HexCoord::new(-1, -3),
        HexCoord::new(0, 0),
        HexCoord::new(0, -1),
        HexCoord::new(0, -2),
        HexCoord::new(0, -3),
        HexCoord::new(1, -1),
        HexCoord::new(1, -2),
        HexCoord::new(1, -3),
        HexCoord::new(1, -4),
        HexCoord::new(2, -1),
        HexCoord::new(2, -2),
        HexCoord::new(2, -3),
        HexCoord::new(2, -4),
        HexCoord::new(3, -2),
        HexCoord::new(3, -3),
        HexCoord::new(3, -4),
        HexCoord::new(3, -5),
    ]
}

fn entry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltfs: Res<Assets<Gltf>>,
    nine_assets: Res<NineAssets>,
    enemy_assets: Res<EnemyAssets>,
    player_field: Res<Field<Player>>,
    cpu_field: Res<Field<Cpu>>,
    _tf_query: Query<&GlobalTransform>,
) {
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    commands.spawn((
        Camera3d::default(),
        MeshPickingCamera,
        Transform::from_xyz(20.0, 40.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let gltf = gltfs.get(&nine_assets.nine).unwrap();
    let h_scene = gltf.default_scene.as_ref().unwrap().clone();
    let e_cell = player_field.cells.get(&HexCoord::new(0, 0)).unwrap();
    let e_nine = commands
        .spawn((WorldAssetRoot(h_scene), Nine, OccupyingCell(*e_cell)))
        .id();

    let e_cell = cpu_field.cells.get(&HexCoord::new(1, -1)).unwrap();
    let _e_mortal = commands
        .spawn((
            WorldAssetRoot(enemy_assets.mortal.clone()),
            Mortal,
            OccupyingCell(*e_cell),
            Health(0.7),
        ))
        .id();
    let e_cell2 = cpu_field.cells.get(&HexCoord::new(0, -3)).unwrap();
    let _e_mortal2 = commands
        .spawn((
            WorldAssetRoot(enemy_assets.mortal.clone()),
            Mortal,
            OccupyingCell(*e_cell2),
            Health(0.7),
        ))
        .id();

    dbg!(HexCoord::new(1, -1).row());
    dbg!(HexCoord::new(0, -3).row());

    commands.spawn((crate::nine::Crush, AvailableAction, OwnedByUnit(e_nine)));
    commands.spawn((crate::nine::Slap, AvailableAction, OwnedByUnit(e_nine)));
}
