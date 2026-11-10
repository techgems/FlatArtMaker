//! One cached result per stage, keyed by a hash of everything it depends on.
//!
//! Each stage's key includes its upstream stage's key, so any change upstream
//! changes every key below it: downstream results go stale without being
//! cleared explicitly.

use flatart_domain::{Error, Result, Stage};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Hashes a stage's inputs. Stable within a process, which is all a cache needs.
pub(crate) fn key_of(inputs: impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    inputs.hash(&mut hasher);
    hasher.finish()
}

pub(crate) struct Cache<T> {
    stage: Stage,
    entry: Option<(u64, T)>,
}

impl<T> Cache<T> {
    pub(crate) fn new(stage: Stage) -> Self {
        Self { stage, entry: None }
    }

    /// The result for `key`. Only fails if called before [`Cache::refresh`]
    /// with the same key, which would be a pipeline bug.
    pub(crate) fn get(&self, key: u64) -> Result<&T> {
        match &self.entry {
            Some((k, value)) if *k == key => Ok(value),
            _ => Err(Error::StageFailed {
                stage: self.stage,
                message: "internal error: result read before it was computed".into(),
            }),
        }
    }

    /// Runs `compute` unless the result for `key` is already cached.
    pub(crate) fn refresh(&mut self, key: u64, compute: impl FnOnce() -> Result<T>) -> Result<u64> {
        if !matches!(&self.entry, Some((k, _)) if *k == key) {
            self.entry = Some((key, compute()?));
        }
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_once_per_key() {
        let mut cache = Cache::new(Stage::Load);
        let mut runs = 0;
        for key in [1, 1, 2, 2, 1] {
            cache
                .refresh(key, || {
                    runs += 1;
                    Ok(key * 10)
                })
                .unwrap();
            assert_eq!(*cache.get(key).unwrap(), key * 10);
        }
        assert_eq!(runs, 3);
        assert!(cache.get(2).is_err());
    }

    #[test]
    fn a_failed_compute_leaves_nothing_cached() {
        let mut cache: Cache<u64> = Cache::new(Stage::Load);
        let failed = cache.refresh(1, || Err(Error::InvalidInput("no".into())));
        assert!(failed.is_err());
        assert!(cache.get(1).is_err());
    }
}
