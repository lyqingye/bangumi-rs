use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Pan115Error {
    #[error("invalid path, path is not a directory or path is not absolute: {0}")]
    InvalidPath(String),

    #[error("invalid url: {0}")]
    InvalidUrl(String),

    #[error("cookie parse failed: {0}")]
    CookieParseFailed(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("download failed")]
    DownloadFailed,

    #[error("unsupport download directory")]
    UnsupportDownloadDirectory,

    #[error("user not login")]
    NotLogin,

    #[error("offline download quota has been used up, you can purchase a VIP experience or upgrade to VIP service to get more quota")]
    OfflineNoTimes,

    #[error("invalid download link")]
    OfflineInvalidLink,

    #[error("offline task existed")]
    OfflineTaskExisted,

    #[error("file order not supported")]
    OrderNotSupport,

    #[error("password incorrect")]
    PasswordIncorrect,

    #[error("requires two-step verification")]
    LoginTwoStepVerify,

    #[error("account not binds mobile")]
    AccountNotBindMobile,

    #[error("credential invalid")]
    CredentialInvalid,

    #[error("session exited")]
    SessionExited,

    #[error("qrcode expired")]
    QrcodeExpired,

    #[error("unexpected error")]
    Unexpected,

    #[error("target already exists")]
    Exist,

    #[error("target does not exist")]
    NotExist,

    #[error("invalid cursor")]
    InvalidCursor,

    #[error("upload reach the limit")]
    UploadTooLarge,

    #[error("upload failed")]
    UploadFailed,

    #[error("can not import directory")]
    ImportDirectory,

    #[error("can not get download URL")]
    DownloadEmpty,

    #[error("can not download directory")]
    DownloadDirectory,

    #[error("target file does not exist or has deleted")]
    DownloadFileNotExistOrHasDeleted,

    #[error("target file is too big to download")]
    DownloadFileTooBig,

    #[error("cyclic copy")]
    CyclicCopy,

    #[error("cyclic move")]
    CyclicMove,

    #[error("video is not ready")]
    VideoNotReady,

    #[error("wrong parameters")]
    WrongParams,

    #[error("repeat login")]
    RepeatLogin,

    #[error("failed to login")]
    FailedToLogin,

    #[error("you have been kicked out by multi-device login management")]
    DoesLoggedOut,

    #[error("pickcode does not exist")]
    PickCodeNotExist,

    #[error("shared link invalid")]
    SharedInvalid,

    #[error("shared link not found")]
    SharedNotFound,

    #[error("empty pickcode")]
    PickCodeIsEmpty,

    #[error("userid/filesize/target/pickcode/ invalid")]
    UploadSH1Invalid,

    #[error("sig invalid")]
    UploadSigInvalid,

    #[error("unknown 115 code: {0}")]
    Unknown115Code(i32),

    #[error("unknown 115 error")]
    Unknown115Error,

    #[error("115 error: {0}")]
    Wrap115Error(String),

    #[error("http error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("decode failed: {0}")]
    DecodeFailed(String),

    #[error("decrypt failed: {0}")]
    DecryptFailed(String),

    #[error("encrypt failed: {0}")]
    EncryptFailed(String),

    #[error("deserialize failed: {0}")]
    DeserializeFailed(#[from] serde_json::Error),
}

pub fn map_115_error_code(code: i32) -> Result<(), Pan115Error> {
    match code {
        // Normal errors
        99 | 990001 => Err(Pan115Error::NotLogin),
        // Offline errors
        10010 => Err(Pan115Error::OfflineNoTimes),
        10004 => Err(Pan115Error::OfflineInvalidLink),
        10008 => Err(Pan115Error::OfflineTaskExisted),
        // Dir errors
        20004 => Err(Pan115Error::Exist),
        // Label errors
        21003 => Err(Pan115Error::Exist),
        // File errors
        20130827 => Err(Pan115Error::OrderNotSupport),
        50028 => Err(Pan115Error::DownloadFileTooBig),
        70005 | 231011 => Err(Pan115Error::DownloadFileNotExistOrHasDeleted),
        91002 => Err(Pan115Error::CyclicCopy),
        800006 => Err(Pan115Error::CyclicMove),
        // Login errors
        40101009 => Err(Pan115Error::PasswordIncorrect),
        40101010 => Err(Pan115Error::LoginTwoStepVerify),
        40101017 => Err(Pan115Error::FailedToLogin),
        40100000 => Err(Pan115Error::WrongParams),
        40101030 => Err(Pan115Error::AccountNotBindMobile),
        40101032 => Err(Pan115Error::CredentialInvalid),
        40101033 | 40101038 => Err(Pan115Error::RepeatLogin),
        40101035 => Err(Pan115Error::DoesLoggedOut),
        40101037 => Err(Pan115Error::SessionExited),
        // QRCode errors
        40199002 => Err(Pan115Error::QrcodeExpired),
        // Params errors
        1001 | 200900 | 990002 => Err(Pan115Error::WrongParams),
        // Share errors
        4100009 => Err(Pan115Error::SharedInvalid),
        4100026 => Err(Pan115Error::SharedNotFound),
        // PickCode errors
        50003 => Err(Pan115Error::PickCodeNotExist),
        50001 => Err(Pan115Error::PickCodeIsEmpty),
        // Upload errors
        402 => Err(Pan115Error::UploadSH1Invalid),
        400 => Err(Pan115Error::UploadSigInvalid),
        // Default case
        _ => Err(Pan115Error::Unknown115Code(code)),
    }
}
