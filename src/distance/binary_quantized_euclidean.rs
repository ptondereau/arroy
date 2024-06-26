use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use rand::Rng;

use super::two_means;
use crate::distance::Distance;
use crate::node::{Leaf, UnalignedF32Slice};
use crate::parallel::ImmutableSubsetLeafs;
use crate::spaces::simple::{dot_product, euclidean_distance};

/// The Euclidean distance between two points in Euclidean space
/// is the length of the line segment between them.
///
/// `d(p, q) = sqrt((p - q)²)`
#[derive(Debug, Clone)]
pub enum BinaryQuantizedEuclidean {}

/// The header of BinaryQuantizedEuclidean leaf nodes.
#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct NodeHeaderBinaryQuantizedEuclidean {
    /// An extra constant term to determine the offset of the plane
    bias: f32,
}

impl Distance for BinaryQuantizedEuclidean {
    type Header = NodeHeaderBinaryQuantizedEuclidean;

    fn name() -> &'static str {
        "binary quantized euclidean"
    }

    fn new_header(_vector: &UnalignedF32Slice) -> Self::Header {
        NodeHeaderBinaryQuantizedEuclidean { bias: 0.0 }
    }

    fn built_distance(p: &Leaf<Self>, q: &Leaf<Self>) -> f32 {
        binary_quantized_euclidean_distance(&p.vector, &q.vector)
    }

    fn init(_node: &mut Leaf<Self>) {}

    fn create_split<R: Rng>(
        children: &ImmutableSubsetLeafs<Self>,
        rng: &mut R,
    ) -> heed::Result<Vec<f32>> {
        let [node_p, node_q] = two_means(rng, children, false)?;
        let vector = node_p.vector.iter().zip(node_q.vector.iter()).map(|(p, q)| p - q).collect();
        let mut normal = Leaf {
            header: NodeHeaderBinaryQuantizedEuclidean { bias: 0.0 },
            vector: Cow::Owned(vector),
        };
        Self::normalize(&mut normal);

        normal.header.bias = normal
            .vector
            .iter()
            .zip(node_p.vector.iter())
            .zip(node_q.vector.iter())
            .map(|((n, p), q)| -n * (p + q) / 2.0)
            .sum();

        Ok(normal.vector.into_owned())
    }

    fn margin(p: &Leaf<Self>, q: &Leaf<Self>) -> f32 {
        p.header.bias + dot_product(&p.vector, &q.vector)
    }

    fn margin_no_header(p: &UnalignedF32Slice, q: &UnalignedF32Slice) -> f32 {
        dot_product(p, q)
    }
}

fn binary_quantized_euclidean_distance(u: &UnalignedF32Slice, v: &UnalignedF32Slice) -> f32 {
    u.as_bytes().iter().zip(v.as_bytes()).map(|(u, v)| (u ^ v).count_ones()).sum::<u32>() as f32
}
