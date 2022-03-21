//! # wav_io
//! wav file reader and writer
//! 
//! ## Example
//! ```
//! use std::fs::File;
//! use wav_io::reader;
//! use wav_io::writer;
//! fn main() {
//!    // read
//!    let file_in = File::open("./i32.wav").unwrap();
//!    let mut wav = reader::from_file(file_in).unwrap();
//!    println!("header={:?}", wav.header);
//!    println!("samples.len={}", wav.samples.len());
//!    // write
//!    let mut file_out = File::create("./out.wav").unwrap();
//!    writer::to_file(&mut file_out, &mut wav).unwrap();   
//! }
//"" ```

pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod splitter;

