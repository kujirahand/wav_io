/// Wav file writer

use crate::header::{WavHeader, SampleFormat, WavData};
use std::io::{Cursor, Write, Read};
use std::fs::File;

const ERR_UNSUPPORTED_FORMAT: &str = "unsupported wav format";
const ERR_IO_ERROR: &str = "io error";

/// write wav file
pub fn to_file(file_out: &mut File, wav: &mut WavData) -> Result<(), &'static str> {
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

pub struct Writer {
    cur: Cursor<Vec<u8>>,
}

impl Writer {
    pub fn new() -> Self {
        Self {
            cur: Cursor::new(Vec::<u8>::new())
        }
    }
    pub fn from_scratch(&mut self, head: &WavHeader, samples: &Vec<f32>) -> Result<(), &'static str> {
        let n_bytes = (head.bits_per_sample / 8) as u32;
        let mut samples_len = samples.len();
        let has_pad = samples_len % 2;
        samples_len += has_pad as usize;
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
        // write data header
        self.write_str("data");
        self.write_u32(data_size);
        // write samples
        for v in samples.iter() {
            match head.sample_format {
                SampleFormat::Int => {
                    match head.bits_per_sample {
                        8 => self.write_i8(*v),
                        16 => self.write_i16(*v),
                        24 => self.write_i24(*v),
                        32 => self.write_i32(*v),
                        _ => return Err(ERR_UNSUPPORTED_FORMAT),
                    }
                }, 
                SampleFormat::Float => {
                    match head.bits_per_sample {
                        32 => self.write_f32(*v),
                        64 => self.write_f32(*v), // TODO: 64bit
                        _ => return Err(ERR_UNSUPPORTED_FORMAT),
                    }
                },
                _ => return Err(ERR_UNSUPPORTED_FORMAT),
            }
        }
        Ok(())
    }
    pub fn to_file(&mut self, file: &mut File) -> Result<usize, std::io::Error> {
        let mut data:Vec<u8> = Vec::new();
        self.cur.set_position(0);
        self.cur.read_to_end(&mut data).unwrap();
        file.write(&data)
    }
        
    pub fn write_str(&mut self, tag: &str) {
        let bytes:Vec<u8> = tag.bytes().collect();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_f32(&mut self, v: f32) {
        let bytes = v.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_i24(&mut self, v: f32) {
        let iv:i32 = (v * 2_147_483_648f32) as i32;
        let bytes = iv.to_le_bytes();
        let wb:[u8; 3] = [bytes[1], bytes[2], bytes[3]];
        self.cur.write(&wb).unwrap();
    }
    pub fn write_i16(&mut self, v: f32) {
        let iv:i16 = (v * 65_536f32) as i16;
        let bytes = iv.to_le_bytes();
        self.cur.write(&bytes).unwrap();
    }
    pub fn write_i32(&mut self, v: f32) {
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
}