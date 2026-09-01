use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::Error;
use crate::Event;

/// Read append-only JSONL in file order. Empty lines are skipped.
pub fn read_events(path: &Path) -> Result<Vec<Event>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        let event = serde_json::from_str(&line).map_err(|source| Error::InvalidLine {
            line: idx + 1,
            source,
        })?;
        events.push(event);
    }
    Ok(events)
}
