use std::time::Duration;

use esp_idf_svc::http::client::{Configuration, Connection, EspHttpConnection};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::{Read, Write};
use esp_idf_svc::sys::EspError;
use log::error;
use serde::Serialize;

use crate::custom_error::CustomError;
use crate::file_server::Transcription;
use crate::SENDGRID_FROM;

#[derive(Clone)]
pub struct SendGrid {
    api_key: String,
}

#[derive(Debug, Serialize)]
struct Email {
    email: String,
}

#[derive(Debug, Serialize)]
struct Content {
    r#type: String,
    value: String,
}

impl Content {
    fn from_transcriptions(transcription: Vec<Transcription>) -> Self {
        let content = transcription
            .into_iter()
            .map(|transcription| format!("<span><b>{}</b></span>: <span>{}</span>", transcription.timestamp, transcription.text))
            .collect::<Vec<_>>()
            .join("<hr/>");

        let header = "<h2>Transcription:</h2>";

        Self {
            r#type: "text/html".to_string(),
            value: format!("{}{}", header, content),
        }
    }
}

#[derive(Debug, Serialize)]
struct Personalization {
    to: Vec<Email>,
}

#[derive(Debug, Serialize)]
struct SendEmailRequest {
    from: Email,
    subject: String,
    personalizations: Vec<Personalization>,
    content: Vec<Content>,
}

impl SendGrid {
    pub fn new<S: Into<String>>(api_key: S) -> Result<Self, EspError> {
        Ok(SendGrid {
            api_key: api_key.into(),
        })
    }

    pub fn send_email<T: Into<String>>(&mut self, to: T, transcription: Vec<Transcription>) -> Result<(), CustomError> {
        let token = format!("Bearer {}", self.api_key.as_str());
        let headers = [
            ("Authorization", token.as_str()),
            ("Content-Type", "application/json"),
        ];

        let request = SendEmailRequest {
            from: Email {
                email: SENDGRID_FROM.to_string(),
            },
            subject: "Your transcription is ready!".to_string(),
            personalizations: vec![
                Personalization {
                    to: vec![
                        Email {
                            email: to.into()
                        }
                    ]
                }
            ],
            content: vec![
                Content::from_transcriptions(transcription)
            ],
        };

        let mut configuration = Configuration::default();
        configuration.timeout = Some(Duration::from_secs(30));

        let mut client = EspHttpConnection::new(&configuration)?;

        client.initiate_request(Method::Post, "https://api.sendgrid.com/v3/mail/send", &headers)?;
        client.write_all(&serde_json::to_vec(&request)?)?;
        client.flush()?;
        client.initiate_response()?;

        let status = client.status();
        if status >= 200 && status < 300 {
            return Ok(());
        }

        let mut buffer = [0; 1000];
        let mut response = vec![];

        while let Ok(length) = client.read(&mut buffer) {
            if length == 0 { break; }

            response.extend_from_slice(&buffer[..length])
        }

        Err(CustomError::FailedToSendEmail(String::from_utf8(response)?))
    }
}
