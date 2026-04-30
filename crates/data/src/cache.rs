//! SHA-256 keyed disk cache using bincode serialization.

use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const CACHE_VERSION: &str = "2rs";

/// Process-wide monotonic counter for temp-file naming. Guarantees two
/// concurrent `set()` calls with the same key use different temp paths.
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

pub struct DiskCache {
    root: PathBuf,
}

impl DiskCache {
    pub fn new(root: PathBuf) -> Self {
        // Ensure cache dir exists and check version
        let namespace_dir = root.join("binance_rs");
        fs::create_dir_all(&namespace_dir).ok();

        let version_file = root.join(".cache_version_rs");
        let current = fs::read_to_string(&version_file).unwrap_or_default();
        if current.trim() != CACHE_VERSION {
            // Wipe stale cache on version mismatch
            if namespace_dir.exists() {
                for entry in fs::read_dir(&namespace_dir).into_iter().flatten() {
                    if let Ok(entry) = entry {
                        fs::remove_file(entry.path()).ok();
                    }
                }
            }
            fs::write(&version_file, CACHE_VERSION).ok();
        }

        Self { root }
    }

    /// Get a cached value by key parts.
    pub fn get<T: DeserializeOwned>(&self, key_parts: &[&str]) -> Option<T> {
        let path = self.cache_path(key_parts);
        let bytes = fs::read(&path).ok()?;
        bincode::deserialize(&bytes).ok()
    }

    /// Store a value in the cache.
    pub fn set<T: Serialize>(&self, key_parts: &[&str], value: &T) {
        let path = self.cache_path(key_parts);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(bytes) = bincode::serialize(value) {
            let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
            let tmp = path.with_extension(format!("{}.{seq}.tmp", std::process::id()));
            if fs::write(&tmp, &bytes).is_ok() {
                fs::rename(&tmp, &path).ok();
            }
        }
    }

    fn cache_path(&self, key_parts: &[&str]) -> PathBuf {
        let key_str = key_parts.join("::");
        let mut hasher = Sha256::new();
        hasher.update(key_str.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        self.root.join("binance_rs").join(format!("{hash}.bin"))
    }
}

/// Default cache root directory.
pub fn default_cache_root() -> PathBuf {
    dirs_next().join(".claude_trader").join("cache")
}

fn dirs_next() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "ct_disk_cache_test_{}_{nanos}",
            std::process::id()
        ))
    }

    /// Starved stress test — 16 threads hammer the same cache key 64 times
    /// each, all released simultaneously via a Barrier to maximise
    /// temp-file race. Before the fix, the shared `{pid}.tmp` temp name
    /// let one thread's `fs::write` overwrite another's in-flight bytes,
    /// occasionally leaving a torn file that failed to deserialise on
    /// read. With the atomic-seq fix every write uses a unique temp path
    /// and the final cached value is always one of the written integers.
    #[test]
    fn concurrent_writes_to_same_key_never_tear() {
        let root = unique_root();
        let cache = Arc::new(DiskCache::new(root.clone()));
        let key_parts: &[&str] = &["stress", "same-key"];

        const THREADS: usize = 16;
        const ITERS: usize = 64;

        let barrier = Arc::new(Barrier::new(THREADS));
        let mut handles = Vec::with_capacity(THREADS);
        for t in 0..THREADS {
            let cache = Arc::clone(&cache);
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                for i in 0..ITERS {
                    let v: u64 = (t as u64) * 1_000_000 + i as u64;
                    cache.set(key_parts, &v);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        // Final read must succeed and must match *some* value any writer
        // would have produced. A torn file would fail to deserialise.
        let got: Option<u64> = cache.get(key_parts);
        let got = got.expect("final cache read failed — torn file suspected");
        let max_value = (THREADS as u64 - 1) * 1_000_000 + (ITERS as u64 - 1);
        assert!(got <= max_value, "unexpected value {got} (> {max_value})");

        // Cleanup best-effort.
        let _ = fs::remove_dir_all(&root);
    }
}
