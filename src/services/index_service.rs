use bloom::BloomFilter;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct SparseIndex {
    index_granularity: usize,
    field_indexes: HashMap<String, BloomFilter>,
}

impl SparseIndex {
    pub fn new(granularity: usize) -> Self {
        Self {
            index_granularity: granularity,
            field_indexes: HashMap::new(),
        }
    }

    pub fn create_index(&mut self, data: &Map<String, Value>, field: &str) -> Map<String, Value> {
        let mut index = Map::new();
        let mut counter = 0;

        for (id, obj) in data {
            if counter % self.index_granularity == 0 {
                if let Some(value) = obj.get(field) {
                    index.insert(id.clone(), value.clone());
                }
            }
            counter += 1;
        }

        let filter = self
            .field_indexes
            .entry(field.to_string())
            .or_insert_with(|| BloomFilter::with_size(100000, 1));

        for value in index.values() {
            if let Some(s) = value.as_str() {
                filter.insert(&s.to_string());
            }
        }
        index
    }

    pub fn exists_in_index(&self, field: &str, value: &str) -> bool {
        if let Some(filter) = self.field_indexes.get(field) {
            filter.contains(&value)
        } else {
            false
        }
    }
}
