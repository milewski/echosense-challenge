use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::http::client::{Configuration, Connection, EspHttpConnection};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{Read, Write};
use esp_idf_svc::sys::EspError;
use esp_idf_svc::tls::X509;
use esp_idf_svc::ws::client::{EspWebSocketClient, EspWebSocketClientConfig, WebSocketEventType};
use log::{error, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::custom_error::CustomError;
use crate::file_server::Transcription;
use crate::stream_audio_writer::AudioWriter;

const SERVER_ROOT_CERT: &[u8] = b"
-----BEGIN CERTIFICATE-----
MIIEXjCCA0agAwIBAgITB3MSSkvL1E7HtTvq8ZSELToPoTANBgkqhkiG9w0BAQsF
ADA5MQswCQYDVQQGEwJVUzEPMA0GA1UEChMGQW1hem9uMRkwFwYDVQQDExBBbWF6
b24gUm9vdCBDQSAxMB4XDTIyMDgyMzIyMjUzMFoXDTMwMDgyMzIyMjUzMFowPDEL
MAkGA1UEBhMCVVMxDzANBgNVBAoTBkFtYXpvbjEcMBoGA1UEAxMTQW1hem9uIFJT
QSAyMDQ4IE0wMjCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBALtDGMZa
qHneKei1by6+pUPPLljTB143Si6VpEWPc6mSkFhZb/6qrkZyoHlQLbDYnI2D7hD0
sdzEqfnuAjIsuXQLG3A8TvX6V3oFNBFVe8NlLJHvBseKY88saLwufxkZVwk74g4n
WlNMXzla9Y5F3wwRHwMVH443xGz6UtGSZSqQ94eFx5X7Tlqt8whi8qCaKdZ5rNak
+r9nUThOeClqFd4oXych//Rc7Y0eX1KNWHYSI1Nk31mYgiK3JvH063g+K9tHA63Z
eTgKgndlh+WI+zv7i44HepRZjA1FYwYZ9Vv/9UkC5Yz8/yU65fgjaE+wVHM4e/Yy
C2osrPWE7gJ+dXMCAwEAAaOCAVowggFWMBIGA1UdEwEB/wQIMAYBAf8CAQAwDgYD
VR0PAQH/BAQDAgGGMB0GA1UdJQQWMBQGCCsGAQUFBwMBBggrBgEFBQcDAjAdBgNV
HQ4EFgQUwDFSzVpQw4J8dHHOy+mc+XrrguIwHwYDVR0jBBgwFoAUhBjMhTTsvAyU
lC4IWZzHshBOCggwewYIKwYBBQUHAQEEbzBtMC8GCCsGAQUFBzABhiNodHRwOi8v
b2NzcC5yb290Y2ExLmFtYXpvbnRydXN0LmNvbTA6BggrBgEFBQcwAoYuaHR0cDov
L2NydC5yb290Y2ExLmFtYXpvbnRydXN0LmNvbS9yb290Y2ExLmNlcjA/BgNVHR8E
ODA2MDSgMqAwhi5odHRwOi8vY3JsLnJvb3RjYTEuYW1hem9udHJ1c3QuY29tL3Jv
b3RjYTEuY3JsMBMGA1UdIAQMMAowCAYGZ4EMAQIBMA0GCSqGSIb3DQEBCwUAA4IB
AQAtTi6Fs0Azfi+iwm7jrz+CSxHH+uHl7Law3MQSXVtR8RV53PtR6r/6gNpqlzdo
Zq4FKbADi1v9Bun8RY8D51uedRfjsbeodizeBB8nXmeyD33Ep7VATj4ozcd31YFV
fgRhvTSxNrrTlNpWkUk0m3BMPv8sg381HhA6uEYokE5q9uws/3YkKqRiEz3TsaWm
JqIRZhMbgAfp7O7FUwFIb7UIspogZSKxPIWJpxiPo3TcBambbVtQOcNRWz5qCQdD
slI2yayq0n2TXoHyNCLEH8rpsJRVILFsg0jc7BaFrMnF462+ajSehgj12IidNeRN
4zl+EoNaWdpnWndvSpAEkq2P
-----END CERTIFICATE-----\0";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum AssemblyResponse {
    PartialTranscript { text: String, created: String },
    FinalTranscript { text: String, created: String },
    SessionBegins { session_id: String },
    SessionInformation { audio_duration_seconds: f32 },
    SessionTerminated,
}

#[derive(Debug, Serialize)]
pub struct SummarizeRequest {
    pub context: String,
    pub final_model: String,
    pub max_output_size: u16,
    pub temperature: f32,
    pub input_text: String,
    pub answer_format: String,
}

#[derive(Debug, Deserialize)]
pub struct SummarizeResponse {
    pub request_id: String,
    pub response: String,
}

impl Default for SummarizeRequest {
    fn default() -> Self {
        SummarizeRequest {
            context: String::new(),
            final_model: "anthropic/claude-3-5-sonnet".to_string(),
            max_output_size: 3000,
            temperature: 0.0,
            input_text: String::new(),
            answer_format: "bullet points".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub upload_url: String,
}

#[derive(Debug, Serialize)]
pub struct TranscribeRequest {
    audio_url: String,
}

#[derive(Debug, Serialize)]
pub struct AskQuestionQuestion {
    question: String,
    answer_format: String,
}

#[derive(Debug, Serialize)]
pub struct AskQuestionRequest {
    questions: Vec<AskQuestionQuestion>,
    input_text: String,
}

#[derive(Debug, Deserialize)]
pub struct AskQuestionResponseItem {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct AskQuestionResponse {
    pub request_id: String,
    pub response: Vec<AskQuestionResponseItem>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionStatus {
    Queued,
    Processing,
    Completed,
    Error,
}

#[derive(Debug, Deserialize)]
pub struct TranscribeResponse {
    pub id: String,
    pub status: TranscriptionStatus,
    pub text: Option<String>,
}

#[derive(Clone)]
pub struct Assembly {
    api_key: String,
}

impl Assembly {
    pub fn new<S: Into<String>>(api_key: S) -> Result<Self, EspError> {
        Ok(Assembly {
            api_key: api_key.into(),
        })
    }

    pub fn summarize_transcripts(
        &mut self,
        transcription: Vec<Transcription>,
    ) -> Result<SummarizeResponse, CustomError> {
        let headers = [
            ("Authorization", self.api_key.as_str()),
            ("Content-Type", "application/json"),
        ];

        let mut request = SummarizeRequest::default();

        request.input_text = transcription
            .into_iter()
            .map(|transcription| transcription.text)
            .collect::<Vec<_>>()
            .join("\n");

        let mut configuration = Configuration::default();
        configuration.timeout = Some(Duration::from_secs(30));

        let mut client = EspHttpConnection::new(&configuration)?;

        client.initiate_request(
            Method::Post,
            "https://api.assemblyai.com/lemur/v3/generate/summary",
            &headers,
        )?;
        client.write_all(&serde_json::to_vec(&request)?)?;
        client.flush()?;
        client.initiate_response()?;
        self.handle_request::<SummarizeResponse, 5000>(client)
    }

    pub fn get_transcript<T: Into<String>>(
        &mut self,
        transcript_id: T,
    ) -> Result<TranscribeResponse, CustomError> {
        let headers = [("Authorization", self.api_key.as_str())];

        let url = format!(
            "https://api.assemblyai.com/v2/transcript/{}",
            transcript_id.into()
        );
        let mut client = EspHttpConnection::new(&Default::default())?;

        client.initiate_request(Method::Get, url.as_str(), &headers)?;
        client.flush()?;
        client.initiate_response()?;

        self.handle_request::<TranscribeResponse, 1000>(client)
    }

    pub fn ask_question<T: Into<String>>(
        &mut self,
        question: T,
        transcription: Vec<Transcription>,
    ) -> Result<AskQuestionResponse, CustomError> {
        let headers = [
            ("Authorization", self.api_key.as_str()),
            ("Content-Type", "application/json"),
        ];

        let mut client = EspHttpConnection::new(&Default::default())?;

        client.initiate_request(
            Method::Post,
            "https://api.assemblyai.com/lemur/v3/generate/question-answer",
            &headers,
        )?;

        let request = AskQuestionRequest {
            questions: vec![AskQuestionQuestion {
                question: question.into(),
                answer_format: "short sentence".to_string(),
            }],
            input_text: transcription
                .into_iter()
                .map(|transcription| transcription.text)
                .collect::<Vec<_>>()
                .join("\n"),
        };

        client.write_all(&serde_json::to_vec(&request)?)?;
        client.flush()?;
        client.initiate_response()?;

        self.handle_request::<AskQuestionResponse, 1000>(client)
    }

    pub fn transcribe<T: Into<String>>(
        &mut self,
        audio_url: T,
    ) -> Result<TranscribeResponse, CustomError> {
        let headers = [
            ("Authorization", self.api_key.as_str()),
            ("Content-Type", "application/json"),
        ];

        let mut client = EspHttpConnection::new(&Default::default())?;

        client.initiate_request(
            Method::Post,
            "https://api.assemblyai.com/v2/transcript",
            &headers,
        )?;

        let request = TranscribeRequest {
            audio_url: audio_url.into(),
        };

        client.write_all(&serde_json::to_vec(&request)?)?;
        client.flush()?;
        client.initiate_response()?;

        self.handle_request::<TranscribeResponse, 1000>(client)
    }

    pub fn transcribe_wait<T: Into<String>>(
        &mut self,
        audio_url: T,
    ) -> Result<TranscribeResponse, CustomError> {
        let response = self.transcribe(audio_url)?;

        loop {
            if let Ok(response) = self.get_transcript(&response.id) {
                match response.status {
                    TranscriptionStatus::Completed => break Ok(response),
                    TranscriptionStatus::Queued | TranscriptionStatus::Processing => {
                        FreeRtos::delay_ms(100)
                    }
                    TranscriptionStatus::Error => panic!("failed to transcribe audio..."),
                }
            }
        }
    }

    pub fn upload<T: std::io::Read, const BUFFER_SIZE: usize>(
        &mut self,
        file: &mut T,
    ) -> Result<UploadResponse, CustomError> {
        let headers = [
            ("Authorization", self.api_key.as_str()),
            ("Content-Type", "application/octet-stream"),
        ];

        let mut client = EspHttpConnection::new(&Default::default())?;

        client.initiate_request(
            Method::Post,
            "https://api.assemblyai.com/v2/upload",
            &headers,
        )?;

        {
            let mut buffer = [0u8; BUFFER_SIZE];
            let mut audio = AudioWriter::initialize::<BUFFER_SIZE>(&mut client)?;

            while let Ok(length) = file.read(&mut buffer) {
                if length == 0 {
                    break;
                }

                for sample in &buffer[..length] {
                    audio.write_sample_u8(*sample).unwrap();
                }
            }

            audio.sync_header()?;
        };

        client.flush()?;
        client.initiate_response()?;

        self.handle_request::<UploadResponse, 500>(client)
    }

    pub fn create_temporary_token(&mut self) -> Result<TokenResponse, CustomError> {
        let headers = [
            ("Authorization", self.api_key.as_str()),
            ("Content-Type", "application/json"),
        ];

        let mut client = EspHttpConnection::new(&Default::default())?;

        client.initiate_request(
            Method::Post,
            "https://api.assemblyai.com/v2/realtime/token",
            &headers,
        )?;

        let payload = b"{\"expires_in\": 3600}";

        client.write_all(payload)?;
        client.flush()?;
        client.initiate_response()?;

        self.handle_request::<TokenResponse, 100>(client)
    }

    pub fn stream(&mut self, sample_rate: u32) -> Result<(EspWebSocketClient<'static>, Receiver<AssemblyResponse>), CustomError> {
        let token = self.create_temporary_token()?.token;

        let config = EspWebSocketClientConfig {
            server_cert: Some(X509::pem_until_nul(SERVER_ROOT_CERT)),
            buffer_size: 1024,
            ..Default::default()
        };

        let timeout = Duration::from_secs(30);
        let endpoint = format!("wss://api.assemblyai.com/v2/realtime/ws?sample_rate={}&enable_extra_session_information=true&token={}", sample_rate, token);

        let (sender, receiver) = channel::<AssemblyResponse>();
        let mut response_buffer = vec![];

        let mut client = EspWebSocketClient::new(&endpoint, &config, timeout, move |event| {
            if let Ok(event) = event {
                match event.event_type {
                    WebSocketEventType::BeforeConnect => info!("BeforeConnect!"),
                    WebSocketEventType::Connected => info!("Connected!"),
                    WebSocketEventType::Disconnected => info!("Disconnected!"),
                    WebSocketEventType::Close(_) => {}
                    WebSocketEventType::Closed => info!("Closed!"),
                    WebSocketEventType::Text(text) => {
                        response_buffer.extend_from_slice(text.as_bytes());

                        if let Ok(response) = serde_json::from_slice::<AssemblyResponse>(response_buffer.as_slice()) {
                            response_buffer.clear();

                            if let Err(error) = sender.send(response) {
                                error!("failed to send response: {}", error);
                            }
                        }
                    }
                    WebSocketEventType::Binary(_) => info!("Got Binary Data!"),
                    WebSocketEventType::Ping => info!("Ping!"),
                    WebSocketEventType::Pong => info!("Pong!"),
                }
            }
        })?;

        while client.is_connected() == false {
            FreeRtos::delay_ms(10);
        }

        Ok((client, receiver))
    }

    fn handle_request<T: DeserializeOwned, const BUFFER_SIZE: usize>(
        &self,
        mut client: EspHttpConnection,
    ) -> Result<T, CustomError> {
        let mut buffer = [0; BUFFER_SIZE];
        let mut response = vec![];

        while let Ok(length) = client.read(&mut buffer) {
            if length == 0 {
                break;
            }
            response.extend_from_slice(&buffer[..length])
        }

        Ok(serde_json::from_slice::<T>(&response)?)
    }
}
