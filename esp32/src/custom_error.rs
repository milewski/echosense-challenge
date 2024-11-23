use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::string::FromUtf8Error;
use std::sync::mpsc::{SendError, Sender};
use std::sync::{MutexGuard, PoisonError};

use esp_idf_svc::io::{EspIOError, ReadExactError};
use esp_idf_svc::sys::EspError;
use qrcode_generator::QRCodeError;
use riff_wave::WriteError;

use crate::file_server::WebsocketMessage;

#[derive(Debug)]
pub enum CustomError {
    // Generic(Box<dyn Error>),
    MutexLockError(String),
    AnyhowError(anyhow::Error),
    EspError(EspError),
    EspIOError(EspIOError),
    SerdeJsonError(serde_json::Error),
    MicrophoneReadExactError(ReadExactError<EspIOError>),
    IOError(std::io::Error),
    FromUtf8Error(FromUtf8Error),
    WebsocketSendError(SendError<WebsocketMessage>),
    ChannelSendError,
    QRCodeError(QRCodeError),
    DisplayError(display_interface::DisplayError),
    WriteWavFileError(WriteError),
    FailedToSendEmail(String)
}

impl Display for CustomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Something went wrong!")
    }
}

impl Error for CustomError {}

impl From<EspError> for CustomError {
    fn from(error: EspError) -> Self {
        CustomError::EspError(error)
    }
}

impl From<EspIOError> for CustomError {
    fn from(error: EspIOError) -> Self {
        CustomError::EspIOError(error)
    }
}

impl From<serde_json::Error> for CustomError {
    fn from(error: serde_json::Error) -> Self {
        CustomError::SerdeJsonError(error)
    }
}

impl From<ReadExactError<EspIOError>> for CustomError {
    fn from(error: ReadExactError<EspIOError>) -> Self {
        CustomError::MicrophoneReadExactError(error)
    }
}

impl From<std::io::Error> for CustomError {
    fn from(error: std::io::Error) -> Self {
        CustomError::IOError(error)
    }
}

impl From<FromUtf8Error> for CustomError {
    fn from(error: FromUtf8Error) -> Self {
        CustomError::FromUtf8Error(error)
    }
}

// impl From<SendError<WebsocketMessage>> for CustomError {
//     fn from(error: SendError<WebsocketMessage>) -> Self {
//         CustomError::WebsocketSendError(error)
//     }
// }

impl From<WriteError> for CustomError {
    fn from(error: WriteError) -> Self {
        CustomError::WriteWavFileError(error)
    }
}

// impl From<Box<dyn Error>> for CustomError {
//     fn from(error: Box<dyn Error>) -> Self {
//         CustomError::Generic(error)
//     }
// }

impl From<anyhow::Error> for CustomError {
    fn from(error: anyhow::Error) -> Self {
        CustomError::AnyhowError(error)
    }
}

impl<'a, T> From<PoisonError<MutexGuard<'a, T>>> for CustomError {
    fn from(error: PoisonError<MutexGuard<T>>) -> Self {
        CustomError::MutexLockError(error.to_string())
    }
}

impl<T> From<SendError<T>> for CustomError {
    fn from(error: SendError<T>) -> Self {
        CustomError::ChannelSendError
    }
}

impl<T> From<crossbeam::channel::SendError<T>> for CustomError {
    fn from(error: crossbeam::channel::SendError<T>) -> Self {
        CustomError::ChannelSendError
    }
}

impl From<display_interface::DisplayError> for CustomError {
    fn from(error: display_interface::DisplayError) -> Self {
        CustomError::DisplayError(error)
    }
}

impl From<QRCodeError> for CustomError {
    fn from(error: QRCodeError) -> Self {
        CustomError::QRCodeError(error)
    }
}