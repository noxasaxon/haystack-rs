use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;

use crate::components::joiners::document_joiner::{DocumentJoiner, JoinMode};
use crate::Component;
use haystack_dataclasses::Document;

fn create_test_documents(count: usize, base_score: f32) -> Vec<Document> {
    (0..count)
        .map(|i| {
            Document::with_params(
                Some(format!("doc{}", i)),
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
fn test_document_joiner_concatenate() -> Result<()> {
    let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, None, true);
    
    let list1 = create_test_documents(3, 0.5);
    let list2 = create_test_documents(2, 0.8);
    
    let documents_value = serde_json::to_value(vec![list1, list2])?;
    let inputs = HashMap::from([("documents".to_string(), documents_value)]);
    
    let outputs = joiner.run(inputs)?;
    let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone())?;
    
    assert_eq!(result_docs.len(), 5, "Should have 5 unique documents");
    Ok(())
}

#[test]
fn test_document_joiner_with_duplicates() -> Result<()> {
    let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, None, true);
    
    let list1 = create_duplicate_documents(3, 0.5);
    let list2 = create_duplicate_documents(3, 0.8);
    
    let documents_value = serde_json::to_value(vec![list1, list2])?;
    let inputs = HashMap::from([("documents".to_string(), documents_value)]);
    
    let outputs = joiner.run(inputs)?;
    let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone())?;
    
    assert_eq!(result_docs.len(), 3, "Should have 3 documents after deduplication");
    
    // Check that documents with higher scores were kept
    for doc in &result_docs {
        assert!(doc.score.unwrap() >= 0.8, "Document should have higher score from list2");
    }
    
    Ok(())
}

#[test]
fn test_document_joiner_merge() -> Result<()> {
    let joiner = DocumentJoiner::new(JoinMode::Merge, Some(vec![0.7, 0.3]), None, true);
    
    let list1 = create_duplicate_documents(3, 0.5);
    let list2 = create_duplicate_documents(3, 1.0);
    
    let documents_value = serde_json::to_value(vec![list1, list2])?;
    let inputs = HashMap::from([("documents".to_string(), documents_value)]);
    
    let outputs = joiner.run(inputs)?;
    let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone())?;
    
    assert_eq!(result_docs.len(), 3, "Should have 3 documents after merging");
    Ok(())
}

#[test]
fn test_document_joiner_with_top_k() -> Result<()> {
    let joiner = DocumentJoiner::new(JoinMode::Concatenate, None, Some(2), true);
    
    let list1 = create_test_documents(3, 0.5);
    let list2 = create_test_documents(2, 0.8);
    
    let documents_value = serde_json::to_value(vec![list1, list2])?;
    let inputs = HashMap::from([("documents".to_string(), documents_value)]);
    
    let outputs = joiner.run(inputs)?;
    let result_docs: Vec<Document> = serde_json::from_value(outputs["documents"].clone())?;
    
    assert_eq!(result_docs.len(), 2, "Should return only 2 documents with top_k=2");
    
    // Documents should be sorted by score (highest first)
    assert!(
        result_docs[0].score.unwrap() >= result_docs[1].score.unwrap(),
        "Results should be sorted by score"
    );
    
    Ok(())
}