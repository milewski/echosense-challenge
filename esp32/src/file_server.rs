use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::ws::FrameType;
use log::info;
use serde::{Deserialize, Serialize};

use crate::custom_error::CustomError;

pub type Sessions = Arc<Mutex<HashMap<i32, Sender<WebsocketMessage>>>>;

#[derive(Debug, Deserialize)]
pub enum Command {
    GetSummary,
    AskQuestion { id: String, question: String },
    SendTranscriptionViaEmail {
        email: Option<String>,
        with_audio: bool,
    }
}

#[derive(Debug, Deserialize)]
struct Action {
    command: Command,
}

pub struct Server {
    sessions: Sessions,
    inner: EspHttpServer<'static>,
}

const INDEX_HTML: &'static [u8] = include_bytes!("../../frontend/dist/index.html");

#[derive(Debug, Serialize)]
pub enum WebsocketMessage {
    PartialTranscription(Transcription),
    FinalTranscription(Transcription),
    Summary(String),
    AnswerQuestion { id: String, answer: String },
}

impl Into<Payload> for WebsocketMessage {
    fn into(self) -> Payload {
        match self {
            WebsocketMessage::PartialTranscription(transcription) => {
                Payload::PartialTranscription(transcription)
            }
            WebsocketMessage::FinalTranscription(transcription) => {
                Payload::FinalTranscription(transcription)
            }
            WebsocketMessage::Summary(text) => Payload::Summary(text),
            WebsocketMessage::AnswerQuestion { id, answer } => {
                Payload::AnswerQuestion { id, answer }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Transcription {
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
enum Payload {
    Summary(String),
    SessionId(i32),
    AnswerQuestion { id: String, answer: String },
    Transcriptions(Vec<Transcription>),
    PartialTranscription(Transcription),
    FinalTranscription(Transcription),
}

impl Server {
    pub fn new() -> Result<Self, CustomError> {
        Ok(Server {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            inner: EspHttpServer::new(&Configuration::default())?,
        })
    }

    pub fn initialize_websocket(
        &mut self,
        frontend_command_sender: Sender<Command>,
        transcriptions: Arc<Mutex<Vec<Transcription>>>,
    ) -> Result<Sessions, CustomError> {
        let sessions = self.sessions.clone();

        self.inner.ws_handler("/connect", move |socket| {
            let (sender, receiver) = channel::<WebsocketMessage>();

            if socket.is_new() {
                let mut sessions = sessions.lock().unwrap();
                let session_id = socket.session();

                sessions.insert(session_id, sender);

                info!("new session: {:?}", socket.session());

                {
                    let transcriptions = transcriptions.lock().unwrap();

                    if transcriptions.is_empty() == false {
                        let message = Payload::Transcriptions(transcriptions.clone());
                        let message = serde_json::to_string::<Payload>(&message.into()).unwrap();

                        socket.send(FrameType::Text(false), message.as_bytes())?;
                    }
                }

                {
                    let message = Payload::SessionId(socket.session());
                    let message = serde_json::to_string::<Payload>(&message.into()).unwrap();

                    socket.send(FrameType::Text(false), message.as_bytes())?;
                }

                let mut dettached = socket.create_detached_sender()?;

                spawn::<_, Result<(), CustomError>>(move || {
                    loop {
                        if let Ok(message) = receiver.recv_timeout(Duration::from_millis(1000)) {
                            dettached.send(
                                FrameType::Text(false),
                                serde_json::to_string::<Payload>(&message.into())?.as_bytes(),
                            )?;
                        }

                        if dettached.is_closed() {
                            break println!("closing websocket connection");
                        }
                    }

                    Ok(())
                });

                return Ok(());
            }

            if socket.is_closed() {
                let mut sessions = sessions.lock().unwrap();
                let session_id = socket.session();

                sessions.remove(&session_id);

                info!("closed websocket session {:?}", session_id);

                return Ok(());
            }

            let (_, _) = match socket.recv(&mut []) {
                Ok(frame) => frame,
                Err(error) => return Err(error),
            };

            let mut buffer = [0; 1000];
            let (_, length) = socket.recv(&mut buffer)?;

            let message = CStr::from_bytes_until_nul(&buffer[..length]).unwrap();
            let message = message.to_str().unwrap();
            let message: Action = serde_json::from_str(message).unwrap();

            frontend_command_sender.send(message.command).unwrap();

            Ok::<(), EspError>(())
        })?;

        Ok(self.sessions.clone())
    }

    pub fn initialize_static_file_server(&mut self) -> Result<(), CustomError> {
        self.inner.fn_handler("/", Method::Get, |request| {
            request
                .into_ok_response()?
                .write_all(INDEX_HTML)
                .map(|_| ())
        })?;

        Ok(())
    }
}
