use std::borrow::Cow;
use std::fmt;

pub use angular::{Angular, NodeHeaderAngular};
pub use binary_quantized_euclidean::{
    BinaryQuantizedEuclidean, NodeHeaderBinaryQuantizedEuclidean,
};
use bytemuck::{Pod, Zeroable};
pub use dot_product::{DotProduct, NodeHeaderDotProduct};
pub use euclidean::{Euclidean, NodeHeaderEuclidean};
use heed::{RwPrefix, RwTxn};
pub use manhattan::{Manhattan, NodeHeaderManhattan};
use rand::Rng;

use crate::internals::{KeyCodec, Side};
use crate::node::{Leaf, SizeMismatch, UnalignedVector};
use crate::parallel::ImmutableSubsetLeafs;
use crate::spaces::simple::dot_product;
use crate::NodeCodec;

mod angular;
mod binary_quantized_euclidean;
mod dot_product;
mod euclidean;
mod manhattan;

/// A trait used by arroy to compute the distances,
/// compute the split planes, and normalize user vectors.
#[allow(missing_docs)]
pub trait Distance: Send + Sync + Sized + Clone + fmt::Debug + 'static {
    /// A header structure with informations related to the
    type Header: Pod + Zeroable + fmt::Debug;

    fn name() -> &'static str;

    fn craft_owned_unaligned_vector_from_f32(vector: Vec<f32>) -> Cow<'static, UnalignedVector> {
        UnalignedVector::owned_f32_vectors_from_f32_slice(vector)
    }

    fn craft_unaligned_vector_from_f32(vector: &[f32]) -> Cow<UnalignedVector> {
        Cow::Borrowed(UnalignedVector::f32_vectors_from_f32_slice(vector))
    }

    fn craft_unaligned_vector_from_bytes(
        vector: &[u8],
    ) -> Result<Cow<UnalignedVector>, SizeMismatch> {
        UnalignedVector::f32_vectors_from_bytes(vector).map(Cow::Borrowed)
    }

    fn read_unaligned_vector(vector: &UnalignedVector) -> Vec<f32> {
        vector.iter_f32().collect()
    }

    fn new_header(vector: &UnalignedVector) -> Self::Header;

    /// Returns a non-normalized distance.
    fn built_distance(p: &Leaf<Self>, q: &Leaf<Self>) -> f32;

    fn non_built_distance(p: &Leaf<Self>, q: &Leaf<Self>) -> f32 {
        Self::built_distance(p, q)
    }

    /// Normalizes the distance returned by the distance method.
    fn normalized_distance(d: f32) -> f32 {
        d.sqrt()
    }

    fn pq_distance(distance: f32, margin: f32, side: Side) -> f32 {
        match side {
            Side::Left => (-margin).min(distance),
            Side::Right => margin.min(distance),
        }
    }

    fn norm(leaf: &Leaf<Self>) -> f32 {
        Self::norm_no_header(&leaf.vector)
    }

    fn norm_no_header(v: &UnalignedVector) -> f32 {
        dot_product(v, v).sqrt()
    }

    fn normalize(node: &mut Leaf<Self>) {
        let norm = Self::norm(node);
        if norm > 0.0 {
            let vec: Vec<_> = node.vector.iter_f32().map(|x| x / norm).collect();
            node.vector = Self::craft_owned_unaligned_vector_from_f32(vec);
        }
    }

    fn init(node: &mut Leaf<Self>);

    fn update_mean(mean: &mut Leaf<Self>, new_node: &Leaf<Self>, norm: f32, c: f32) {
        let vec: Vec<_> = mean
            .vector
            .iter_f32()
            .zip(new_node.vector.iter_f32())
            .map(|(x, n)| (x * c + n / norm) / (c + 1.0))
            .collect();
        mean.vector = Self::craft_owned_unaligned_vector_from_f32(vec);
    }

    fn create_split<'a, R: Rng>(
        children: &'a ImmutableSubsetLeafs<Self>,
        rng: &mut R,
    ) -> heed::Result<Cow<'a, UnalignedVector>>;

    fn margin(p: &Leaf<Self>, q: &Leaf<Self>) -> f32 {
        Self::margin_no_header(&p.vector, &q.vector)
    }

    fn margin_no_header(p: &UnalignedVector, q: &UnalignedVector) -> f32;

    fn side<R: Rng>(normal_plane: &UnalignedVector, node: &Leaf<Self>, rng: &mut R) -> Side {
        let dot = Self::margin_no_header(&node.vector, normal_plane);
        if dot > 0.0 {
            Side::Right
        } else if dot < 0.0 {
            Side::Left
        } else {
            Side::random(rng)
        }
    }

    fn preprocess(
        _wtxn: &mut RwTxn,
        _new_iter: impl for<'a> Fn(
            &'a mut RwTxn,
        ) -> heed::Result<RwPrefix<'a, KeyCodec, NodeCodec<Self>>>,
    ) -> heed::Result<()> {
        Ok(())
    }
}

fn two_means<D: Distance, R: Rng>(
    rng: &mut R,
    leafs: &ImmutableSubsetLeafs<D>,
    cosine: bool,
) -> heed::Result<[Leaf<'static, D>; 2]> {
    // This algorithm is a huge heuristic. Empirically it works really well, but I
    // can't motivate it well. The basic idea is to keep two centroids and assign
    // points to either one of them. We weight each centroid by the number of points
    // assigned to it, so to balance it.

    const ITERATION_STEPS: usize = 200;

    let [leaf_p, leaf_q] = leafs.choose_two(rng)?.unwrap();
    let mut leaf_p = leaf_p.into_owned();
    let mut leaf_q = leaf_q.into_owned();

    if cosine {
        D::normalize(&mut leaf_p);
        D::normalize(&mut leaf_q);
    }

    D::init(&mut leaf_p);
    D::init(&mut leaf_q);

    let mut ic = 1.0;
    let mut jc = 1.0;
    for _ in 0..ITERATION_STEPS {
        let node_k = leafs.choose(rng)?.unwrap();
        let di = ic * D::non_built_distance(&leaf_p, &node_k);
        let dj = jc * D::non_built_distance(&leaf_q, &node_k);
        let norm = if cosine { D::norm(&node_k) } else { 1.0 };
        if norm.is_nan() || norm <= 0.0 {
            continue;
        }
        if di < dj {
            Distance::update_mean(&mut leaf_p, &node_k, norm, ic);
            Distance::init(&mut leaf_p);
            ic += 1.0;
        } else if dj < di {
            Distance::update_mean(&mut leaf_q, &node_k, norm, jc);
            Distance::init(&mut leaf_q);
            jc += 1.0;
        }
    }

    Ok([leaf_p, leaf_q])
}
