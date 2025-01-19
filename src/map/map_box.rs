use bevy::{
    asset::RenderAssetUsages, math::Vec3, prelude::{Mesh, MeshBuilder}, render::mesh::Indices
};
use wgpu::PrimitiveTopology;

//pub fn spawn_box() {
//    let front = commands.spawn((
//        Front,
//        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Front))),
//        MeshMaterial3d(materials.add(StandardMaterial {
//            base_color_texture: Some(image.clone()),
//            ..default()
//        })),
//    ));
//
//    let back = commands.spawn((
//        Back,
//        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Back))),
//        MeshMaterial3d(materials.add(StandardMaterial {
//            base_color_texture: Some(image.clone()),
//            ..default()
//        })),
//    ));
//
//    let left = commands.spawn((
//        Left,
//        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Left))),
//        MeshMaterial3d(materials.add(StandardMaterial {
//            base_color_texture: Some(image.clone()),
//            ..default()
//        })),
//    ));
//
//    let right = commands.spawn((
//        Right,
//        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Right))),
//        MeshMaterial3d(materials.add(StandardMaterial {
//            base_color_texture: Some(image.clone()),
//            ..default()
//        })),
//    ));
//
//    let top = commands.spawn((
//        Top,
//        Mesh3d(meshes.add(BoxFaceBuilder::new(1.0, FaceType::Top))),
//        MeshMaterial3d(materials.add(StandardMaterial {
//            base_color_texture: Some(image.clone()),
//            ..default()
//        })),
//    ));
//}

pub struct BoxFaceBuilder {
    face: FaceType,
    half_size: Vec3,
}

pub enum FaceType {
    Front,
    Back,
    Right,
    Left,
    Top,
    //Bottom,
}

impl BoxFaceBuilder {
    pub fn new(length: f32, face: FaceType) -> Self {
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
