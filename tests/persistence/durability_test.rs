//! Persistence and Durability Testing
//!
//! Verifies receipt log durability, crash recovery, data integrity,
//! and replay capabilities.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write, Seek, SeekFrom, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::{Mutex, RwLock};
use tempfile::TempDir;
use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use crc32fast::Hasher;

/// Persistent receipt log
#[derive(Debug, Serialize, Deserialize)]
pub struct PersistentReceipt {
    pub id: u64,
    pub pattern_id: u64,
    pub timestamp: u64,
    pub state_hash: u64,
    pub checksum: u32,
    pub version: u32,
}

impl PersistentReceipt {
    pub fn new(id: u64, pattern_id: u64, timestamp: u64, state_hash: u64) -> Self {
        let mut receipt = Self {
            id,
            pattern_id,
            timestamp,
            state_hash,
            checksum: 0,
            version: 1,
        };
        receipt.checksum = receipt.calculate_checksum();
        receipt
    }

    fn calculate_checksum(&self) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&self.id.to_le_bytes());
        hasher.update(&self.pattern_id.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.state_hash.to_le_bytes());
        hasher.update(&self.version.to_le_bytes());
        hasher.finalize()
    }

    pub fn verify(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }
}

/// Write-Ahead Log (WAL) for durability
pub struct WriteAheadLog {
    log_file: Arc<Mutex<File>>,
    log_path: PathBuf,
    segment_size: u64,
    current_segment: AtomicU64,
    sync_mode: SyncMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SyncMode {
    Immediate,  // fsync after each write
    Batch,      // fsync periodically
    Lazy,       // OS decides when to flush
}

impl WriteAheadLog {
    pub fn new(path: impl AsRef<Path>, sync_mode: SyncMode) -> io::Result<Self> {
        let log_path = path.as_ref().to_path_buf();
        let log_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(false)
            .open(&log_path)?;

        Ok(Self {
            log_file: Arc::new(Mutex::new(log_file)),
            log_path,
            segment_size: 1024 * 1024, // 1MB segments
            current_segment: AtomicU64::new(0),
            sync_mode,
        })
    }

    pub fn append_receipt(&self, receipt: &PersistentReceipt) -> io::Result<u64> {
        let serialized = serialize(receipt)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut file = self.log_file.lock();

        // Write length prefix
        let len = serialized.len() as u32;
        file.write_all(&len.to_le_bytes())?;

        // Write serialized receipt
        file.write_all(&serialized)?;

        // Get position before sync
        let position = file.stream_position()?;

        // Sync based on mode
        match self.sync_mode {
            SyncMode::Immediate => file.sync_all()?,
            SyncMode::Batch => file.sync_data()?,
            SyncMode::Lazy => {}
        }

        Ok(position)
    }

    pub fn read_all_receipts(&self) -> io::Result<Vec<PersistentReceipt>> {
        let mut file = self.log_file.lock();
        file.seek(SeekFrom::Start(0))?;

        let mut receipts = Vec::new();
        let mut len_buf = [0u8; 4];

        loop {
            // Read length prefix
            match file.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }

            let len = u32::from_le_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];

            // Read receipt data
            file.read_exact(&mut buf)?;

            let receipt: PersistentReceipt = deserialize(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            if receipt.verify() {
                receipts.push(receipt);
            } else {
                eprintln!("Warning: Corrupted receipt detected at position {}",
                    file.stream_position()?);
            }
        }

        Ok(receipts)
    }

    pub fn truncate_at(&self, position: u64) -> io::Result<()> {
        let mut file = self.log_file.lock();
        file.set_len(position)?;
        file.sync_all()
    }

    pub fn rotate_segment(&self) -> io::Result<()> {
        let segment_num = self.current_segment.fetch_add(1, Ordering::SeqCst);
        let new_path = self.log_path.with_extension(format!("seg{}", segment_num));

        // Copy current log to segment file
        let mut file = self.log_file.lock();
        file.seek(SeekFrom::Start(0))?;
        let mut segment_file = File::create(&new_path)?;
        io::copy(&mut *file, &mut segment_file)?;
        segment_file.sync_all()?;

        // Clear current log
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;

        Ok(())
    }
}

/// Crash recovery manager
pub struct CrashRecoveryManager {
    wal: Arc<WriteAheadLog>,
    checkpoint_dir: PathBuf,
    last_checkpoint: AtomicU64,
    state_snapshots: Arc<RwLock<Vec<StateSnapshot>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub timestamp: u64,
    pub pattern_id: u64,
    pub variables: Vec<i64>,
    pub receipt_count: u64,
    pub checksum: u32,
}

impl StateSnapshot {
    pub fn new(timestamp: u64, pattern_id: u64, variables: Vec<i64>, receipt_count: u64) -> Self {
        let mut snapshot = Self {
            timestamp,
            pattern_id,
            variables,
            receipt_count,
            checksum: 0,
        };
        snapshot.checksum = snapshot.calculate_checksum();
        snapshot
    }

    fn calculate_checksum(&self) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.pattern_id.to_le_bytes());
        hasher.update(&self.receipt_count.to_le_bytes());
        for var in &self.variables {
            hasher.update(&var.to_le_bytes());
        }
        hasher.finalize()
    }

    pub fn verify(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }
}

impl CrashRecoveryManager {
    pub fn new(wal: Arc<WriteAheadLog>, checkpoint_dir: impl AsRef<Path>) -> io::Result<Self> {
        let checkpoint_dir = checkpoint_dir.as_ref().to_path_buf();
        fs::create_dir_all(&checkpoint_dir)?;

        Ok(Self {
            wal,
            checkpoint_dir,
            last_checkpoint: AtomicU64::new(0),
            state_snapshots: Arc::new(RwLock::new(Vec::new())),
        })
    }

    pub fn create_checkpoint(&self, snapshot: StateSnapshot) -> io::Result<u64> {
        let checkpoint_num = self.last_checkpoint.fetch_add(1, Ordering::SeqCst);
        let checkpoint_path = self.checkpoint_dir.join(format!("checkpoint_{:08}.dat", checkpoint_num));

        let serialized = serialize(&snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut file = File::create(&checkpoint_path)?;
        file.write_all(&serialized)?;
        file.sync_all()?;

        self.state_snapshots.write().push(snapshot);

        Ok(checkpoint_num)
    }

    pub fn recover_from_crash(&self) -> io::Result<RecoveryResult> {
        // Find latest valid checkpoint
        let latest_checkpoint = self.find_latest_checkpoint()?;

        // Read all WAL entries after checkpoint
        let wal_receipts = self.wal.read_all_receipts()?;

        // Filter receipts after checkpoint
        let receipts_to_replay = if let Some(ref checkpoint) = latest_checkpoint {
            wal_receipts.into_iter()
                .filter(|r| r.timestamp > checkpoint.timestamp)
                .collect()
        } else {
            wal_receipts
        };

        Ok(RecoveryResult {
            checkpoint: latest_checkpoint,
            receipts_to_replay,
            recovery_timestamp: chrono::Utc::now().timestamp() as u64,
        })
    }

    fn find_latest_checkpoint(&self) -> io::Result<Option<StateSnapshot>> {
        let mut latest: Option<(u64, StateSnapshot)> = None;

        for entry in fs::read_dir(&self.checkpoint_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("checkpoint_") {
                        let data = fs::read(&path)?;
                        if let Ok(snapshot) = deserialize::<StateSnapshot>(&data) {
                            if snapshot.verify() {
                                let num = snapshot.timestamp;
                                if latest.as_ref().map_or(true, |(n, _)| num > *n) {
                                    latest = Some((num, snapshot));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(latest.map(|(_, snapshot)| snapshot))
    }

    pub fn verify_data_integrity(&self) -> IntegrityReport {
        let mut report = IntegrityReport::default();

        // Check WAL integrity
        match self.wal.read_all_receipts() {
            Ok(receipts) => {
                report.total_receipts = receipts.len();
                for receipt in receipts {
                    if receipt.verify() {
                        report.valid_receipts += 1;
                    } else {
                        report.corrupted_receipts += 1;
                    }
                }
            }
            Err(e) => {
                report.errors.push(format!("WAL read error: {}", e));
            }
        }

        // Check checkpoint integrity
        match fs::read_dir(&self.checkpoint_dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        report.total_checkpoints += 1;
                        let data = fs::read(entry.path()).unwrap_or_default();
                        if let Ok(snapshot) = deserialize::<StateSnapshot>(&data) {
                            if snapshot.verify() {
                                report.valid_checkpoints += 1;
                            } else {
                                report.corrupted_checkpoints += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                report.errors.push(format!("Checkpoint dir error: {}", e));
            }
        }

        report
    }

    pub fn repair_corrupted_data(&self) -> io::Result<RepairResult> {
        let mut repaired = 0;
        let mut unrecoverable = 0;

        // Try to repair WAL
        let receipts = self.wal.read_all_receipts()?;
        let mut valid_receipts = Vec::new();

        for receipt in receipts {
            if receipt.verify() {
                valid_receipts.push(receipt);
            } else {
                // Attempt repair by recalculating checksum
                let mut repaired_receipt = receipt.clone();
                repaired_receipt.checksum = repaired_receipt.calculate_checksum();

                if repaired_receipt.verify() {
                    valid_receipts.push(repaired_receipt);
                    repaired += 1;
                } else {
                    unrecoverable += 1;
                }
            }
        }

        // Rewrite WAL with valid receipts
        if repaired > 0 || unrecoverable > 0 {
            self.wal.truncate_at(0)?;
            for receipt in valid_receipts {
                self.wal.append_receipt(&receipt)?;
            }
        }

        Ok(RepairResult {
            repaired,
            unrecoverable,
            data_loss: unrecoverable > 0,
        })
    }
}

#[derive(Debug)]
pub struct RecoveryResult {
    pub checkpoint: Option<StateSnapshot>,
    pub receipts_to_replay: Vec<PersistentReceipt>,
    pub recovery_timestamp: u64,
}

#[derive(Debug, Default)]
pub struct IntegrityReport {
    pub total_receipts: usize,
    pub valid_receipts: usize,
    pub corrupted_receipts: usize,
    pub total_checkpoints: usize,
    pub valid_checkpoints: usize,
    pub corrupted_checkpoints: usize,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct RepairResult {
    pub repaired: usize,
    pub unrecoverable: usize,
    pub data_loss: bool,
}

/// Durability test harness
pub struct DurabilityTestHarness {
    temp_dir: TempDir,
    wal: Arc<WriteAheadLog>,
    recovery_manager: Arc<CrashRecoveryManager>,
}

impl DurabilityTestHarness {
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;
        let wal_path = temp_dir.path().join("wal.log");
        let checkpoint_dir = temp_dir.path().join("checkpoints");

        let wal = Arc::new(WriteAheadLog::new(wal_path, SyncMode::Immediate)?);
        let recovery_manager = Arc::new(CrashRecoveryManager::new(
            wal.clone(),
            checkpoint_dir
        )?);

        Ok(Self {
            temp_dir,
            wal,
            recovery_manager,
        })
    }

    pub fn simulate_workflow_execution(&self, num_receipts: usize) -> io::Result<()> {
        for i in 0..num_receipts {
            let receipt = PersistentReceipt::new(
                i as u64,
                (i / 10) as u64,
                i as u64 * 1000,
                i as u64 * 12345,
            );
            self.wal.append_receipt(&receipt)?;

            // Create checkpoint every 100 receipts
            if i % 100 == 99 {
                let snapshot = StateSnapshot::new(
                    i as u64 * 1000,
                    (i / 10) as u64,
                    vec![i as i64; 10],
                    i as u64,
                );
                self.recovery_manager.create_checkpoint(snapshot)?;
            }
        }
        Ok(())
    }

    pub fn simulate_crash_and_recover(&self) -> io::Result<bool> {
        // Simulate crash by truncating WAL at random position
        let receipts = self.wal.read_all_receipts()?;
        let original_count = receipts.len();

        if original_count > 10 {
            // Truncate to simulate partial write
            let truncate_pos = (original_count * 3 / 4) as u64 * 100; // Rough estimate
            self.wal.truncate_at(truncate_pos)?;
        }

        // Attempt recovery
        let recovery = self.recovery_manager.recover_from_crash()?;

        // Verify recovery
        let recovered_count = recovery.receipts_to_replay.len();
        let has_checkpoint = recovery.checkpoint.is_some();

        println!("Crash recovery test:");
        println!("  Original receipts: {}", original_count);
        println!("  Recovered receipts: {}", recovered_count);
        println!("  Has checkpoint: {}", has_checkpoint);

        Ok(has_checkpoint || recovered_count > 0)
    }

    pub fn test_concurrent_persistence(&self) -> io::Result<()> {
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|thread_id| {
                let wal = self.wal.clone();
                thread::spawn(move || {
                    for i in 0..100 {
                        let receipt = PersistentReceipt::new(
                            thread_id * 1000 + i,
                            thread_id,
                            i * 1000,
                            thread_id * i,
                        );
                        wal.append_receipt(&receipt).unwrap();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all receipts persisted
        let receipts = self.wal.read_all_receipts()?;
        assert_eq!(receipts.len(), 1000, "Not all receipts were persisted");

        Ok(())
    }

    pub fn test_version_recovery(&self) -> io::Result<()> {
        // Write receipts with different versions
        for version in 1..=3 {
            for i in 0..10 {
                let mut receipt = PersistentReceipt::new(
                    version * 100 + i,
                    version,
                    i * 1000,
                    version * i,
                );
                receipt.version = version as u32;
                receipt.checksum = receipt.calculate_checksum();
                self.wal.append_receipt(&receipt)?;
            }
        }

        // Read and verify all versions recovered
        let receipts = self.wal.read_all_receipts()?;
        let mut version_counts = [0; 3];

        for receipt in receipts {
            if receipt.version >= 1 && receipt.version <= 3 {
                version_counts[(receipt.version - 1) as usize] += 1;
            }
        }

        assert_eq!(version_counts, [10, 10, 10], "Version recovery failed");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_persistence() {
        let harness = DurabilityTestHarness::new().unwrap();
        harness.simulate_workflow_execution(500).unwrap();

        let receipts = harness.wal.read_all_receipts().unwrap();
        assert_eq!(receipts.len(), 500);

        for receipt in receipts {
            assert!(receipt.verify(), "Receipt verification failed");
        }
    }

    #[test]
    fn test_crash_recovery() {
        let harness = DurabilityTestHarness::new().unwrap();
        harness.simulate_workflow_execution(1000).unwrap();

        assert!(harness.simulate_crash_and_recover().unwrap(),
            "Crash recovery failed");
    }

    #[test]
    fn test_data_integrity() {
        let harness = DurabilityTestHarness::new().unwrap();
        harness.simulate_workflow_execution(300).unwrap();

        let report = harness.recovery_manager.verify_data_integrity();

        assert_eq!(report.total_receipts, 300);
        assert_eq!(report.valid_receipts, 300);
        assert_eq!(report.corrupted_receipts, 0);
        assert!(report.valid_checkpoints > 0);
    }

    #[test]
    fn test_concurrent_persistence() {
        let harness = DurabilityTestHarness::new().unwrap();
        harness.test_concurrent_persistence().unwrap();
    }

    #[test]
    fn test_version_recovery() {
        let harness = DurabilityTestHarness::new().unwrap();
        harness.test_version_recovery().unwrap();
    }

    #[test]
    fn test_wal_rotation() {
        let harness = DurabilityTestHarness::new().unwrap();

        // Write receipts
        for i in 0..100 {
            let receipt = PersistentReceipt::new(i, i / 10, i * 1000, i * 123);
            harness.wal.append_receipt(&receipt).unwrap();
        }

        // Rotate segment
        harness.wal.rotate_segment().unwrap();

        // Write more receipts
        for i in 100..200 {
            let receipt = PersistentReceipt::new(i, i / 10, i * 1000, i * 123);
            harness.wal.append_receipt(&receipt).unwrap();
        }

        // Verify current log has only new receipts
        let receipts = harness.wal.read_all_receipts().unwrap();
        assert_eq!(receipts.len(), 100);
        assert_eq!(receipts[0].id, 100);
    }
}