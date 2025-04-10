#[cfg(test)]
mod tests {
    extern crate bm25;
    
    #[test]
    fn test_bm25_api() {
        // Create test documents
        let docs = vec![
            ("1", "The quick brown fox jumps over the lazy dog"),
            ("2", "A fast yellow fox leaps across a sleeping hound"),
            ("3", "The five boxing wizards jump quickly"),
        ];
        
        // Create BM25 instance - note this is different from the imports seen earlier
        let bm25_model = bm25::BM25::new(&docs, Default::default())
            .expect("Failed to create BM25 instance");
        
        // Search for documents
        let results = bm25_model.search("fox jumps", None)
            .expect("Failed to search");
        
        println!("Results: {:?}", results);
        assert!(!results.is_empty());
    }
}