/*
 * ArokoDB Core Library
 *
 * This crate currently provides an exact, in-memory vector index. It serves as
 * a correctness baseline for future approximate indexes and storage backends.
 */

pub use node::HnswNode;

pub mod node {
    use std::fmt;

    /// Vector data and metadata used by the index.
    #[derive(Clone, Debug)]
    pub struct HnswNode {
        pub embedding: Vec<f32>,
        pub level: usize,
        pub id: usize,
        pub links: Vec<usize>,
        pub score: f32,
    }

    impl HnswNode {
        pub fn new(id: usize, embedding: Vec<f32>, level: usize) -> Self {
            Self {
                embedding,
                level,
                id,
                links: vec![0; level + 1],
                score: 0.0,
            }
        }

        pub fn squared_distance(&self, other: &Self) -> f32 {
            squared_euclidean(&self.embedding, &other.embedding)
        }

        pub fn cosine_similarity(&self, other: &Self) -> f32 {
            cosine_similarity(&self.embedding, &other.embedding)
        }
    }

    impl fmt::Display for HnswNode {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "VectorNode[id={}, embed_dim={}, score={:.3}]",
                self.id,
                self.embedding.len(),
                self.score
            )
        }
    }

    use crate::distance::{cosine_similarity, squared_euclidean};
}

/// Exact in-memory vector index. Every query scans all vectors (`O(N * dim)`),
/// making this implementation useful as a correctness and recall baseline.
pub mod index {
    use crate::distance::{cosine_distance, l1_distance, squared_euclidean};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum DistanceMetric {
        SquaredEuclidean,
        Cosine,
        Manhattan,
    }

    /// Configuration for an index.
    #[derive(Clone, Debug)]
    pub struct IndexParams {
        pub dim: usize,
    }

    impl Default for IndexParams {
        fn default() -> Self {
            Self { dim: 768 }
        }
    }

    pub struct VectorIndex {
        params: IndexParams,
        vectors: Vec<f32>,
        ids: Vec<usize>,
        next_id: usize,
    }

    impl VectorIndex {
        pub fn new(params: IndexParams) -> Self {
            assert!(params.dim > 0, "Vector dimension must be greater than zero");
            Self {
                params,
                vectors: Vec::new(),
                ids: Vec::new(),
                next_id: 0,
            }
        }

        pub fn insert(&mut self, embedding: Vec<f32>) -> usize {
            assert_eq!(
                embedding.len(),
                self.params.dim,
                "Embedding dimension {} doesn't match index dimension {}",
                embedding.len(),
                self.params.dim
            );

            while self.contains_id(self.next_id) {
                self.next_id = self
                    .next_id
                    .checked_add(1)
                    .expect("No vector IDs remain available");
            }
            let id = self.next_id;
            self.insert_with_id(id, embedding)
                .expect("Generated vector ID must be unused");
            self.next_id = self.next_id.saturating_add(1);
            id
        }

        pub fn insert_with_id(&mut self, id: usize, embedding: Vec<f32>) -> Result<(), String> {
            if embedding.len() != self.params.dim {
                return Err(format!(
                    "Vector has dimension {}, but index dimension is {}",
                    embedding.len(),
                    self.params.dim
                ));
            }
            if self.contains_id(id) {
                return Err(format!("Vector ID {id} already exists"));
            }

            self.ids.push(id);
            self.vectors.extend_from_slice(&embedding);
            Ok(())
        }

        pub fn contains_id(&self, id: usize) -> bool {
            self.ids.contains(&id)
        }

        pub fn records(&self) -> impl Iterator<Item = (usize, &[f32])> {
            let dim = self.params.dim;
            self.ids.iter().copied().zip(self.vectors.chunks_exact(dim))
        }

        /// Return up to `k` nearest vectors, ordered by squared Euclidean
        /// distance and then by insertion ID for deterministic ties.
        pub fn search(&self, query: &[f32], k: usize) -> Vec<(usize, f32)> {
            self.search_with_metric(query, k, DistanceMetric::SquaredEuclidean)
        }

        pub fn search_with_metric(
            &self,
            query: &[f32],
            k: usize,
            metric: DistanceMetric,
        ) -> Vec<(usize, f32)> {
            assert_eq!(
                query.len(),
                self.params.dim,
                "Query dimension {} doesn't match index dimension {}",
                query.len(),
                self.params.dim
            );

            let dim = self.params.dim;
            let mut results: Vec<_> = self
                .vectors
                .chunks_exact(dim)
                .enumerate()
                .map(|position| {
                    let (id, embedding) = (self.ids[position.0], position.1);
                    let distance = match metric {
                        DistanceMetric::SquaredEuclidean => squared_euclidean(embedding, query),
                        DistanceMetric::Cosine => cosine_distance(embedding, query),
                        DistanceMetric::Manhattan => l1_distance(embedding, query),
                    };
                    (id, distance)
                })
                .collect();

            results.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            results.truncate(k);
            results
        }

        pub fn len(&self) -> usize {
            self.ids.len()
        }

        pub fn is_empty(&self) -> bool {
            self.ids.is_empty()
        }

        pub fn params(&self) -> &IndexParams {
            &self.params
        }
    }

    impl Default for VectorIndex {
        fn default() -> Self {
            Self::new(IndexParams::default())
        }
    }

    /// Compatibility name retained for existing callers. This is not an HNSW
    /// implementation; use `VectorIndex` to make the exact-search behavior clear.
    pub type HnswIndex = VectorIndex;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn exact_search_returns_nearest_vectors_in_order() {
            let mut index = VectorIndex::new(IndexParams { dim: 2 });
            index.insert(vec![10.0, 10.0]);
            index.insert(vec![1.0, 1.0]);
            index.insert(vec![2.0, 2.0]);

            assert_eq!(index.search(&[1.5, 1.5], 2), vec![(1, 0.5), (2, 0.5)]);
        }

        #[test]
        fn search_caps_results_and_handles_zero_k() {
            let mut index = VectorIndex::new(IndexParams { dim: 1 });
            index.insert(vec![0.0]);
            index.insert(vec![1.0]);

            assert_eq!(index.search(&[0.0], 10).len(), 2);
            assert!(index.search(&[0.0], 0).is_empty());
        }

        #[test]
        fn supports_custom_ids_and_rejects_duplicates() {
            let mut index = VectorIndex::new(IndexParams { dim: 1 });
            index.insert_with_id(42, vec![1.0]).unwrap();
            assert_eq!(index.search(&[1.0], 1), vec![(42, 0.0)]);
            assert!(index.insert_with_id(42, vec![2.0]).is_err());
            assert!(index.insert_with_id(7, vec![2.0, 3.0]).is_err());
        }

        #[test]
        fn metrics_return_sorted_distance_results() {
            let mut index = VectorIndex::new(IndexParams { dim: 2 });
            index.insert_with_id(12, vec![1.0, 0.0]).unwrap();
            index.insert_with_id(4, vec![0.0, 2.0]).unwrap();

            assert_eq!(
                index.search_with_metric(&[1.0, 0.0], 2, DistanceMetric::Cosine),
                vec![(12, 0.0), (4, 1.0)]
            );
            assert_eq!(
                index.search_with_metric(&[1.0, 0.0], 1, DistanceMetric::Manhattan),
                vec![(12, 0.0)]
            );
        }

        #[test]
        fn empty_index_returns_no_results() {
            let index = VectorIndex::new(IndexParams { dim: 2 });
            assert!(index.search(&[0.0, 0.0], 5).is_empty());
        }
    }
}

/// Distance metric utilities.
pub mod distance {
    pub fn squared_euclidean(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vector dimensions must match");
        a.iter().zip(b).map(|(ai, bi)| (ai - bi).powi(2)).sum()
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vector dimensions must match");
        let dot: f32 = a.iter().zip(b).map(|(ai, bi)| ai * bi).sum();
        let norm_a = a.iter().map(|value| value * value).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|value| value * value).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        (1.0 - cosine_similarity(a, b)).clamp(0.0, 2.0)
    }

    pub fn l1_distance(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vector dimensions must match");
        a.iter().zip(b).map(|(ai, bi)| (ai - bi).abs()).sum()
    }

    pub fn manhattan(a: &[f32], b: &[f32]) -> f32 {
        l1_distance(a, b)
    }
}

pub use distance::{cosine_distance, cosine_similarity, l1_distance, squared_euclidean};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_metrics_match_expected_values() {
        assert_eq!(squared_euclidean(&[0.0, 0.0], &[3.0, 3.0]), 18.0);
        assert_eq!(l1_distance(&[1.0, 1.0], &[2.0, 2.0]), 2.0);
        assert!((cosine_similarity(&[1.0, 1.0], &[1.0, 1.0]) - 1.0).abs() < 1e-6);
        assert_eq!(cosine_distance(&[1.0, 0.0], &[1.0, 0.0]), 0.0);
    }

    #[test]
    fn zero_vector_has_zero_cosine_similarity() {
        assert_eq!(cosine_similarity(&[0.0, 0.0], &[1.0, 0.0]), 0.0);
    }
}
