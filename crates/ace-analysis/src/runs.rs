/// Summary of repeated-byte runs found in one block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunStats {
    /// Number of runs with length at least four.
    pub run_count: usize,
    /// Total bytes covered by useful runs.
    pub repeated_bytes: usize,
    /// Longest useful run.
    pub longest_run: usize,
}

/// Finds repeated-byte runs of length at least four.
pub fn analyze_runs(input: &[u8]) -> RunStats {
    if input.is_empty() {
        return RunStats {
            run_count: 0,
            repeated_bytes: 0,
            longest_run: 0,
        };
    }
    let mut i = 0usize;
    let mut run_count = 0usize;
    let mut repeated_bytes = 0usize;
    let mut longest_run = 0usize;
    while i < input.len() {
        let mut j = i + 1;
        while j < input.len() && input[j] == input[i] {
            j += 1;
        }
        let len = j - i;
        if len >= 4 {
            run_count += 1;
            repeated_bytes += len;
            longest_run = longest_run.max(len);
        }
        i = j;
    }
    RunStats {
        run_count,
        repeated_bytes,
        longest_run,
    }
}
