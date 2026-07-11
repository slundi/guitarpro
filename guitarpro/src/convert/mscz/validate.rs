//! Loss report emitted by [`super::mscx_to_loaded_score`].
//!
//! The report tallies MSCX features that the current converter recognizes
//! but does not (yet) preserve in the [`LoadedScore`](crate::model::optimized::LoadedScore).
//! It is deliberately shallow — a name + count — so tests can assert that a
//! specific tag was surfaced without pinning line numbers.

use std::collections::BTreeMap;

/// Summary of MSCX features the converter observed but did not represent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LossReport {
    /// Element name → occurrence count. Only elements the converter
    /// explicitly knows about but chose not to translate are counted here;
    /// truly unknown XML is not enumerated (the raw XML preserves it).
    counts: BTreeMap<String, u32>,
}

impl LossReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one occurrence of a missing feature.
    pub fn note(&mut self, name: impl Into<String>) {
        let entry = self.counts.entry(name.into()).or_default();
        *entry = entry.saturating_add(1);
    }

    /// Number of distinct feature names recorded.
    pub fn distinct(&self) -> usize {
        self.counts.len()
    }

    /// Total occurrences across all feature names.
    pub fn total(&self) -> u32 {
        self.counts.values().sum()
    }

    /// Iterate over recorded (name, count) pairs, sorted by name.
    pub fn iter(&self) -> impl Iterator<Item = (&str, u32)> {
        self.counts
            .iter()
            .map(|(name, count)| (name.as_str(), *count))
    }

    /// Return the count recorded for `name`, or 0.
    pub fn get(&self, name: &str) -> u32 {
        self.counts.get(name).copied().unwrap_or(0)
    }

    /// `true` when no features are recorded.
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_iterate_in_sorted_order() {
        let mut report = LossReport::new();
        report.note("Slur");
        report.note("Dynamic");
        report.note("Slur");
        report.note("Dynamic");
        report.note("Dynamic");
        report.note("Tuplet");

        assert_eq!(report.distinct(), 3);
        assert_eq!(report.total(), 6);
        assert_eq!(report.get("Slur"), 2);
        assert_eq!(report.get("Dynamic"), 3);
        assert_eq!(report.get("Tuplet"), 1);
        assert_eq!(report.get("Nonexistent"), 0);

        let entries: Vec<(&str, u32)> = report.iter().collect();
        assert_eq!(
            entries,
            vec![("Dynamic", 3), ("Slur", 2), ("Tuplet", 1)],
            "iter must be sorted by name"
        );
    }

    #[test]
    fn empty_report_reports_zero() {
        let report = LossReport::new();
        assert!(report.is_empty());
        assert_eq!(report.distinct(), 0);
        assert_eq!(report.total(), 0);
    }
}
