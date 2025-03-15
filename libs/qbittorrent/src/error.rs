use reqwest::StatusCode;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API Returned bad response: {explain}")]
    BadResponse { explain: &'static str },

    #[error("API returned unknown status code: {0}")]
    UnknownHttpCode(StatusCode),

    #[error("Non ASCII header")]
    NonAsciiHeader,

    #[error(transparent)]
    Api(#[from] ApiError),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

/// Errors defined and returned by the API
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User's IP is banned for too many failed login attempts")]
    IpBanned,

    #[error("API routes requires login, try again")]
    NotLoggedIn,

    #[error("Torrent not found")]
    TorrentNotFound,

    #[error("Torrent name is empty")]
    TorrentNameEmpty,

    #[error("`newUrl` is not a valid URL")]
    InvalidTrackerUrl,

    #[error("`newUrl` already exists for the torrent or `origUrl` was not found")]
    ConflictTrackerUrl,

    #[error("None of the given peers are valid")]
    InvalidPeers,

    #[error("Torrent queueing is not enabled")]
    QueueingDisabled,

    #[error("Torrent metadata hasn't downloaded yet or at least one file id was not found")]
    MetaNotDownloadedOrIdNotFound,

    #[error("Save path is empty")]
    SavePathEmpty,

    #[error("User does not have write access to the directory")]
    NoWriteAccess,

    #[error("Unable to create save path directory")]
    UnableToCreateDir,

    #[error("Category name does not exist")]
    CategoryNotFound,

    #[error("Category editing failed")]
    CategoryEditingFailed,

    #[error("Invalid `newPath` or `oldPath`, or `newPath` already in use")]
    InvalidPath,
}
