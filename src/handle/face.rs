use super::Handle;
use crate::data::*;
use glam::{Vec2, Vec3};
use itertools::Either;

impl<'a> Handle<'a, FaceV2> {
    /// Get the texture of the face
    pub fn texture(&self) -> Handle<'a, TextureInfo> {
        self.bsp
            .textures_info
            .get(self.texture_info as usize)
            .map(|texture_info| Handle {
                bsp: self.bsp,
                data: texture_info,
            })
            .unwrap()
    }

    /// Get all vertices making up the face
    pub fn vertices(&self) -> impl Iterator<Item = &'a Vertex> + 'a {
        let bsp = self.bsp;
        self.vertex_indexes()
            .map(move |vert_index| bsp.vertices.get(vert_index as usize).unwrap())
    }

    /// Get the vertex indexes of all vertices making up the face
    ///
    /// The indexes index into the `vertices` field of the bsp file
    pub fn vertex_indexes(&self) -> impl Iterator<Item = u16> + 'a {
        let bsp = self.bsp;
        (self.data.first_edge..(self.data.first_edge + self.data.num_edges as i32))
            .map(move |surface_edge| bsp.surface_edges.get(surface_edge as usize).unwrap())
            .map(move |surface_edge| {
                bsp.edges
                    .get(surface_edge.edge_index() as usize)
                    .map(|edge| (edge, surface_edge.direction()))
                    .unwrap()
            })
            .map(|(edge, direction)| match direction {
                EdgeDirection::FirstToLast => edge.start_index,
                EdgeDirection::LastToFirst => edge.end_index,
            })
    }

    pub fn edge_direction(&self) -> EdgeDirection {
        self.bsp.surface_edges[self.first_edge as usize].direction()
    }

    /// Check if the face is flagged as visible
    pub fn is_visible(&self) -> bool {
        let texture = self.texture();
        !texture.flags.intersects(
            TextureFlags::SKY2D
                | TextureFlags::SKY
                | TextureFlags::TRIGGER
                | TextureFlags::HINT
                | TextureFlags::SKIP
                | TextureFlags::NODRAW,
        )
    }

    pub fn uvs(&self) -> impl Iterator<Item = Vec2> {
        self.vertex_positions().map(|pos| self.texture().uv(pos))
    }

    pub fn lightmap_uvs(&self) -> impl Iterator<Item = Vec2> {
        self.vertex_positions()
            .map(|pos| self.texture().lightmap_uv(pos))
            .map(|uv| uv - self.light_map_texture_min.as_vec2())
    }

    /// Triangulate the face
    ///
    /// Triangulation only works for faces that can be turned into a triangle fan trivially
    pub fn triangulate(&self) -> impl Iterator<Item = [Vec3; 3]> + 'a {
        let mut vertices = self.vertices();

        let a = vertices.next().expect("face with <3 points");
        let mut b = vertices.next().expect("face with <3 points");

        vertices.map(move |c| {
            let points = [c.position, b.position, a.position];
            b = c;
            points
        })
    }

    pub fn displacement(&self) -> Option<Handle<'a, DisplacementInfo>> {
        self.bsp.displacement(self.displacement_info as usize)
    }

    /// Get the vertex positions for the face
    ///
    /// This either calculates the displacement or normal triangulation depending on the face
    pub fn vertex_positions(&self) -> impl Iterator<Item = Vec3> + 'a {
        self.displacement()
            .map(|displacement| displacement.triangulated_displaced_vertices())
            .map(Either::Left)
            .unwrap_or_else(|| Either::Right(self.triangulate().flatten()))
    }

    pub fn normal(&self) -> Vec3 {
        self.bsp.plane(self.plane_num as usize).unwrap().normal
    }
}
