use bevy::{
    asset::RenderAssetUsages, math::Vec3, prelude::{Mesh, MeshBuilder}, render::mesh::Indices
};
use wgpu::PrimitiveTopology;

struct BoxFaceBuilder {
    face: FaceType,
    half_size: Vec3,
}

enum FaceType {
    Front,
    Back,
    Right,
    Left,
    Top,
    //Bottom,
}

impl BoxFaceBuilder {
    fn new(length: f32, face: FaceType) -> Self {
        Self {
            face,
            half_size: Vec3::new(length, length, length) / 2.0,
        }
    }
}

impl MeshBuilder for BoxFaceBuilder {
    fn build(&self) -> Mesh {
        let min = -self.half_size;
        let max = self.half_size;

        // Suppose Y-up right hand, and camera look from +Z to -Z
        let vertices = match self.face {
            FaceType::Front => &[
                ([min.x, min.y, max.z], [0.0, 0.0, 1.0], [0.0, 1.0]),
                ([max.x, min.y, max.z], [0.0, 0.0, 1.0], [1.0, 1.0]),
                ([max.x, max.y, max.z], [0.0, 0.0, 1.0], [1.0, 0.0]),
                ([min.x, max.y, max.z], [0.0, 0.0, 1.0], [0.0, 0.0]),
            ],
            FaceType::Back => &[
                ([min.x, max.y, min.z], [0.0, 0.0, -1.0], [1.0, 0.0]),
                ([max.x, max.y, min.z], [0.0, 0.0, -1.0], [0.0, 0.0]),
                ([max.x, min.y, min.z], [0.0, 0.0, -1.0], [0.0, 1.0]),
                ([min.x, min.y, min.z], [0.0, 0.0, -1.0], [1.0, 1.0]),
            ],
            FaceType::Right => &[
                ([max.x, min.y, min.z], [1.0, 0.0, 0.0], [0.0, 1.0]),
                ([max.x, max.y, min.z], [1.0, 0.0, 0.0], [0.0, 0.0]),
                ([max.x, max.y, max.z], [1.0, 0.0, 0.0], [1.0, 0.0]),
                ([max.x, min.y, max.z], [1.0, 0.0, 0.0], [1.0, 1.0]),
            ],
            FaceType::Left => &[
                ([min.x, min.y, max.z], [-1.0, 0.0, 0.0], [1.0, 1.0]),
                ([min.x, max.y, max.z], [-1.0, 0.0, 0.0], [1.0, 0.0]),
                ([min.x, max.y, min.z], [-1.0, 0.0, 0.0], [0.0, 0.0]),
                ([min.x, min.y, min.z], [-1.0, 0.0, 0.0], [0.0, 1.0]),
            ],
            FaceType::Top => &[
                ([max.x, max.y, min.z], [0.0, 1.0, 0.0], [1.0, 0.0]),
                ([min.x, max.y, min.z], [0.0, 1.0, 0.0], [0.0, 0.0]),
                ([min.x, max.y, max.z], [0.0, 1.0, 0.0], [0.0, 1.0]),
                ([max.x, max.y, max.z], [0.0, 1.0, 0.0], [1.0, 1.0]),
            ],
            //FaceType::Bottom => todo!(),
        };
        let indices = vec![0, 1, 2, 2, 3, 0];

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
    }
}
