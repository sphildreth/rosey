use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JournalOp {
    MoveStarted,
    CopyStarted,
    CopyVerified,
    SourceDeleted,
    RenameCompleted,
    DestinationCreated,
    Completed,
    Failed,
    RolledBack,
    Skipped,
    Replaced,
    KeptBoth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub op: JournalOp,
    pub src: Utf8PathBuf,
    pub dst: Utf8PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub timestamp: u64,
}

impl JournalEntry {
    pub fn now(op: JournalOp, src: &Utf8Path, dst: &Utf8Path) -> Self {
        Self {
            op,
            src: src.to_path_buf(),
            dst: dst.to_path_buf(),
            bytes: None,
            error: None,
            timestamp: now_secs(),
        }
    }

    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes = Some(bytes);
        self
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[derive(Debug)]
pub struct OperationJournal {
    writer: Mutex<Option<File>>,
    path: Utf8PathBuf,
}

impl OperationJournal {
    pub fn open(path: &Utf8Path) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().append(true).create(true).open(path.as_std_path())?;

        Ok(Self { writer: Mutex::new(Some(file)), path: path.to_path_buf() })
    }

    pub fn create_temp(prefix: &str) -> Result<Self, std::io::Error> {
        let dir = std::env::temp_dir();
        let path = Utf8PathBuf::from_path_buf(dir).unwrap_or_else(|_| Utf8PathBuf::from(""));
        let timestamp = now_secs();
        let filename = format!("{prefix}_{timestamp}.journal");
        let full_path = path.join(filename);

        let file =
            OpenOptions::new().write(true).create_new(true).open(full_path.as_std_path())?;

        Ok(Self { writer: Mutex::new(Some(file)), path: full_path })
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    pub fn record(&self, entry: &JournalEntry) {
        if let Ok(mut guard) = self.writer.lock() {
            if let Some(ref mut file) = *guard {
                let line = serde_json::to_string(entry).unwrap_or_default();
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
        }
    }

    pub fn record_op(&self, op: JournalOp, src: &Utf8Path, dst: &Utf8Path) {
        self.record(&JournalEntry::now(op, src, dst));
    }

    pub fn record_error(&self, op: JournalOp, src: &Utf8Path, dst: &Utf8Path, error: &str) {
        self.record(&JournalEntry::now(op, src, dst).with_error(error));
    }

    pub fn close(&self) {
        if let Ok(mut guard) = self.writer.lock() {
            *guard = None;
        }
    }

    pub fn read_entries(path: &Utf8Path) -> Result<Vec<JournalEntry>, std::io::Error> {
        let file = File::open(path.as_std_path())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<JournalEntry>(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    pub fn find_incomplete_transfers(
        entries: &[JournalEntry],
    ) -> Vec<(&JournalEntry, &JournalEntry)> {
        let mut incomplete = Vec::new();
        let mut started: std::collections::HashMap<String, &JournalEntry> =
            std::collections::HashMap::new();

        for entry in entries {
            let src = entry.src.as_str().to_string();
            match entry.op {
                JournalOp::MoveStarted | JournalOp::CopyStarted => {
                    started.insert(src.clone(), entry);
                }
                JournalOp::Completed | JournalOp::Failed | JournalOp::RolledBack => {
                    started.remove(&src);
                }
                _ => {}
            }
        }

        for (_, start_entry) in started {
            incomplete.push((start_entry, start_entry));
        }

        incomplete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn journal_writes_and_reads_entries() {
        let dir = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let journal_path = path.join("test.journal");

        let journal = OperationJournal::open(&journal_path).unwrap();

        let src = Utf8Path::new("/src/myfile.mkv");
        let dst = Utf8Path::new("/dst/myfile.mkv");

        journal.record_op(JournalOp::MoveStarted, src, dst);
        journal.record_op(JournalOp::Completed, src, dst);
        journal.close();

        let entries = OperationJournal::read_entries(&journal_path).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].op, JournalOp::MoveStarted);
        assert_eq!(entries[1].op, JournalOp::Completed);
    }

    #[test]
    fn journal_records_errors() {
        let dir = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let journal_path = path.join("test_errors.journal");

        let journal = OperationJournal::open(&journal_path).unwrap();

        let src = Utf8Path::new("/src/broken.mkv");
        let dst = Utf8Path::new("/dst/broken.mkv");

        journal.record_error(JournalOp::Failed, src, dst, "permission denied");
        journal.close();

        let entries = OperationJournal::read_entries(&journal_path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].op, JournalOp::Failed);
        assert_eq!(entries[0].error.as_deref(), Some("permission denied"));
    }

    #[test]
    fn journal_detects_incomplete_transfers() {
        let entries = vec![
            JournalEntry::now(JournalOp::MoveStarted, Utf8Path::new("/src/a.mkv"), Utf8Path::new("/dst/a.mkv")),
            JournalEntry::now(JournalOp::Completed, Utf8Path::new("/src/a.mkv"), Utf8Path::new("/dst/a.mkv")),
            JournalEntry::now(JournalOp::MoveStarted, Utf8Path::new("/src/b.mkv"), Utf8Path::new("/dst/b.mkv")),
            JournalEntry::now(JournalOp::CopyStarted, Utf8Path::new("/src/c.mkv"), Utf8Path::new("/dst/c.mkv")),
        ];

        let incomplete = OperationJournal::find_incomplete_transfers(&entries);
        assert_eq!(incomplete.len(), 2);
    }

    #[test]
    fn journal_records_with_bytes() {
        let dir = TempDir::new().unwrap();
        let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).unwrap();
        let journal_path = path.join("test_bytes.journal");

        let journal = OperationJournal::open(&journal_path).unwrap();

        let src = Utf8Path::new("/src/large.mkv");
        let dst = Utf8Path::new("/dst/large.mkv");

        let entry = JournalEntry::now(JournalOp::CopyStarted, src, dst).with_bytes(42_000_000);
        journal.record(&entry);
        journal.close();

        let entries = OperationJournal::read_entries(&journal_path).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].bytes, Some(42_000_000));
    }

    #[test]
    fn journal_create_temp_works() {
        let journal = OperationJournal::create_temp("rosey_test").unwrap();
        assert!(journal.path().to_string().contains("rosey_test"));
        journal.close();

        let _ = std::fs::remove_file(journal.path().as_std_path());
    }
}
