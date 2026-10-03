#![feature(more_float_constants)]
use std::f32::consts::PI;

use bevy::{color::palettes::css::SILVER, prelude::*};
use bevy_asset_loader::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::{
    actions::ActionPlugin, asset::{GameState, NineAssetPlugin}, field::{Field, FieldPlugin, HexCoord, OccupyingCell, load_field}, turn::{Cpu, Player, TurnPlugin}, ui::NineUiPlugin, unit::{Unit, UnitPlugin},
};

pub mod actions;
pub mod asset;
pub mod damage;
pub mod field;
pub mod nine;
pub mod symmetra;
pub mod turn;
pub mod ui;
pub mod unit;

#[derive(AssetCollection, Resource)]
struct NineAssets {
    #[asset(path = "gltf/nine.glb")]
    nine: Handle<Gltf>,
    #[asset(path = "gltf/symmetra.glb")]
    symmetra: Handle<Gltf>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            NineAssetPlugin,
            FieldPlugin,
            UnitPlugin,
            ActionPlugin,
            TweeningPlugin,
            MeshPickingPlugin,
            TurnPlugin,
            NineUiPlugin,
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
                        Transform::from_rotation(Quat::from_rotation_y(PI)),
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
        HexCoord::new(-2, 1),
        HexCoord::new(-2, 2),
        HexCoord::new(-2, 3),
        HexCoord::new(-2, 4),
        HexCoord::new(-1, 1),
        HexCoord::new(-1, 2),
        HexCoord::new(-1, 3),
        HexCoord::new(0, 0),
        HexCoord::new(0, 1),
        HexCoord::new(0, 2),
        HexCoord::new(0, 3),
        HexCoord::new(1, 0),
        HexCoord::new(1, 1),
        HexCoord::new(1, 2),
        HexCoord::new(2, -1),
        HexCoord::new(2, 0),
        HexCoord::new(2, 1),
        HexCoord::new(2, 2),
    ]
}

fn entry(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gltfs: Res<Assets<Gltf>>,
    nine_assets: Res<NineAssets>,
    player_field: Res<Field<Player>>,
    cpu_field: Res<Field<Cpu>>,
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
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    let gltf = gltfs.get(&nine_assets.nine).unwrap();
    let h_scene = gltf.default_scene.as_ref().unwrap().clone();
    let e_cell = player_field.cells.get(&HexCoord::new(0, 0)).unwrap();
    commands.spawn((WorldAssetRoot(h_scene), Unit, OccupyingCell(*e_cell)));
}
