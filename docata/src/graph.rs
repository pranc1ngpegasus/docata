use crate::catalog::Catalog;
use std::collections::HashMap;

pub struct Graph {
    forward: HashMap<String, Vec<String>>,
    reverse: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn from_catalog(catalog: &Catalog) -> Self {
        let mut forward = HashMap::new();
        let mut reverse = HashMap::new();

        for edge in &catalog.edges {
            forward
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());

            reverse
                .entry(edge.to.clone())
                .or_insert_with(Vec::new)
                .push(edge.from.clone());
        }

        Self { forward, reverse }
    }

    #[must_use]
    pub fn deps(
        &self,
        id: &str,
    ) -> Vec<String> {
        self.forward.get(id).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn refs(
        &self,
        id: &str,
    ) -> Vec<String> {
        self.reverse.get(id).cloned().unwrap_or_default()
    }
}
