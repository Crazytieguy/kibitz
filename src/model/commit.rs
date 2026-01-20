/// Information about a single commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Short hash (7 chars)
    pub oid: String,
    /// Full hash for git operations
    pub oid_full: String,
    /// First line of commit message
    pub message: String,
}
