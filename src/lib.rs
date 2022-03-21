//! # wav_io
//! wav file reader and writer
//! 
//! ## Example
//! ```
//! use std::fs::File;
//! use wav_io::{reader, writer, utils, header, resample, splitter};
//! fn main() {
//!    // read
//!    let file_in = File::open("./i32.wav").unwrap();
//!    let mut wav = reader::from_file(file_in).unwrap();
//!    println!("header={:?}", wav.header);
//!    println!("samples.len={}", wav.samples.len());
//! 
//!    // write
//!    let mut file_out = File::create("./out.wav").unwrap();
//!    writer::to_file(&mut file_out, &mut wav).unwrap();
//! 
//!    // resample
//!    let new_sample_rate = 8_000;
//!    let mut file_out_16000 = File::create("./test-out8000.wav").unwrap();
//!    let samples2 = resample::linear(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
//!    let mut wav2 = header::WavData{header: wav.header, samples: samples2};
//!    wav2.header.sample_rate = new_sample_rate;
//!    writer::to_file(&mut file_out_16000, &mut wav2).unwrap();
//!    println!("resample.writer={:?}", wav2.header);
//! }
//"" ```

pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod resample;
pub mod splitter;


