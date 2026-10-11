use bevy::asset::RenderAssetUsages;
use bevy::platform::collections::HashMap;
use bevy::{
    math::Affine2,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use crate::field::{Highlighter, HighlighterMaterials};

pub struct DirectorPlugin;

impl Plugin for DirectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DirectorMaterials>()
            .add_systems(Update, animate_director_materials)
            .add_observer(spawn_director_arrow);
    }
}

#[derive(Component)]
#[require(Highlighter)]
pub struct Director {
    pub curve: CubicCurve<Vec3>,
    pub speed: f32,
    pub width: f32,
    pub tiling_distance: f32,
}

#[derive(Resource)]
pub struct DirectorMaterials(pub HashMap<Highlighter, Handle<StandardMaterial>>);

impl FromWorld for DirectorMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        DirectorMaterials(HashMap::from(Highlighter::VALUES.map(|v| {
            (
                v,
                materials.add(StandardMaterial {
                    base_color_texture: None, // TODO stripes
                    base_color: LinearRgba::from(v).into(),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..Default::default()
                }),
            )
        })))
    }
}

fn spawn_director_arrow(
    trigger: On<Add, Director>,
    mut commands: Commands,
    materials: Res<HighlighterMaterials>,
    mut meshes: ResMut<Assets<Mesh>>,
    director_query: Query<(&Director, &Highlighter)>,
) {
    let (director, highlighter) = director_query.get(trigger.entity).unwrap();

    let h_mesh = meshes.add(generate_arcing_ribbon(
        &director.curve,
        director.width,
        director.tiling_distance,
    ));
    let h_mat = materials.0.get(highlighter).unwrap();

    commands
        .entity(trigger.entity)
        .insert((Mesh3d(h_mesh), MeshMaterial3d(h_mat.clone())));
}

fn animate_director_materials(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    director_materials: Res<DirectorMaterials>,
) {
    for h_material in director_materials.0.values() {
        let mut material = materials.get_mut(h_material).unwrap();
        let offset = time.elapsed_secs() % 1.0;
        material.uv_transform = Affine2::from_translation(Vec2::new(-offset, 0.0));
    }
}

fn integrate_path_length(curve: &CubicCurve<Vec3>, div: usize) -> f32 {
    let mut len = 0.0;
    let mut positions = curve.iter_positions(div);

    if let Some(mut p0) = positions.next() {
        for p1 in positions {
            len += p0.distance(p1);
            p0 = p1;
        }
    }

    len
}

/// Generates a ribbon that tracks a 3D arc while staying horizontally flat to the sky
// thanks to gemini my goat for the idea, although the code it made kinda sucked
fn generate_arcing_ribbon(curve: &CubicCurve<Vec3>, width: f32, tiling_distance: f32) -> Mesh {
    let mut vertices = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_width = width / 2.0;
    let path_len = integrate_path_length(curve, 100);
    let segments = (path_len / 0.1) as usize;

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let s = curve.position(t);
        let tangent = curve.velocity(t);

        // generate vertices to left and right of curve for every segment
        let up = Vec3::Y;
        let tangent_proj = tangent.reject_from(Vec3::Y).normalize();
        let rvec = tangent_proj.cross(up);

        vertices.push(s - rvec * half_width);
        vertices.push(s + rvec * half_width);

        // define the texture uv to repeat once over tiling distance
        let u = (path_len / tiling_distance) * t;
        uvs.push(Vec2::new(u, 0.0));
        uvs.push(Vec2::new(u, 1.0));

        // add two triangles corresponding to segment quad
        // (we're using strip primitive fancy oo)
        indices.push(i as u32 * 2);
        indices.push(i as u32 * 2 + 1);
    }

    Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}
