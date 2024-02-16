/// Wav file writer

use crate::header::{WavHeader, SampleFormat, WavData};
use std::io::{Cursor, Write, Read};
use std::fs::File;

const ERR_UNSUPPORTED_FORMAT: &str = "unsupported wav format";
const ERR_IO_ERROR: &str = "io error";

/// WavData to file
pub fn to_file(file_out: &mut File, wav: &WavData) -> Result<(), &'static str> {
    let mut w = Writer::new();
    match w.from_scratch(&wav.header, &wav.samples) {
        Err(err) => return Err(err),
        Ok(_) => {},
    }
    match w.to_file(file_out) {
        Err(_) => return Err(ERR_IO_ERROR),
        Ok(_) => {},
    }
    Ok(())
}

/// WavData to bytes
pub fn to_bytes(head: &WavHeader, samples: &Vec<f32>) -> Result<Vec<u8>, &'static str> {
    let mut w = Writer::new();
    match w.from_scratch(head, samples) {
        Err(err) => return Err(err),
        Ok(_) => {},
    }
    Ok(w.to_bytes())
}

/// Samples: Vec<i16> to file
pub fn i16samples_to_file(file_out: &mut File, header: &WavHeader, samples: &Vec<i16>) -> Result<(), &'static str> {
    let mut w = Writer::new();
    match w.from_scratch_i16(header, samples) {
        Err(err) => return Err(err),
        Ok(_) => {},
    }
    match w.to_file(file_out) {
        Err(_) => return Err(ERR_IO_ERROR),
        Ok(_) => {},
    }
    Ok(())
}

/// Samples: Vec<i32> to file
pub fn i32samples_to_file(file_out: &mut File, header: &WavHeader, samples: &Vec<i32>) -> Result<(), &'static str> {
    let mut w = Writer::new();
    match w.from_scratch_i(header, samples) {
        Err(err) => return Err(err),
        Ok(_) => {},
    }
    match w.to_file(file_out) {
        Err(_) => return Err(ERR_IO_ERROR),
        Ok(_) => {},
    }
    Ok(())
}

/// Samples: Vec<f32> to file
pub fn f32samples_to_file(file_out: &mut File, header: &WavHeader, samples: &Vec<f32>) -> Result<(), &'static str> {
    let mut w = Writer::new();
    match w.from_scratch(header, samples) {
        Err(err) => return Err(err),
        Ok(_) => {},
    }
    match w.to_file(file_out) {
        Err(_) => return Err(ERR_IO_ERROR),
        Ok(_) => {},
    }
    Ok(())
}

/// Generate WAV file data
pub struct Writer {
    cur: Cursor<Vec<u8>>,
}

impl Writer {
    /// new struct
    pub fn new() -> Self {
        Self {
            cur: Cursor::new(Vec::<u8>::new())
        }
    }
    /// write RIFF header
    pub fn write_riff_header(&mut self, head: &WavHeader, samples_len: u32) -> Result<(), &'static str> {
        let n_bytes = (head.bits_per_sample / 8) as u32;
        let data_size = n_bytes as u32 * samples_len as u32;
        let chunk_size = 4 + 24 + (8 + data_size);
        // write header
        self.write_str("RIFF");
        self.write_u32(chunk_size);
        self.write_str("WAVE");
        self.write_str("fmt ");
        self.write_u32(16);
        let audio_format = match head.sample_format {
            SampleFormat::Int => 1,
            SampleFormat::Float => 3,
            _ => return Err(ERR_UNSUPPORTED_FORMAT),
        };
        self.write_u16(audio_format);
        self.write_u16(head.channels);
        self.write_u32(head.sample_rate);
        self.write_u32(head.sample_rate * n_bytes * head.channels as u32);
        self.write_u16(n_bytes as u16 * head.channels);
        self.write_u16(head.bits_per_sample);
        // has LIST header
        match head.clone().list_chunk {
            Some(list) => {
                let block = list.make_block();
                self.write_str("LIST");
                self.write_u32(block.len() as u32 + 4);
                self.write_str("INFO");
                self.cur.write(&block).unwrap();
            },
            None => {},
        }
        Ok(())
    }
    /// write sample to bytes
    pub fn from_scratch(&mut self, head: &WavHeader, samples: &Vec<f32>) -> Result<(), &'static str> {
        // calc data_size
        let n_bytes = (head.bits_per_sample / 8) as u32;
        let mut samples_len = samples.len();
        let has_pad = samples_len % 2;
        samples_len += has_pad as usize;
        let data_size = n_bytes as u32 * samples_len as u32;
        // write riff header
        match self.write_riff_header(head, samples_len as u32) {
            Err(err) => return Err(err),
            Ok(_) => {},
        }
        // write data header
        self.write_str("data");
        self.write_u32(data_size);
        // write samples
        match head.sample_format {
            SampleFormat::Int => {
                match head.bits_per_sample {
                    8 => for v in samples.iter() { self.write_f32_to_u8(*v); },
                    16 => for v in samples.iter() { self.write_f32_to_i16(*v); },
                    24 => for v in samples.iter() { self.write_f32_to_i24(*v); },
                    32 => for v in samples.iter() { self.write_f32_to_i32(*v); },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            SampleFormat::Float => {
                match head.bits_per_sample {
                    32 => for v in samples.iter() { self.write_f32(*v) },
                    64 => for v in samples.iter() { self.write_f64(*v as f64) },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            _ => return Err(ERR_UNSUPPORTED_FORMAT),
        }
        Ok(())
    }
    /// write sample(Vec<i32>) to bytes
    pub fn from_scratch_i(&mut self, head: &WavHeader, samples: &Vec<i32>) -> Result<(), &'static str> {
        // calc data_size
        let n_bytes = (head.bits_per_sample / 8) as u32;
        let mut samples_len = samples.len();
        let has_pad = samples_len % 2;
        samples_len += has_pad as usize;
        let data_size = n_bytes as u32 * samples_len as u32;
        // write riff header
        match self.write_riff_header(head, samples_len as u32) {
            Err(err) => return Err(err),
            Ok(_) => {},
        }
        // write data header
        self.write_str("data");
        self.write_u32(data_size);
        let max = std::i32::MAX as f32;
        let get_rate = |v:i32| -> f32 { v as f32 / max };
        // write samples
        match head.sample_format {
            SampleFormat::Int => {
                match head.bits_per_sample {
                    8 => for v in samples.iter() { self.write_u8( (get_rate(*v) * 127f32 + 127f32) as u8 ); },
                    16 => for v in samples.iter() { self.write_i16( (get_rate(*v) * std::i16::MAX as f32) as i16); },
                    24 => {
                        let max24 = (0xFFFFFFu32 / 2 - 1) as f32;
                        for v in samples.iter() { self.write_i24( (get_rate(*v) * max24) as i32); }
                    },
                    32 => for v in samples.iter() { self.write_i32(*v as i32); },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            SampleFormat::Float => {
                match head.bits_per_sample {
                    32 => for v in samples.iter() { self.write_f32(get_rate(*v)) },
                    64 => for v in samples.iter() { self.write_f64(get_rate(*v) as f64) },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            _ => return Err(ERR_UNSUPPORTED_FORMAT),
        }
        Ok(())
    }

    /// write sample(Vec<i16>) to bytes
    pub fn from_scratch_i16(&mut self, head: &WavHeader, samples: &Vec<i16>) -> Result<(), &'static str> {
        // calc data_size
        let n_bytes = (head.bits_per_sample / 8) as u32;
        let mut samples_len = samples.len();
        let has_pad = samples_len % 2;
        samples_len += has_pad as usize;
        let data_size = n_bytes as u32 * samples_len as u32;
        // write riff header
        match self.write_riff_header(head, samples_len as u32) {
            Err(err) => return Err(err),
            Ok(_) => {},
        }
        // write data header
        self.write_str("data");
        self.write_u32(data_size);
        let max = std::i16::MAX as f32;
        let get_rate = |v:i16| -> f32 { v as f32 / max };
        // write samples
        match head.sample_format {
            SampleFormat::Int => {
                match head.bits_per_sample {
                    8 => for v in samples.iter() { self.write_u8( (get_rate(*v) * 127f32 + 127f32) as u8 ); },
                    16 => for v in samples.iter() { self.write_i16( *v ); },
                    24 => {
                        let max24 = (0xFFFFFFu32 / 2 - 1) as f32;
                        for v in samples.iter() { self.write_i24( (get_rate(*v) * max24) as i32); }
                    },
                    32 => for v in samples.iter() { self.write_i32( (get_rate(*v) * std::i32::MAX as f32) as i32 ); },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            SampleFormat::Float => {
                match head.bits_per_sample {
                    32 => for v in samples.iter() { self.write_f32(get_rate(*v)) },
                    64 => for v in samples.iter() { self.write_f64(get_rate(*v) as f64) },
                    _ => return Err(ERR_UNSUPPORTED_FORMAT),
                }
            },
            _ => return Err(ERR_UNSUPPORTED_FORMAT),
        }
        Ok(())
    }

    /// write bytes to file
    pub fn to_file(&mut self, file: &mut File) -> Result<usize, std::io::Error> {
        let mut data:Vec<u8> = Vec::new();
        self.cur.set_position(0);
        self.cur.read_to_end(&mut data).unwrap();
        file.write(&data)
    }

    /// write bytes to Vec<u8>
    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut data:Vec<u8> = Vec::new();
        self.cur.set_position(0);
        self.cur.read_to_end(&mut data).unwrap();
        data
    }

    pub fn write_str(&mut self, tag: &str) {
        let bytes:Vec<u8> = tag.bytes().collect();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f32(&mut self, v: f32) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f64(&mut self, v: f64) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f32_to_u8(&mut self, v: f32) {
        let iv:u8 = ((v * 128.0) as i16 + 127) as u8;
        let bytes = iv.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f32_to_i24(&mut self, v: f32) {
        let iv:i32 = (v * 2_147_483_648f32) as i32;
        let bytes = iv.to_le_bytes();
        let wb:[u8; 3] = [bytes[1], bytes[2], bytes[3]];
        self.cur.write(&wb).unwrap();
    }
    pub fn write_f32_to_i16(&mut self, v: f32) {
        let iv:i16 = (v * 32768f32) as i16;
        let bytes = iv.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f32_to_i32(&mut self, v: f32) {
        let iv:i32 = (v * 2_147_483_648f32) as i32;
        let bytes = iv.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_u32(&mut self, v: u32) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_u24(&mut self, v: u32) {
        let b0:u8 = ((v >> 0) & 0xFF) as u8;
        let b1:u8 = ((v >> 8) & 0xFF) as u8;
        let b2:u8 = ((v >> 16) & 0xFF) as u8;
        let bytes = [b0, b1, b2];
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_u16(&mut self, v: u16) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_u8(&mut self, v: u8) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_i16(&mut self, v: i16) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_i32(&mut self, v: i32) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_i24(&mut self, v: i32) {
        let b = Self::i24_to_bytes(v);
        let wb:[u8; 3] = [b[0], b[1], b[2]];
        self.cur.write(&wb).unwrap();
    }
    pub fn i24_to_bytes(v:i32) -> [u8; 3] {
        let b = v.to_le_bytes();
        [b[0], b[1], b[2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    
    fn write_byte() {
        let b = Writer::i24_to_bytes(1);
        assert_eq!(b[0], 1);
        assert_eq!(b[1], 0);
        assert_eq!(b[2], 0);
        let b = Writer::i24_to_bytes(0x1FFFF);
        assert_eq!(b[0], 0xFF);
        assert_eq!(b[1], 0xFF);
        assert_eq!(b[2], 1);
        let b = Writer::i24_to_bytes(-1);
        assert_eq!(b[0], 255);
        assert_eq!(b[1], 255);
        assert_eq!(b[2], 255);
        let b = Writer::i24_to_bytes(-2);
        assert_eq!(b[0], 254);
        assert_eq!(b[1], 255);
        assert_eq!(b[2], 255);
    }

    #[test]
    fn write_to_bytes() {
        let mut samples = Vec::new();
        samples.push(0.0);
        samples.push(0.0);
        let head = WavHeader::new_mono();
        let mut w = Writer::new();
        match w.from_scratch(&head, &samples) {
            Ok(_) => {
                let data = w.to_bytes();
                assert_eq!(data.len(), 52);
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }

        match to_bytes(&head, &samples) {
            Ok(data) => {
                assert_eq!(data.len(), 52);
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
