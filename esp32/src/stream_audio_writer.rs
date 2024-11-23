use std::io::{BufWriter, SeekFrom};

use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::io::Write;
use riff_wave::{WaveWriter, WriteResult};

use crate::SAMPLE_RATE_HZ;

pub struct AudioWriter<'a> {
    client: &'a mut EspHttpConnection,
    position: usize,
    size: usize,
}

impl<'a> AudioWriter<'a> {
    fn new(client: &'a mut EspHttpConnection) -> AudioWriter<'a> {
        Self {
            client,
            position: 0,
            size: 0,
        }
    }

    pub fn initialize<const BUFFER_SIZE: usize>(client: &'a mut EspHttpConnection) -> WriteResult<WaveWriter<BufWriter<AudioWriter<'a>>>> {
        WaveWriter::new(1, SAMPLE_RATE_HZ, 16, BufWriter::with_capacity(BUFFER_SIZE, Self::new(client)))
    }
}

impl<'a> std::io::Write for AudioWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let written = self
            .client
            .write(buf)
            .expect("could not push down bytes to client...");
        self.size += written;
        Ok(written)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.client.flush().expect("unable to flush bytes...");
        Ok(())
    }
}

impl<'a> std::io::Seek for AudioWriter<'a> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.size as i64 + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        self.position = new_position as usize;

        Ok(self.position as u64)
    }
}
