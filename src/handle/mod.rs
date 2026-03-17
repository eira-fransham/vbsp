mod displacement;
mod face;
mod game;

use crate::Bsp;
use crate::data::*;
use crate::lighting::AmbientVoxelGrid;
use crate::lighting::AmbientVoxelGridBuilder;
use crate::lighting::LeafAmbientIndex;
use crate::lighting::LeafWithAmbientIndex;
use ahash::RandomState;
use glam::Vec2;
use glam::Vec3;
use itertools::Either;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// A handle represents a data structure in the bsp file and the bsp file containing it.
///
/// Keeping a reference of the bsp file with the data is required since a lot of data types
/// reference parts from other structures in the bsp file
#[derive(Copy, Clone)]
pub struct HandleGeneric<'a, T> {
    pub bsp: &'a Bsp,
    pub data: T,
}

pub type Handle<'a, T> = HandleGeneric<'a, &'a T>;

impl<T: Debug> Debug for HandleGeneric<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

impl<'a, T> AsRef<T> for Handle<'a, T> {
    fn as_ref(&self) -> &'a T {
        self.data
    }
}

impl<T> Deref for HandleGeneric<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> Handle<'a, T> {
    pub fn new(bsp: &'a Bsp, data: &'a T) -> Self {
        Handle { bsp, data }
    }
}

impl<'a> Handle<'a, Model> {
    /// Get all faces that make up the model
    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> {
        let start = self.first_face as usize;
        let end = start + self.face_count as usize;
        let bsp = self.bsp;

        bsp.faces[start..end]
            .iter()
            .map(move |face| Handle::new(bsp, face))
    }

    pub fn root(&self) -> Option<Handle<'a, Node>> {
        let root_idx: usize = self.head_node.try_into().ok()?;
        self.bsp.node(root_idx)
    }

    pub fn leaves(&self) -> impl Iterator<Item = Handle<'a, Leaf>> {
        self.root().into_iter().flat_map(|node| node.leaves())
    }

    /// Get all faces that make up the model
    pub fn faces_with_id(&self) -> impl Iterator<Item = (i32, Handle<'a, Face>)> {
        let start = self.first_face;
        let end = start + self.face_count;

        (start..end).zip(self.faces())
    }

    pub fn textures(&self) -> impl Iterator<Item = Handle<'_, TextureInfo>> {
        self.bsp.textures()
    }
}

impl<'a> From<HandleGeneric<'a, LeafWithAmbientIndex<'a>>> for Handle<'a, Leaf> {
    fn from(value: HandleGeneric<'a, LeafWithAmbientIndex<'a>>) -> Self {
        Handle {
            bsp: value.bsp,
            data: value.data.leaf,
        }
    }
}

impl<'a> HandleGeneric<'a, LeafWithAmbientIndex<'a>> {
    pub fn ambient_voxel_grid(&self) -> AmbientVoxelGrid {
        let LeafAmbientIndex { count, start } = *self.data.ambient_index;
        let start = start as usize;
        let count = count as usize;

        let mut builder = AmbientVoxelGridBuilder::new();
        for cube in &self.bsp.ambient_lighting_data[start..start + count] {
            builder.set(cube.position, cube.data);
        }

        builder.finish()
    }
}

impl<'a> Handle<'a, Node> {
    /// Get the plane splitting this node
    pub fn plane(&self) -> Handle<'a, Plane> {
        self.bsp.plane(self.plane_index as _).unwrap()
    }

    pub fn children(&self) -> Option<[Either<Handle<'a, Node>, Handle<'a, Leaf>>; 2]> {
        let [left, right] = self.children.map(|i| {
            if i < 0 {
                self.bsp
                    .leaf((!i).try_into().ok()?)
                    .map(Into::into)
                    .map(Either::Right)
            } else {
                self.bsp.node(i.try_into().ok()?).map(Either::Left)
            }
        });

        Some([left?, right?])
    }

    pub fn leaves(&self) -> impl Iterator<Item = Handle<'a, Leaf>> + use<'a> {
        Box::new(
            self.children()
                .into_iter()
                .flatten()
                .flat_map(|child| match child {
                    Either::Left(node) => Either::Left(node.leaves()),
                    Either::Right(leaf) => Either::Right(std::iter::once(leaf)),
                }),
        ) as Box<dyn Iterator<Item = Handle<'a, Leaf>>>
    }

    pub fn vis_clusters(&self) -> HashMap<i32, Vec<Handle<'a, Leaf>>> {
        let mut out = HashMap::<i32, Vec<Handle<'a, Leaf>>>::new();

        for leaf in self.leaves() {
            out.entry(leaf.cluster).or_default().push(leaf);
        }

        out
    }
}

impl<'a, L> HandleGeneric<'a, L>
where
    L: Deref<Target = Leaf>,
{
    /// Get all other leaves visible from this one
    pub fn visible_set(&self) -> Option<impl Iterator<Item = Handle<'a, Leaf>>> {
        let cluster = self.cluster;
        let bsp = self.bsp;

        if cluster < 0 {
            None
        } else {
            let visible_clusters = bsp.vis_data.visible_clusters(cluster);
            Some(
                bsp.leaves
                    .iter()
                    .filter(move |leaf| {
                        if leaf.cluster == cluster {
                            true
                        } else if leaf.cluster > 0 {
                            visible_clusters[leaf.cluster as usize]
                        } else {
                            false
                        }
                    })
                    .map(move |leaf| Handle { bsp, data: leaf }),
            )
        }
    }

    /// Get all faces in this leaf
    pub fn faces(&self) -> impl Iterator<Item = Handle<'a, Face>> + use<'a, L> {
        self.faces_with_id().map(|(_, face)| face)
    }

    /// Get all faces that make up the model
    pub fn faces_with_id(&self) -> impl Iterator<Item = (u32, Handle<'a, Face>)> + use<'a, L> {
        let start = self.first_leaf_face;
        let end = start + self.leaf_face_count;
        let bsp = self.bsp;
        bsp.leaf_faces[start as usize..end as usize]
            .iter()
            .filter_map(move |leaf_face| {
                Some((leaf_face.face as u32, bsp.face(leaf_face.face as usize)?))
            })
    }
}

impl<'a> Handle<'a, TextureInfo> {
    pub fn texture_data(&self) -> Handle<'a, TextureData> {
        Handle::new(
            self.bsp,
            &self.bsp.textures_data[self.data.texture_data_index as usize],
        )
    }
    pub fn name(&self) -> &'a str {
        self.texture_data().name()
    }

    /// Get a color that is unique but deterministic for this texture
    pub fn debug_color(&self) -> [u8; 3] {
        self.texture_data().debug_color()
    }

    pub fn lightmap_uv(&self, pos: Vec3) -> Vec2 {
        self.lightmap_transforms.project(pos) + Vec2::splat(0.5)
    }

    pub fn uv(&self, pos: Vec3) -> Vec2 {
        self.texture_transforms.project(pos) / self.texture_data().width as f32
    }
}

impl<'a> Handle<'a, TextureData> {
    pub fn name(&self) -> &'a str {
        let start = self.bsp.texture_string_tables[self.name_string_table_id as usize] as usize;
        let part = &self.bsp.texture_string_data[start..];
        if let Some((s, _)) = part.split_once('\0') {
            s
        } else {
            part
        }
    }

    /// Get a color that is unique but deterministic for this texture
    pub fn debug_color(&self) -> [u8; 3] {
        let name_hash = RandomState::with_seeds(0, 0, 0, 0)
            .hash_one(self.name())
            .to_le_bytes();
        [name_hash[0], name_hash[1], name_hash[2]]
    }
}
