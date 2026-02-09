use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Not a git repository")]
    NotGitRepository,

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Base branch not found. Please configure git-branch-delete.base in .gitconfig")]
    BaseBranchNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
