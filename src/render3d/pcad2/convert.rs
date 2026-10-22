use crate::{
    color::{self, Color},
    math::{Quat, Vec2, Vec3},
    render3d::{Mesh, Model, Triangle, Vertex, Texture},
};

use super::raw::{
    RawFace,
    RawMesh,
    RawNode,
    RawPcad2,
};

pub fn to_model(raw: &RawPcad2) -> Model {
    let palette = convert_palette(&raw.texture.colors);

    let texture = convert_texture(
        raw,
        &palette,
    );

    let mut meshes = Vec::new();

    convert_node(
        &raw.graph,
        &palette,
        &mut meshes,
    );

    Model {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        meshes,
        texture: Some(texture),
    }
}

pub fn to_meshes(raw: &RawPcad2) -> Vec<Mesh> {
    let palette = convert_palette(&raw.texture.colors);

    let mut meshes = Vec::new();

    convert_node(
        &raw.graph,
        &palette,
        &mut meshes,
    );

    meshes
}

fn convert_node(
    node: &RawNode,
    palette: &[Color],
    meshes: &mut Vec<Mesh>,
) {
    if let Some(raw_mesh) = &node.mesh {
        meshes.push(convert_mesh(
            raw_mesh,
            node,
            palette,
        ));
    }

    for child in &node.children {
        convert_node(
            child,
            palette,
            meshes,
        );
    }
}

fn convert_palette(colors: &[[f32; 3]]) -> Vec<Color> {
    colors
        .iter()
        .map(|rgb| {
            let r = (rgb[0] * 7.0)
                .round()
                .clamp(0.0, 7.0) as u8;

            let g = (rgb[1] * 7.0)
                .round()
                .clamp(0.0, 7.0) as u8;

            let b = (rgb[2] * 3.0)
                .round()
                .clamp(0.0, 3.0) as u8;

            color::rgb(r, g, b)
        })
        .collect()
}

fn convert_mesh(
    raw_mesh: &RawMesh,
    node: &RawNode,
    palette: &[Color],
) -> Mesh {
    let vertices: Vec<Vec3> = raw_mesh
        .vertices
        .chunks_exact(3)
        .map(|v| {
            Vec3::new(
                v[0] * node.transform.scale.x,
                v[1] * node.transform.scale.y,
                v[2] * node.transform.scale.z,
            )
        })
        .collect();

    let mut triangles = Vec::new();

    for face in &raw_mesh.faces {
        convert_face(
            face,
            &vertices,
            palette,
            &mut triangles,
        );
    }

    Mesh::new(
        Vec3::new(
            node.transform.pos.x,
            node.transform.pos.y,
            node.transform.pos.z,
        ),

        // Keep this identity until we verify exactly how
        // picoCAD2 stores rotation units/order.
        Quat::IDENTITY,

        triangles
    )
}

fn convert_face(
    face: &RawFace,
    positions: &[Vec3],
    palette: &[Color],
    triangles: &mut Vec<Triangle>,
) {
    if face.vertex_ids.len() < 3 {
        return;
    }

    let color = palette
        .get(face.color)
        .copied()
        .unwrap_or(color::WHITE);

    let mut vertices = Vec::new();

    for (i, vertex_id) in face.vertex_ids.iter().enumerate() {
        // picoCAD2 IDs are 1-based.
        if *vertex_id == 0 {
            continue;
        }

        let index = vertex_id - 1;

        let Some(position) =
            positions.get(index).copied()
        else {
            continue;
        };

        let uv_index = i * 2;

        let uv = if uv_index + 1 < face.uvs.len() {
            Vec2::new(
                face.uvs[uv_index],
                face.uvs[uv_index + 1],
            )
        } else {
            Vec2::ZERO
        };

        vertices.push(Vertex {
            position,
            uv,
        });
    }

    if vertices.len() < 3 {
        return;
    }

    // Triangle fan:
    //
    // 0---1
    // | / |
    // |/  |
    // 3---2
    //
    // becomes:
    // 0,1,2
    // 0,2,3

    for i in 1..vertices.len() - 1 {
        triangles.push(Triangle {
            vertices: [
                vertices[0],
                vertices[i],
                vertices[i + 1],
            ],

            color: color,
        });
    }
}

fn convert_texture(
    raw: &RawPcad2,
    palette: &[Color],
) -> Texture {
    let pixels = raw
        .texture
        .pixels
        .chars()
        .filter_map(|c| c.to_digit(16))
        .map(|index| {
            palette
                .get(index as usize)
                .copied()
                .unwrap_or(color::BLACK)
        })
        .collect();

    Texture::new(
        128,
        128,
        pixels,
    )
}