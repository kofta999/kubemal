use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Anime not found {0}")]
    AnimeNotFound(String),
    #[error("Watch record' name attribute not found")]
    WatchRecordNameNotFound,
    #[error("Couldn't update WatchRecord status")]
    StatusUpdate,
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Total Episodes {0} is less or equal than zero {1}")]
    EpisodeCountTooLow(i32, i32),
    #[error("Total Episodes {0} is more than anime episode count {1}")]
    EpisodeCountTooHigh(i32, i32),
    #[error("spec.status field is set even the anime's completed (watched all episodes)")]
    StatusOverride,
    #[error("Watch record is not found")]
    WatchRecordNotFound,
    #[error("Anime associated with WatchRecord is not found")]
    AnimeNotFound,
}

#[derive(Error, Debug)]
pub enum MutationError {
    #[error("missing object name")]
    MissingName,
    #[error("failed to apply patch: {0}")]
    ApplyPatch(String),
    #[error("Anime associated with WatchRecord is not found")]
    AnimeNotFound,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Kubernetes client error: {0}")]
    Kube(#[from] kube::Error),
}
