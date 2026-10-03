use bevy::{color::palettes, prelude::*};

use crate::{
    asset::GameState,
    damage::{DamageAssets, DamageBuffer, DamageVariant, Health, HealthCap, Resist},
    field::{Field, HexCoord, OccupyingCell},
    turn::{Cpu, Player, Team},
};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), spawn_unit_details_window)
            .add_systems(Update, (reposition::<Player>, reposition::<Cpu>))
            .add_observer(show_unit_details);
    }
}

#[derive(Component, Default)]
#[require(
    Health,
    Resist,
    DamageBuffer,
    Transform,
    OccupyingCell(Entity::PLACEHOLDER)
)]
pub struct Unit;

pub struct UnitDesc {
    pub name: &'static str,
    pub passive: &'static str,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct SelectedUnit;

#[derive(Resource)]
pub struct ActiveUnits(pub Vec<Entity>);

pub fn reposition<T: Team + Component>(
    field: Res<Field<T>>,
    q_moved_units: Query<(&mut Transform, &OccupyingCell), (Changed<OccupyingCell>, With<T>)>,
    q_hexcoord: Query<&HexCoord>,
) {
    for (mut tf, OccupyingCell(e_cell)) in q_moved_units {
        let cell_hexcoord = q_hexcoord.get(*e_cell).unwrap();
        *tf = field.to_world_space(*cell_hexcoord);
        info!("unit moved to {:?}", cell_hexcoord);
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct UnitDetailsWindow;

#[derive(Component, Debug, Clone, FromTemplate)]
#[relationship(relationship_target = HealthBarOwner)]
pub struct HealthBar(pub Entity);

#[derive(Component, Debug, Clone, FromTemplate)]
#[relationship_target(relationship = HealthBar)]
pub struct HealthBarOwner(Entity);

#[derive(Component, Debug, Clone, FromTemplate)]
#[relationship(relationship_target = UnitResistDetailsOwner)]
pub struct UnitResistDetailsValue(pub Entity);

#[derive(Component, Debug, Clone, Default)]
#[relationship_target(relationship = UnitResistDetailsValue)]
pub struct UnitResistDetailsOwner(Vec<Entity>);

pub fn spawn_unit_details_window(mut commands: Commands, damage_assets: Res<DamageAssets>) {
    #[rustfmt::skip]
    let resist_value_layout = vec![
        (DamageVariant::Physical, 1, 1),
        (DamageVariant::Magical,  1, 2),
        (DamageVariant::Heat,     2, 1),
        (DamageVariant::Cold,     2, 2),
        (DamageVariant::Plague,   2, 3),
        (DamageVariant::Radiant,  3, 1),
        (DamageVariant::Abyssal,  3, 2),
    ];

    bsn! {
        #MainWindow
        UnitDetailsWindow
        UnitResistDetailsOwner::default()
        HealthBarOwner(#HealthBar)
        Node {
            top: Val::ZERO,
            right: Val::ZERO,
            margin: UiRect::all(Val::Px(8.0)),
            padding: UiRect::all(Val::Px(8.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        }

        Children [
            (
                #Photo
                Node {
                    height: Val::Px(40.0),
                    aspect_ratio: { Some(1.0) },
                }
                ImageNode::default()
            ),

            (
                #ResistValues
                Node {
                    display: Display::Grid,
                    grid_template_columns: { RepeatedGridTrack::flex::<Vec<_>>(3, 1.0) },
                    grid_template_rows: { RepeatedGridTrack::flex::<Vec<_>>(3, 1.0) },
                }
                Children [
                    // Instead of a manual loop mutation, evaluate a clean procedural bsn list mapping
                    {
                        resist_value_layout.iter().map(|(damage_ty, col, row)| {
                            bsn! {
                                Node {
                                    grid_column: GridPlacement::start(*col),
                                    grid_row: GridPlacement::span(*row),
                                }
                                Children [
                                    (
                                        Node {
                                            height: Val::Px(24.0),
                                            aspect_ratio: { Some(1.0) },
                                        }
                                        ImageNode {
                                            image: { damage_assets.icon_from_damage_variant(*damage_ty).unwrap().clone() },
                                        }
                                    ),
                                    (
                                        UnitResistDetailsValue(#MainWindow)
                                        Text::new("1.00 x")
                                    )
                                ]
                            }
                        }).collect::<Vec<_>>()
                    }
                ]
            ),

            (
                #HealthBar
                HealthBar(#MainWindow)
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    left: Val::ZERO,
                    height: Val::Px(10.0),
                }
                BackgroundColor(Color::from(palettes::basic::RED))
            )
        ]
    };
}

pub fn show_unit_details(
    trigger: On<Add, SelectedUnit>,
    unit_query: Query<(&Health, &Resist, &HealthCap)>,
    unit_details_query: Query<(&HealthBarOwner, &UnitResistDetailsOwner), With<UnitDetailsWindow>>,
    mut health_bar_query: Query<&mut Node, With<HealthBar>>,
    mut resist_query: Query<&mut Text, With<UnitResistDetailsValue>>,
) {
    let (health, resist, health_cap) = unit_query.get(trigger.entity).unwrap();
    let (HealthBarOwner(e_healthbar), UnitResistDetailsOwner(e_resists)) =
        unit_details_query.single().unwrap();

    let mut health_bar_node = health_bar_query.get_mut(*e_healthbar).unwrap();
    health_bar_node.width = Val::Percent(health.0 / health_cap.0 * 100.0);

    for (damage_ty, value) in resist.0.iter() {
        let e_resist = e_resists[*damage_ty as usize];
        let mut resist_text = resist_query.get_mut(e_resist).unwrap();
        resist_text.0 = format!("{0:.2} x", value);
    }
}
