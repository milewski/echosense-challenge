#![feature(seek_stream_len)]
#![allow(warnings)]
extern crate core;

use button_driver::{Button, ButtonConfig};
use crossbeam::channel::Sender;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::Drawable;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{OutputPin, PinDriver};
use esp_idf_svc::hal::i2c::I2cDriver;
use esp_idf_svc::hal::i2s::I2sRx;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::esp_random;
use esp_idf_svc::wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi};
use esp_idf_svc::ws::client::EspWebSocketClient;
use esp_idf_svc::ws::FrameType;
use log::{debug, error, info, warn};
use std::cell::{OnceCell, RefCell};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Instant;

use crate::assembly::{Assembly, AssemblyResponse, SummarizeRequest};
use crate::custom_error::CustomError;
use crate::display::{Display, DrawState};
use crate::file_server::{Command, Server, Sessions, Transcription, WebsocketMessage};
use crate::microphone::Microphone;
use crate::mini_sdcard::MiniSDCard;
use crate::network::Network;
use crate::sendgrid::SendGrid;

mod assembly;
mod custom_error;
mod file_server;
mod microphone;
mod mini_sdcard;
mod network;
mod qrcode;
mod stream_audio_writer;
mod display;
mod images;
mod sendgrid;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const ASSEMBLY_APIKEY: &str = env!("ASSEMBLY_APIKEY");
const SENDGRID_APIKEY: &str = env!("SENDGRID_APIKEY");
const SENDGRID_FROM: &str = env!("SENDGRID_FROM");

const SUMMARY_UPLOAD_CHUNK: usize = 1000;
const MICROPHONE_RECORD_BUFFER_SIZE: usize = 1000;
const SAMPLE_RATE_HZ: u32 = 16000;

fn main() -> Result<(), CustomError> {
    EspLogger::initialize_default();

    info!("initializing...");

    let peripherals = Peripherals::take()?;
    let mut toggle = Arc::new(AtomicBool::new(false));
    let toggle_a = toggle.clone();
    let toggle_b = toggle.clone();
    let toggle_c = toggle.clone();

    // Initialize Display
    let mut display = Display::new(
        peripherals.i2c0,
        peripherals.pins.gpio10,
        peripherals.pins.gpio9,
    )?;

    display.draw(DrawState::Initializing)?;

    // Setup push button
    let pin = PinDriver::input(peripherals.pins.gpio5)?;
    let mut button = Button::<_, Instant>::new(pin, ButtonConfig::default());

    // Initialize SD Card / Mount SD Card
    let sdcard = MiniSDCard::new(
        peripherals.spi2,
        peripherals.pins.gpio2, // sck
        peripherals.pins.gpio3, // mosi
        peripherals.pins.gpio1, // miso
        peripherals.pins.gpio4, // cs
        false,
    )?;

    info!("SD Card: {:?}", sdcard.info()?);

    // Initialize microphone
    let microphone = Microphone::<_, MICROPHONE_RECORD_BUFFER_SIZE>::new(
        peripherals.i2s0,
        peripherals.pins.gpio8, // sck
        peripherals.pins.gpio6, // sd
        peripherals.pins.gpio7, // ws
        SAMPLE_RATE_HZ,
    )?;

    // Initialize Wifi network
    let network = Network::new(peripherals.modem, WIFI_SSID, WIFI_PASSWORD)?.connect()?;

    // Initialize AssemblyAI SDK
    let mut assembly = Assembly::new(ASSEMBLY_APIKEY)?;

    // Initialize WebServer
    let mut file_server = Server::new()?;

    file_server.initialize_static_file_server()?;

    info!("Static file server initialized");

    let (frontend_command_sender, frontend_command_receiver) = channel::<Command>();
    let (transcription_uploaded_notifier, transcription_uploader_receiver) = std::sync::mpsc::channel::<String>();

    let transcriptions = Arc::new(Mutex::new(vec![]));
    let transcriptions_a = transcriptions.clone();
    let transcriptions_b = transcriptions.clone();

    let sessions = file_server.initialize_websocket(frontend_command_sender, transcriptions_a)?;
    let sessions_a = sessions.clone();
    let sessions_b = sessions.clone();
    let sessions_c = sessions.clone();

    info!("Websocket initialized.");

    let ip_info = network.ip_info()?;
    let address = format!("http://{}", ip_info.ip);

    info!("Address: {:?}", address);

    display.draw(DrawState::QRCode(address))?;

    {
        let (live_transcription_websocket_notifier, receiver) = assembly.stream(SAMPLE_RATE_HZ)?;

        // spawn(move || generate_summary(transcription_uploader_receiver, sessions_a));
        spawn(move || handle_transcription_thread(receiver, sessions_b, transcriptions));
        spawn(move || {
            handle_frontend_sent_commands(frontend_command_receiver, transcriptions_b, sessions_c, toggle_a)
        });

        let (sender, receiver) = crossbeam::channel::unbounded::<Vec<u8>>();

        let receiver_a = receiver.clone();
        let receiver_b = receiver.clone();

        std::thread::Builder::new()
            // .stack_size(20000)
            .spawn(move || record_microphone(sender, microphone))?;

        std::thread::Builder::new()
            // .stack_size(20000)
            .spawn(move || {
                start_live_transcription(receiver_a, live_transcription_websocket_notifier)
            })?;

        std::thread::Builder::new()
            // .stack_size(20000)
            .spawn(move || record_audio_from_microphone_to_the_sdcard(receiver_b))?;

        loop {
            button.tick();

            if button.is_clicked() {
                display.draw(DrawState::Initializing)?;
                break Ok(());
            }

            if toggle.load(Ordering::Relaxed) {
                display.draw(DrawState::Done)?;
            }

            button.reset();

            FreeRtos::delay_ms(1);
        }
    }
}

fn process_uploading_task_queue(
    filenames_b: Arc<Mutex<VecDeque<String>>>,
    transcription_uploaded_notifier: std::sync::mpsc::Sender<String>,
) -> Result<(), CustomError> {
    loop {
        let mut filename = {
            let mut filenames = filenames_b.lock()?;
            filenames.pop_front()
        };

        if let Some(filename) = filename {
            warn!("uploading...");

            let mut file = File::open(&filename)?;

            if file.stream_len()? == 0 {
                warn!("file length is zero, skipping...");
                continue;
            }

            let mut assembly = Assembly::new(ASSEMBLY_APIKEY)?;
            let response = assembly.upload::<_, SUMMARY_UPLOAD_CHUNK>(&mut file)?;
            let response = assembly.transcribe_wait(&response.upload_url)?;

            info!("uploaded successfully, transcription id: {}", response.id);

            transcription_uploaded_notifier.send(response.id)?;
        }

        FreeRtos::delay_ms(1000)
    }
}

fn record_audio_from_microphone_to_the_sdcard(receiver: crossbeam::channel::Receiver<Vec<u8>>) -> Result<(), CustomError> {
    let random_number = unsafe { esp_random() };
    let filename = format!("/sdcard/{}.wav", random_number);

    let mut audio_file = File::create(&filename)?;

    loop {
        if let Ok(microphone_data) = receiver.recv() {
            audio_file.write_all(microphone_data.as_slice())?;
        }
    }
}

fn periodically_upload_transcriptions(
    receiver: crossbeam::channel::Receiver<Vec<u8>>,
    transcription_uploaded_notifier: std::sync::mpsc::Sender<String>,
) -> Result<(), CustomError> {
    let mut filenames = Arc::new(Mutex::new(VecDeque::<String>::new()));
    let filenames_a = filenames.clone();
    let filenames_b = filenames.clone();

    std::thread::Builder::new().spawn::<_, Result<(), CustomError>>(move || {
        process_uploading_task_queue(filenames_b, transcription_uploaded_notifier)
    })?;

    loop {
        let random_number = unsafe { esp_random() };
        let filename = format!("/sdcard/{}.wav", random_number);

        let mut audio_file = File::create(&filename)?;
        let mut chunks = 0;

        loop {
            if let Ok(microphone_data) = receiver.recv() {
                audio_file.write_all(microphone_data.as_slice())?;
                chunks += 1;
            }

            if chunks >= SUMMARY_UPLOAD_CHUNK {
                let mut filenames = filenames_a.lock()?;
                filenames.push_back(filename);

                break drop(audio_file);
            }
        }
    }
}

fn start_live_transcription(
    receiver: crossbeam::channel::Receiver<Vec<u8>>,
    mut live_transcription_websocket_notifier: EspWebSocketClient,
) -> Result<(), CustomError> {
    loop {
        if let Ok(microphone_data) = receiver.recv() {
            live_transcription_websocket_notifier.send(FrameType::Binary(false), microphone_data.as_slice())?;
        }
    }
}

fn record_microphone(
    sender: Sender<Vec<u8>>,
    mut microphone: Microphone<I2sRx, MICROPHONE_RECORD_BUFFER_SIZE>,
) -> Result<(), CustomError> {
    loop {
        sender.send(microphone.sample()?.to_vec())?;
    }
}

fn handle_frontend_sent_commands(
    frontend_command_receiver: Receiver<Command>,
    transcriptions: Arc<Mutex<Vec<Transcription>>>,
    sessions: Sessions,
    trigger: Arc<AtomicBool>,
) -> Result<(), CustomError> {
    while let Ok(command) = frontend_command_receiver.recv() {
        info!("received command: {:?}", command);

        let mut assembly = Assembly::new(ASSEMBLY_APIKEY)?;
        let transcriptions = {
            let transcriptions = transcriptions.lock()?;
            transcriptions.clone()
        };

        match command {
            Command::GetSummary => {
                let response = assembly.summarize_transcripts(transcriptions)?;

                info!("summary: {:?}", response);

                let sessions = sessions.lock()?;

                for (_, notifier) in sessions.iter() {
                    notifier.send(WebsocketMessage::Summary(response.response.clone()))?;
                }
            }
            Command::AskQuestion { id, question } => {
                let response = assembly.ask_question(question, transcriptions)?;
                let sessions = sessions.lock()?;

                info!("{:?}", response);

                for (_, notifier) in sessions.iter() {
                    notifier.send(WebsocketMessage::AnswerQuestion {
                        id: id.clone(),
                        answer: response.response[0].answer.clone(),
                    })?;
                }
            }
            Command::SendTranscriptionViaEmail { email, with_audio } => {
                let mut sendgrid = SendGrid::new(SENDGRID_APIKEY)?;

                if let Some(email) = email {
                    sendgrid.send_email(email, transcriptions)?
                }

                trigger.store(true, Ordering::Relaxed);
            }
        }
    }

    Ok(())
}

fn generate_summary(receiver: Receiver<String>, sessions: Sessions) -> Result<(), CustomError> {
    let mut assembly = Assembly::new(ASSEMBLY_APIKEY)?;
    let mut uploaded_transcriptions_ids = vec![];

    loop {
        if let Ok(transcription_id) = receiver.recv() {
            warn!("generating summary: {:?}", transcription_id);

            uploaded_transcriptions_ids.push(transcription_id);

            // let response = assembly.summarize_transcripts(SummarizeRequest {
            //     transcript_ids: uploaded_transcriptions_ids.clone(),
            //     ..Default::default()
            // }).unwrap();

            // info!("summary: {:?}", response);
            //
            // let sessions = sessions.lock()?;
            //
            // for (_, notifier) in sessions.iter() {
            //     notifier.send(WebsocketMessage::Summary(response.response.clone()))?;
            // }
        }
    }
}

fn handle_transcription_thread(
    receiver: Receiver<AssemblyResponse>,
    sessions: Sessions,
    transcriptions: Arc<Mutex<Vec<Transcription>>>,
) -> Result<(), CustomError> {
    loop {
        if let Ok(message) = receiver.recv() {
            match message {
                AssemblyResponse::PartialTranscript { text, created } => {
                    if text.is_empty() == false {
                        let transcription = Transcription {
                            text: text.clone(),
                            timestamp: created.clone(),
                        };

                        let sessions = sessions.lock()?;

                        for (_, notifier) in sessions.iter() {
                            // info!("P: ...{}", &text[text.len().saturating_sub(100)..]);
                            notifier.send(WebsocketMessage::PartialTranscription(
                                transcription.clone(),
                            ))?
                        }
                    }
                }
                AssemblyResponse::FinalTranscript { text, created } => {
                    println!("{:?}", created);
                    if text.is_empty() == false {
                        let transcription = Transcription {
                            text: text.clone(),
                            timestamp: created.clone(),
                        };

                        {
                            let mut transcriptions = transcriptions.lock()?;
                            transcriptions.push(transcription.clone())
                        }

                        let sessions = sessions.lock()?;
                        for (_, notifier) in sessions.iter() {
                            // warn!("F: ...{}", &text[text.len().saturating_sub(100)..]);
                            notifier
                                .send(WebsocketMessage::FinalTranscription(transcription.clone()))?
                        }
                    }
                }
                AssemblyResponse::SessionBegins { .. } => {}
                AssemblyResponse::SessionInformation { .. } => {}
                AssemblyResponse::SessionTerminated => {}
            }
        }
    }
}
