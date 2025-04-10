/*!
 * DocumentJoiner component
 * 
 * Joins multiple lists of documents into a single list using various join modes.
 */

use std::collections::HashMap;
use std::cmp::Ordering;
use anyhow::{Result, anyhow};
use serde_json::Value;
use serde::{Serialize, Deserialize};

use crate::{Component, ComponentBase, InputSocket, OutputSocket};
use haystack_dataclasses::Document;

/// Enum for join mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinMode {
    /// Keeps the highest-scored document in case of duplicates
    #[serde(rename = "concatenate")]
    Concatenate,
    
    /// Calculates a weighted sum of scores for duplicates and merges them
    #[serde(rename = "merge")]
    Merge,
    
    /// Merges and assigns scores based on reciprocal rank fusion
    #[serde(rename = "reciprocal_rank_fusion")]
    ReciprocalRankFusion,
    
    /// Merges and assigns scores based on scores distribution in each Retriever
    #[serde(rename = "distribution_based_rank_fusion")]
    DistributionBasedRankFusion,
}

impl JoinMode {
    /// Convert a string to a JoinMode enum
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "concatenate" => Ok(JoinMode::Concatenate),
            "merge" => Ok(JoinMode::Merge),
            "reciprocal_rank_fusion" => Ok(JoinMode::ReciprocalRankFusion),
            "distribution_based_rank_fusion" => Ok(JoinMode::DistributionBasedRankFusion),
            _ => Err(anyhow!(
                "Unknown join mode '{}'. Supported modes in DocumentJoiner are: \
                concatenate, merge, reciprocal_rank_fusion, distribution_based_rank_fusion",
                s
            )),
        }
    }
    
    /// Convert JoinMode to string
    pub fn to_str(&self) -> &'static str {
        match self {
            JoinMode::Concatenate => "concatenate",
            JoinMode::Merge => "merge",
            JoinMode::ReciprocalRankFusion => "reciprocal_rank_fusion",
            JoinMode::DistributionBasedRankFusion => "distribution_based_rank_fusion",
        }
    }
}

impl std::fmt::Display for JoinMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// Joins multiple lists of documents into a single list.
///
/// It supports different join modes:
/// - concatenate: Keeps the highest-scored document in case of duplicates.
/// - merge: Calculates a weighted sum of scores for duplicates and merges them.
/// - reciprocal_rank_fusion: Merges and assigns scores based on reciprocal rank fusion.
/// - distribution_based_rank_fusion: Merges and assigns scores based on scores distribution in each Retriever.
pub struct DocumentJoiner {
    /// Base component implementation
    base: ComponentBase,
    
    /// Join mode for handling duplicate documents
    join_mode: JoinMode,
    
    /// Weights for each list of documents to influence how they're joined
    weights: Option<Vec<f32>>,
    
    /// The maximum number of documents to return
    top_k: Option<usize>,
    
    /// Whether to sort documents by score
    sort_by_score: bool,
}

impl DocumentJoiner {
    /// Create a new DocumentJoiner component
    pub fn new(
        join_mode: JoinMode,
        weights: Option<Vec<f32>>,
        top_k: Option<usize>,
        sort_by_score: bool,
    ) -> Self {
        // Create input and output sockets
        let mut input_sockets = HashMap::new();
        input_sockets.insert(
            "documents".to_string(),
            InputSocket::new_variadic::<Vec<Document>>(
                "documents".to_string(),
                "Vec<Document>".to_string(),
                None,
                false,
            ),
        );
        
        let mut output_sockets = HashMap::new();
        output_sockets.insert(
            "documents".to_string(),
            OutputSocket::new(
                "documents".to_string(),
                std::any::TypeId::of::<Vec<Document>>(),
                "Vec<Document>".to_string(),
            ),
        );
        
        // Create init parameters
        let mut init_parameters = HashMap::new();
        init_parameters.insert("join_mode".to_string(), Value::String(join_mode.to_str().to_string()));
        
        if let Some(weights_vec) = &weights {
            let weight_values: Vec<Value> = weights_vec.iter().map(|&w| Value::from(w)).collect();
            init_parameters.insert("weights".to_string(), Value::Array(weight_values));
        }
        
        if let Some(top_k_val) = top_k {
            init_parameters.insert("top_k".to_string(), Value::from(top_k_val));
        }
        
        init_parameters.insert("sort_by_score".to_string(), Value::from(sort_by_score));
        
        // Normalize weights if provided
        let normalized_weights = weights.map(|w| {
            let sum: f32 = w.iter().sum();
            w.into_iter().map(|weight| weight / sum).collect()
        });
        
        Self {
            base: ComponentBase::new(init_parameters, input_sockets, output_sockets),
            join_mode,
            weights: normalized_weights,
            top_k,
            sort_by_score,
        }
    }
    
    /// Create instance with default parameters
    pub fn default() -> Self {
        Self::new(JoinMode::Concatenate, None, None, true)
    }
    
    /// Concatenate multiple lists of Documents and return only the Document with the highest score for duplicates
    fn concatenate(&self, document_lists: Vec<Vec<Document>>) -> Vec<Document> {
        let mut docs_per_id: HashMap<String, Vec<Document>> = HashMap::new();
        
        // Collect all documents grouped by ID
        for documents in document_lists {
            for doc in documents {
                docs_per_id.entry(doc.id.clone()).or_default().push(doc);
            }
        }
        
        // Select the document with the best score from each group
        docs_per_id
            .into_iter()
            .map(|(_, docs)| {
                docs.into_iter().max_by(|a, b| {
                    let a_score = a.score.unwrap_or(f32::NEG_INFINITY);
                    let b_score = b.score.unwrap_or(f32::NEG_INFINITY);
                    a_score.partial_cmp(&b_score).unwrap_or(Ordering::Equal)
                }).unwrap()
            })
            .collect()
    }
    
    /// Merge multiple lists of Documents and calculate a weighted sum of the scores of duplicate Documents
    fn merge(&self, document_lists: Vec<Vec<Document>>) -> Vec<Document> {
        // If no documents, return empty list
        if document_lists.is_empty() {
            return Vec::new();
        }
        
        // Calculate scores for each document ID
        let mut scores_map: HashMap<String, f32> = HashMap::new();
        let mut documents_map: HashMap<String, Document> = HashMap::new();
        
        // Get weights (or use defaults if none specified)
        let weights = match &self.weights {
            Some(w) => w.clone(),
            None => {
                let weight = 1.0 / (document_lists.len() as f32);
                vec![weight; document_lists.len()]
            }
        };
        
        // Calculate weighted scores for each document
        for (documents, weight) in document_lists.iter().zip(weights.iter()) {
            for doc in documents {
                let score = doc.score.unwrap_or(0.0) * weight;
                *scores_map.entry(doc.id.clone()).or_insert(0.0) += score;
                documents_map.insert(doc.id.clone(), doc.clone());
            }
        }
        
        // Update document scores
        let mut result = Vec::new();
        for (id, doc) in documents_map {
            let mut updated_doc = doc;
            updated_doc.score = Some(scores_map[&id]);
            result.push(updated_doc);
        }
        
        result
    }
    
    /// Merge multiple lists of Documents and assign scores based on reciprocal rank fusion
    fn reciprocal_rank_fusion(&self, document_lists: Vec<Vec<Document>>) -> Vec<Document> {
        // If no documents, return empty list
        if document_lists.is_empty() {
            return Vec::new();
        }
        
        // Constant k (60 from the original paper, +1 as Rust is 0-indexed)
        let k = 61_f32;
        
        // Calculate scores for each document ID
        let mut scores_map: HashMap<String, f32> = HashMap::new();
        let mut documents_map: HashMap<String, Document> = HashMap::new();
        
        // Get weights (or use defaults if none specified)
        let weights = match &self.weights {
            Some(w) => w.clone(),
            None => {
                let weight = 1.0 / (document_lists.len() as f32);
                vec![weight; document_lists.len()]
            }
        };
        
        // Calculate weighted RRF scores
        for (documents, weight) in document_lists.iter().zip(weights.iter()) {
            for (rank, doc) in documents.iter().enumerate() {
                let rrf_score = (weight * document_lists.len() as f32) / (k + rank as f32);
                *scores_map.entry(doc.id.clone()).or_insert(0.0) += rrf_score;
                documents_map.insert(doc.id.clone(), doc.clone());
            }
        }
        
        // Normalize scores
        // Maximum possible score is achieved by a document that is ranked first in all lists with non-zero weight
        let max_score = document_lists.len() as f32 / k;
        
        // Update document scores
        let mut result = Vec::new();
        for (id, doc) in documents_map {
            let mut updated_doc = doc;
            updated_doc.score = Some(scores_map[&id] / max_score);
            result.push(updated_doc);
        }
        
        result
    }
    
    /// Merge multiple lists of Documents and assign scores based on Distribution-Based Score Fusion
    fn distribution_based_rank_fusion(&self, document_lists: Vec<Vec<Document>>) -> Vec<Document> {
        let mut documents_copy = document_lists.clone();
        
        // Normalize scores within each document list based on distribution
        for documents in &mut documents_copy {
            if documents.is_empty() {
                continue;
            }
            
            // Extract scores
            let scores: Vec<f32> = documents
                .iter()
                .map(|doc| doc.score.unwrap_or(0.0))
                .collect();
            
            // Calculate mean and standard deviation
            let mean_score = scores.iter().sum::<f32>() / scores.len() as f32;
            let variance = scores
                .iter()
                .map(|&s| (s - mean_score).powi(2))
                .sum::<f32>() / scores.len() as f32;
            let std_dev = variance.sqrt();
            
            // Set boundary for normalization (3 standard deviations)
            let min_score = mean_score - 3.0 * std_dev;
            let max_score = mean_score + 3.0 * std_dev;
            let delta_score = max_score - min_score;
            
            // Normalize scores
            for doc in documents.iter_mut() {
                let original_score = doc.score.unwrap_or(0.0);
                let normalized_score = if delta_score != 0.0 {
                    (original_score - min_score) / delta_score
                } else {
                    0.0 // All docs have the same score, uninformative for the query
                };
                doc.score = Some(normalized_score);
            }
        }
        
        // Concatenate the normalized document lists
        self.concatenate(documents_copy)
    }
}

impl Component for DocumentJoiner {
    fn run(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>> {
        // Extract the variadic documents input
        let documents_variadic = crate::extract_variadic!(inputs, "documents", Vec<Document>);
        
        // Convert to Vec<Vec<Document>> for processing
        let document_lists: Vec<Vec<Document>> = documents_variadic.iter().cloned().collect();
        
        // Apply appropriate join function based on mode
        let mut output_documents = match self.join_mode {
            JoinMode::Concatenate => self.concatenate(document_lists),
            JoinMode::Merge => self.merge(document_lists),
            JoinMode::ReciprocalRankFusion => self.reciprocal_rank_fusion(document_lists),
            JoinMode::DistributionBasedRankFusion => self.distribution_based_rank_fusion(document_lists),
        };
        
        // Sort documents by score if required
        if self.sort_by_score {
            output_documents.sort_by(|a, b| {
                let a_score = a.score.unwrap_or(f32::NEG_INFINITY);
                let b_score = b.score.unwrap_or(f32::NEG_INFINITY);
                // Sort in descending order (higher scores first)
                b_score.partial_cmp(&a_score).unwrap_or(Ordering::Equal)
            });
        }
        
        // Apply top_k limit if specified
        if let Some(top_k) = self.top_k {
            output_documents.truncate(top_k);
        }
        
        // Return the results
        let mut outputs = HashMap::new();
        outputs.insert("documents".to_string(), serde_json::to_value(output_documents)?);
        
        Ok(outputs)
    }
    
    fn input_sockets(&self) -> &HashMap<String, InputSocket> {
        self.base.input_sockets()
    }
    
    fn output_sockets(&self) -> &HashMap<String, OutputSocket> {
        self.base.output_sockets()
    }
    
    fn init_parameters(&self) -> &HashMap<String, Value> {
        self.base.init_parameters()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haystack_dataclasses::Document;
    
    fn create_test_documents(count: usize, base_score: f32) -> Vec<Document> {
        (0..count)
            .map(|i| {
                Document::with_params(
                    Some(format!("doc{}_score{}", i, base_score)),  // Ensure unique IDs across test sets
                    Some(format!("Content {}", i)),
                    None,
                    None,
                    Some(base_score + (i as f32) * 0.1),
                    None,
                    None,
                )
            })
            .collect()
    }
    
    fn create_duplicate_documents(count: usize, base_score: f32) -> Vec<Document> {
        (0..count)
            .map(|i| {
                let id = format!("doc{}", i % 3); // Creates duplicates
                Document::with_params(
                    Some(id),
                    Some(format!("Content {}", i)),
                    None,
                    None,
                    Some(base_score + (i as f32) * 0.1),
                    None,
                    None,
                )
            })
            .collect()
    }
    
    #[test]
    fn test_concatenate_mode() {
        let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, None, true);
        
        // Create two sets of documents with different IDs
        let mut list1 = Vec::new();
        for i in 0..3 {
            list1.push(Document::with_params(
                Some(format!("doc{}", i)),
                Some(format!("Content {}", i)),
                None,
                None,
                Some(0.5 + (i as f32) * 0.1),
                None,
                None,
            ));
        }

        let mut list2 = Vec::new();
        for i in 3..5 {  // Using different IDs than list1
            list2.push(Document::with_params(
                Some(format!("doc{}", i)),
                Some(format!("Content {}", i)),
                None,
                None,
                Some(0.8 + (i as f32) * 0.1),
                None,
                None,
            ));
        }
        
        let documents_value = serde_json::to_value(vec![list1, list2]).unwrap();
        let inputs = HashMap::from([("documents".to_string(), documents_value)]);
        
        let outputs = joiner.run(inputs).unwrap();
        let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone()).unwrap();
        
        assert_eq!(result_docs.len(), 5, "Should have 5 total documents when IDs don't overlap");
    }
    
    #[test]
    fn test_concatenate_mode_with_duplicates() {
        let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, None, true);
        
        let list1 = create_duplicate_documents(3, 0.5);
        let list2 = create_duplicate_documents(3, 0.8);
        
        let documents_value = serde_json::to_value(vec![list1, list2]).unwrap();
        let inputs = HashMap::from([("documents".to_string(), documents_value)]);
        
        let outputs = joiner.run(inputs).unwrap();
        let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone()).unwrap();
        
        // Should have 3 documents after deduplication (one for each unique ID)
        assert_eq!(result_docs.len(), 3);
        
        // The documents with higher scores should be kept
        assert!(result_docs.iter().all(|doc| doc.score.unwrap() >= 0.8));
    }
    
    #[test]
    fn test_merge_mode() {
        let joiner = DocumentJoiner::new(JoinMode::Merge, Some(vec![0.7, 0.3]), None, true);
        
        let list1 = create_duplicate_documents(3, 0.5);
        let list2 = create_duplicate_documents(3, 1.0);
        
        let documents_value = serde_json::to_value(vec![list1, list2]).unwrap();
        let inputs = HashMap::from([("documents".to_string(), documents_value)]);
        
        let outputs = joiner.run(inputs).unwrap();
        let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone()).unwrap();
        
        // Should have 3 documents after merging (one for each unique ID)
        assert_eq!(result_docs.len(), 3);
        
        // Check that scores are weighted appropriately
        for doc in result_docs {
            let id = doc.id.strip_prefix("doc").unwrap().parse::<usize>().unwrap();
            let expected_score1 = 0.5 + (id as f32) * 0.1;
            let expected_score2 = 1.0 + (id as f32) * 0.1;
            let expected_merged = expected_score1 * 0.7 + expected_score2 * 0.3;
            
            assert!((doc.score.unwrap() - expected_merged).abs() < 0.001);
        }
    }
    
    #[test]
    fn test_top_k_limit() {
        let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, Some(2), true);
        
        let list1 = create_test_documents(3, 0.5);
        let list2 = create_test_documents(2, 0.8);
        
        let documents_value = serde_json::to_value(vec![list1, list2]).unwrap();
        let inputs = HashMap::from([("documents".to_string(), documents_value)]);
        
        let outputs = joiner.run(inputs).unwrap();
        let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone()).unwrap();
        
        // Should have only 2 documents due to top_k
        assert_eq!(result_docs.len(), 2);
        
        // Documents should be sorted by score (highest first)
        assert!(result_docs[0].score.unwrap() >= result_docs[1].score.unwrap());
    }
    
    #[test]
    fn test_ordering_without_sort() {
        let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, None, false);
        
        // Create two sets of documents with different IDs
        let mut list1 = Vec::new();
        for i in 0..3 {
            list1.push(Document::with_params(
                Some(format!("doc{}", i)),
                Some(format!("Content {}", i)),
                None,
                None,
                Some(0.5 + (i as f32) * 0.1),
                None,
                None,
            ));
        }

        let mut list2 = Vec::new();
        for i in 3..5 {  // Using different IDs than list1
            list2.push(Document::with_params(
                Some(format!("doc{}", i)),
                Some(format!("Content {}", i)),
                None,
                None,
                Some(0.8 + (i as f32) * 0.1),
                None,
                None,
            ));
        }
        
        let documents_value = serde_json::to_value(vec![list1, list2]).unwrap();
        let inputs = HashMap::from([("documents".to_string(), documents_value)]);
        
        let outputs = joiner.run(inputs).unwrap();
        let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone()).unwrap();
        
        // Should still have 5 documents
        assert_eq!(result_docs.len(), 5, "Should have 5 total documents when IDs don't overlap");
        
        // But ordering might not be by score
        // (We can't assert specific ordering since it's implementation-dependent without sorting)
    }
}