use crate::scan::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::{self, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DuplicateId {
    pub id: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnresolvedDependency {
    pub from_id: String,
    pub to_id: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct DependencyCycle {
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub duplicate_ids: Vec<DuplicateId>,
    pub unresolved_dependencies: Vec<UnresolvedDependency>,
    pub dependency_cycles: Vec<DependencyCycle>,
}

impl ValidationReport {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.duplicate_ids.is_empty()
            && self.unresolved_dependencies.is_empty()
            && self.dependency_cycles.is_empty()
    }
}

impl Display for ValidationReport {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result {
        writeln!(f, "validation failed:")?;

        if !self.duplicate_ids.is_empty() {
            writeln!(f, "- duplicate ids: {}", self.duplicate_ids.len())?;
            for duplicate in &self.duplicate_ids {
                writeln!(
                    f,
                    "  - `{}` appears in: {}",
                    duplicate.id,
                    duplicate.paths.join(", ")
                )?;
            }
        }

        if !self.unresolved_dependencies.is_empty() {
            writeln!(
                f,
                "- unresolved dependencies: {}",
                self.unresolved_dependencies.len()
            )?;
            for unresolved in &self.unresolved_dependencies {
                writeln!(
                    f,
                    "  - `{}` -> `{}` (from {})",
                    unresolved.from_id, unresolved.to_id, unresolved.path
                )?;
            }
        }

        if !self.dependency_cycles.is_empty() {
            writeln!(f, "- dependency cycles: {}", self.dependency_cycles.len())?;
            for cycle in &self.dependency_cycles {
                if let Some(first) = cycle.ids.first() {
                    let mut path = cycle.ids.join(" -> ");
                    path.push_str(" -> ");
                    path.push_str(first);
                    writeln!(f, "  - {path}")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("{report}")]
pub struct ValidationError {
    report: ValidationReport,
}

impl ValidationError {
    #[must_use]
    pub const fn report(&self) -> &ValidationReport {
        &self.report
    }
}

/// Validate scanned entries.
///
/// # Errors
///
/// Returns `ValidationError` if duplicate IDs, unresolved dependencies, or
/// dependency cycles are detected.
pub fn validate_entries(entries: &[Entry]) -> Result<(), ValidationError> {
    let report = build_validation_report(entries);

    if report.is_empty() {
        Ok(())
    } else {
        Err(ValidationError { report })
    }
}

fn build_validation_report(entries: &[Entry]) -> ValidationReport {
    ValidationReport {
        duplicate_ids: find_duplicate_ids(entries),
        unresolved_dependencies: find_unresolved_dependencies(entries),
        dependency_cycles: find_dependency_cycles(entries),
    }
}

fn find_duplicate_ids(entries: &[Entry]) -> Vec<DuplicateId> {
    let mut by_id: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    for entry in entries {
        by_id
            .entry(entry.id.as_str())
            .or_default()
            .push(entry.path.to_string_lossy().to_string());
    }

    by_id
        .into_iter()
        .filter_map(|(id, mut paths)| {
            if paths.len() < 2 {
                return None;
            }

            paths.sort();
            paths.dedup();

            Some(DuplicateId {
                id: id.to_owned(),
                paths,
            })
        })
        .collect()
}

fn find_unresolved_dependencies(entries: &[Entry]) -> Vec<UnresolvedDependency> {
    let known_ids = entries
        .iter()
        .map(|entry| entry.id.as_str())
        .collect::<HashSet<_>>();

    let mut ordered_entries = entries.iter().collect::<Vec<_>>();
    ordered_entries.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(left.path.as_os_str().cmp(right.path.as_os_str()))
    });

    let mut unresolved_dependencies = Vec::new();

    for entry in ordered_entries {
        let mut deps = entry.deps.clone();
        deps.sort();
        deps.dedup();

        for dep in deps {
            if !known_ids.contains(dep.as_str()) {
                unresolved_dependencies.push(UnresolvedDependency {
                    from_id: entry.id.clone(),
                    to_id: dep,
                    path: entry.path.to_string_lossy().to_string(),
                });
            }
        }
    }

    unresolved_dependencies
}

fn find_dependency_cycles(entries: &[Entry]) -> Vec<DependencyCycle> {
    let known_ids = entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<BTreeSet<_>>();

    let mut adjacency = known_ids
        .iter()
        .map(|id| (id.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();

    for entry in entries {
        for dep in &entry.deps {
            if known_ids.contains(dep) {
                adjacency
                    .entry(entry.id.clone())
                    .or_default()
                    .insert(dep.clone());
            }
        }
    }

    let mut components = strongly_connected_components(&adjacency);

    components.retain(|component| {
        if component.len() > 1 {
            return true;
        }

        component.first().is_some_and(|id| {
            adjacency
                .get(id)
                .is_some_and(|neighbors| neighbors.contains(id))
        })
    });

    components.sort_by_key(|left| left.join("\0"));

    components
        .into_iter()
        .map(|ids| DependencyCycle { ids })
        .collect()
}

fn strongly_connected_components(
    adjacency: &BTreeMap<String, BTreeSet<String>>
) -> Vec<Vec<String>> {
    struct TarjanState {
        index: usize,
        stack: Vec<String>,
        on_stack: HashSet<String>,
        indices: HashMap<String, usize>,
        low_link: HashMap<String, usize>,
        components: Vec<Vec<String>>,
    }

    fn strong_connect(
        node: &str,
        adjacency: &BTreeMap<String, BTreeSet<String>>,
        state: &mut TarjanState,
    ) {
        let node_key = node.to_owned();

        state.indices.insert(node_key.clone(), state.index);
        state.low_link.insert(node_key.clone(), state.index);
        state.index += 1;
        state.stack.push(node_key.clone());
        state.on_stack.insert(node_key.clone());

        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                if !state.indices.contains_key(neighbor) {
                    strong_connect(neighbor, adjacency, state);

                    if let Some(&neighbor_low_link) = state.low_link.get(neighbor) {
                        let node_low_link = state.low_link.get(node).copied().unwrap_or(usize::MAX);
                        let next_low_link = node_low_link.min(neighbor_low_link);
                        state.low_link.insert(node_key.clone(), next_low_link);
                    }
                } else if state.on_stack.contains(neighbor)
                    && let Some(&neighbor_index) = state.indices.get(neighbor)
                {
                    let node_low_link = state.low_link.get(node).copied().unwrap_or(usize::MAX);
                    let next_low_link = node_low_link.min(neighbor_index);
                    state.low_link.insert(node_key.clone(), next_low_link);
                }
            }
        }

        let Some(&node_index) = state.indices.get(node) else {
            return;
        };
        let Some(&node_low_link) = state.low_link.get(node) else {
            return;
        };

        if node_index == node_low_link {
            let mut component = Vec::new();

            while let Some(candidate) = state.stack.pop() {
                state.on_stack.remove(&candidate);
                let done = candidate == node;
                component.push(candidate);
                if done {
                    break;
                }
            }

            component.sort();
            state.components.push(component);
        }
    }

    let mut state = TarjanState {
        index: 0,
        stack: Vec::new(),
        on_stack: HashSet::new(),
        indices: HashMap::new(),
        low_link: HashMap::new(),
        components: Vec::new(),
    };

    for node in adjacency.keys() {
        if !state.indices.contains_key(node) {
            strong_connect(node, adjacency, &mut state);
        }
    }

    state.components
}

#[cfg(test)]
mod tests {
    use super::validate_entries;
    use crate::scan::Entry;
    use std::path::PathBuf;

    fn entry(
        id: &str,
        deps: &[&str],
        path: &str,
    ) -> Entry {
        Entry {
            id: id.to_owned(),
            deps: deps.iter().map(ToString::to_string).collect(),
            path: PathBuf::from(path),
            node_type: None,
            domain: None,
            status: None,
            source_of_truth: None,
        }
    }

    #[test]
    fn detects_duplicate_unresolved_and_cycle() {
        let entries = vec![
            entry("a", &["b", "missing"], "docs/a.md"),
            entry("b", &["a"], "docs/b.md"),
            entry("a", &[], "docs/a-duplicate.md"),
        ];

        let error = validate_entries(&entries).expect_err("validation must fail");
        let report = error.report();

        assert_eq!(report.duplicate_ids.len(), 1);
        assert_eq!(report.duplicate_ids[0].id, "a");

        assert_eq!(report.unresolved_dependencies.len(), 1);
        assert_eq!(report.unresolved_dependencies[0].from_id, "a");
        assert_eq!(report.unresolved_dependencies[0].to_id, "missing");

        assert_eq!(report.dependency_cycles.len(), 1);
        assert_eq!(
            report.dependency_cycles[0].ids,
            vec!["a".to_owned(), "b".to_owned()]
        );
    }

    #[test]
    fn passes_for_valid_graph() {
        let entries = vec![
            entry("a", &[], "docs/a.md"),
            entry("b", &["a"], "docs/b.md"),
            entry("c", &["b"], "docs/c.md"),
        ];

        validate_entries(&entries).expect("validation must pass");
    }
}
