use std::path::{Path, PathBuf};

use sysinfo::Disks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskSpaceSnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct DiskSpaceCollector {
    observed_path: PathBuf,
}

impl DiskSpaceCollector {
    pub fn for_current_dir() -> std::io::Result<Self> {
        std::env::current_dir().map(Self::new)
    }

    pub fn snapshot(&self, disks: &Disks) -> Option<DiskSpaceSnapshot> {
        let candidates = disks.list().iter().map(DiskSpaceCandidate::from_disk).collect::<Vec<_>>();
        select_disk_space(&self.observed_path, &candidates)
    }

    fn new(observed_path: PathBuf) -> Self {
        Self { observed_path }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiskSpaceCandidate {
    mount_point: PathBuf,
    total_bytes: u64,
    available_bytes: u64,
}

impl DiskSpaceCandidate {
    fn from_disk(disk: &sysinfo::Disk) -> Self {
        Self {
            mount_point: disk.mount_point().to_path_buf(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        }
    }

    fn mount_path_len(&self) -> usize {
        self.mount_point.as_os_str().to_string_lossy().len()
    }

    fn snapshot(&self) -> DiskSpaceSnapshot {
        DiskSpaceSnapshot {
            total_bytes: self.total_bytes,
            available_bytes: self.available_bytes,
        }
    }
}

fn select_disk_space(observed_path: &Path, candidates: &[DiskSpaceCandidate]) -> Option<DiskSpaceSnapshot> {
    candidates
        .iter()
        .filter(|candidate| !candidate.mount_point.as_os_str().is_empty())
        .filter(|candidate| observed_path.starts_with(&candidate.mount_point))
        .max_by_key(|candidate| candidate.mount_path_len())
        .map(DiskSpaceCandidate::snapshot)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{DiskSpaceCandidate, DiskSpaceSnapshot, select_disk_space};

    #[test]
    fn selects_longest_matching_mount_point() {
        let candidates = vec![candidate("/", 100, 10), candidate("/var", 200, 20), candidate("/var/lib/hook", 300, 30)];

        let selected = select_disk_space(&PathBuf::from("/var/lib/hook/data"), &candidates);

        assert_eq!(
            selected,
            Some(DiskSpaceSnapshot {
                total_bytes: 300,
                available_bytes: 30
            })
        );
    }

    #[test]
    fn returns_none_without_matching_mount_point() {
        let candidates = vec![candidate("/mnt/app", 100, 10)];

        let selected = select_disk_space(&PathBuf::from("/srv/hook"), &candidates);

        assert_eq!(selected, None);
    }

    fn candidate(mount_point: &str, total_bytes: u64, available_bytes: u64) -> DiskSpaceCandidate {
        DiskSpaceCandidate {
            mount_point: PathBuf::from(mount_point),
            total_bytes,
            available_bytes,
        }
    }
}
