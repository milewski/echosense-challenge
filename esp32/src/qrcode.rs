use qrcode_generator::QrCodeEcc;
use crate::custom_error::CustomError;

pub struct QRCode {
    matrix: Vec<Vec<bool>>,
    offset: (usize, usize),
    scale: usize,
    width: usize,
    height: usize,
}

impl QRCode {
    pub fn new<T: AsRef<[u8]>>(content: T, width: usize, height: usize, scale: usize, offset: (usize, usize)) -> Result<Self, CustomError> {
        Ok(Self {
            matrix: qrcode_generator::to_matrix(content, QrCodeEcc::Low)?,
            scale,
            offset,
            width,
            height,
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut byte_vec = vec![0u8; self.width * self.height / 8];

        let qr_width = self.matrix.len();
        let qr_height = self.matrix[0].len();

        for y in 0..self.height {
            for x in 0..self.width {
                // Determine corresponding position in the QR code matrix
                let qr_x = (x / self.scale) - self.offset.0;
                let qr_y = (y / self.scale) - self.offset.1;

                // Ensure we stay within bounds of the QR code
                let pixel_on = if qr_x < qr_width && qr_y < qr_height {
                    self.matrix[qr_y][qr_x]
                } else {
                    false
                };

                let byte_index = (x + y * self.width) / 8;
                let bit_index = 7 - (x % 8);

                if pixel_on {
                    byte_vec[byte_index] |= 1 << bit_index;
                }
            }
        }

        byte_vec
    }
}
