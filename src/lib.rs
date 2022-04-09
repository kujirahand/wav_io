//! # wav_io
//! Utilities for WAV files 
//! 
//! This crate can read, write, split, resample WAV files
//! 
//! # Supported format
//! - PCM 8, 16, 24, 32 bits Int
//! - PCM 32, 64 bits Float
//! 
//! # Functoins
//! - read & write
//! - resample
//! - split by silence
//! - make sine wave & MML (music macro language)
//! 
//! ## Quick Start
//! 
//! Write Wav file:
//! 
//! ```
//! use std::fs::File;
//! use std::f32::consts::PI;
//! use wav_io::{writer, header::*};
//! fn main() {
//!     // header
//!     let head = WavHeader::new_mono();
//!     // make sine wave
//!     let mut samples: Vec<f32> = vec![];
//!     for t in 0..head.sample_rate {
//!         let v = ((t as f32 / head.sample_rate as f32) * 440.0 * 2.0 * PI).sin() * 0.6;
//!         samples.push(v);
//!     }
//!     // write to file
//!     let mut file_out = File::create("./sine.wav").unwrap();
//!     writer::f32samples_to_file(&mut file_out, &head, &samples).unwrap();
//! }
//! ```
//! 
//! Read Wav file:
//! 
//! ```
//! use std::fs::File;
//! use std::f32::consts::PI;
//! use wav_io::{reader, header::*};
//! fn main() {
//!     // open file
//!     let file_in = File::open("./sine.wav").unwrap();
//!     // read from file
//!     let wav = reader::from_file(file_in).unwrap();
//!     // show header info
//!     println!("header={:?}", wav.header);
//!     // show samples
//!     println!("samples.len={}", wav.samples.len());
//!     for (i, v) in wav.samples.iter().enumerate() {
//!         println!("{}: {}v", i, v);
//!         if (i > 32) { break; } // show first 32 samples
//!     }
//! }
//! ```
//! 
//! ## Other Example
//! ```
//! use std::fs::File;
//! use std::f32::consts::PI;
//! use wav_io::{reader, writer, utils, resample, splitter, header::*, tone};
//! fn main() {
//!     // write tone
//!     let header = WavHeader::new_mono();
//!     let mut samples = vec![];
//!     for t in 0..header.sample_rate {
//!         let v = ((t as f32 / header.sample_rate as f32) * 440.0 * 2.0 * PI).sin() * 0.6;
//!         samples.push(v);
//!     }
//!     let mut file_out = File::create("./tone.wav").unwrap();
//!     writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
//!     
//!     // read wav file
//!     let file_in = File::open("./tone.wav").unwrap();
//!     let wav = reader::from_file(file_in).unwrap();
//!     println!("header={:?}", wav.header);
//!     println!("samples.len={}", wav.samples.len());
//!     
//!     // resample
//!     let file_in = File::open("./tone.wav").unwrap();
//!     let wav = reader::from_file(file_in).unwrap();
//!     let new_sample_rate = 16_000;
//!     let mut file_out = File::create("./tone-resample.wav").unwrap();
//!     let samples2 = resample::linear(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
//!     let mut wav2 = WavData{header: wav.header, samples: samples2};
//!     wav2.header.sample_rate = new_sample_rate;
//!     writer::to_file(&mut file_out, &wav2).unwrap();
//!     
//!     // melody
//!     let mut header = WavHeader::new_mono();
//!     let mut samples = vec![];
//!     let opt = tone::ToneOptions::new();
//!     header.sample_rate = opt.sample_rate;
//!     tone::write_mml(&mut samples, "l4 cegr rdfr egrr c1", &opt);
//!     let mut file_out = File::create("./melody.wav").unwrap();
//!     writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
//! 
//!     // split
//!    let file_in = File::open("./melody.wav").unwrap();
//!    let mut wav = reader::from_file(file_in).unwrap();
//!    let mut samples = wav.samples;
//!    if wav.header.channels >= 2 {
//!        samples = utils::stereo_to_mono(samples);
//!        wav.header.channels = 1;
//!    }
//!    let range_vec = splitter::split_samples(&mut samples, wav.header.sample_rate, &splitter::WavSplitOption::new());
//!    for (i, range) in range_vec.iter().enumerate() {
//!        let fname = format!("sub-{}.wav", i);
//!        println!("split_samples={}", fname);
//!        let mut file_out = File::create(fname).unwrap();
//!        let sub = splitter::sub_samples(&samples, *range);
//!        let wav = WavData{header: wav.header, samples: sub};
//!        writer::to_file(&mut file_out, &wav).unwrap();
//!    }
//! }
//"" ```

/// Wav file header
pub mod header;
/// Wav file Reader
pub mod reader;
/// Wav file Writer
pub mod writer;
/// Wav file Resampler
pub mod resample;
/// Wav file Splitter
pub mod splitter;
/// Tone Generator
pub mod tone;
/// Utilities
pub mod utils;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::f32::consts::PI;
    use header::*;
    
    #[test]
    fn write_tone() {
        // write tone
        let header = WavHeader::new_mono();
        let mut samples = vec![];
        for t in 0..header.sample_rate {
            let v = ((t as f32 / header.sample_rate as f32) * 440.0 * 2.0 * PI).sin() * 0.6;
            samples.push(v);
        }
        let samples_len = samples.len();
        let mut file_out = File::create("./tone.wav").unwrap();
        writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
        assert_eq!(samples_len, header.sample_rate as usize);

        // melody
        let mut header = WavHeader::new_mono();
        let mut samples = vec![];
        let opt = tone::ToneOptions::new();
        header.sample_rate = opt.sample_rate;
        tone::write_mml(&mut samples, "l4 crer g1", &opt);
        let mut file_out = File::create("./melody.wav").unwrap();
        writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
    }

    #[test]
    fn read_write() {
        // path
        let wavfile = "./tone.wav";
        if !std::path::Path::new(wavfile).exists() { return; }
        // read
        let file_in = File::open(wavfile).unwrap();
        let wav = reader::from_file(file_in).unwrap();
        assert_eq!(wav.header.channels, 1); // mono
        let time_sec = wav.samples.len() as f32 / wav.header.sample_rate as f32;
        assert_eq!(time_sec, 1.0);
        // println!("header={:?}", wav.header);
        // println!("samples.len={}", wav.samples.len());
        
        // write
        let wavfile2 = "./tone2.wav";
        let mut file_out = File::create(wavfile2).unwrap();
        writer::to_file(&mut file_out, &wav).unwrap();

        // read again
        let file_in = File::open(wavfile2).unwrap();
        let wav = reader::from_file(file_in).unwrap();
        assert_eq!(wav.header.channels, 1); // mono
        let time_sec = wav.samples.len() as f32 / wav.header.sample_rate as f32;
        assert_eq!(time_sec, 1.0);
    }

    #[test]
    fn resample() {    
        let wavfile = "./tone.wav";
        let wavfile2 = "./tone-resample.wav";
        if !std::path::Path::new(wavfile).exists() { return; }
        // resample
        let file_in = File::open(wavfile).unwrap();
        let wav = reader::from_file(file_in).unwrap();
        let new_sample_rate = 16_000;
        let mut file_out = File::create(wavfile2).unwrap();
        let samples2 = resample::linear(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
        let mut wav2 = header::WavData{header: wav.header, samples: samples2};
        wav2.header.sample_rate = new_sample_rate;
        writer::to_file(&mut file_out, &wav2).unwrap();
        
        // read and test
        let file_in = File::open(wavfile2).unwrap();
        let wav = reader::from_file(file_in).unwrap();
        assert_eq!(wav.header.channels, 1); // mono
        assert_eq!(wav.header.sample_rate, new_sample_rate); // sample_rate
    }

    #[test]
    fn split() {
        // split
        let wavfile = "./melody.wav";
        // write melody
        let mut header = WavHeader::new_mono();
        let mut samples = vec![];
        let opt = tone::ToneOptions::new();
        header.sample_rate = opt.sample_rate;
        tone::write_mml(&mut samples, "l1 grrr errr c1", &opt);
        let mut file_out = File::create(wavfile).unwrap();
        writer::to_file(&mut file_out, &WavData{header, samples}).unwrap();
        // read melody
        let file_in = File::open(wavfile).unwrap();
        let mut wav = reader::from_file(file_in).unwrap();
        let mut samples = wav.samples;
        // convert to mono
        if wav.header.channels >= 2 {
            samples = utils::stereo_to_mono(samples);
            wav.header.channels = 1;
        }
        assert_eq!(samples.len() > 100, true);
        // split
        let opt = splitter::WavSplitOption::new();
        let range_vec = splitter::split_samples(&mut samples, wav.header.sample_rate, &opt);
        println!("{:?}", range_vec);
        assert_eq!(range_vec.len(), 3);
        // write to file
        for (i, range) in range_vec.iter().enumerate() {
            let fname = format!("./sub-{}.wav", i);
            println!("split_samples={}", fname);
            let mut file_out = File::create(fname).unwrap();
            let sub = splitter::sub_samples(&samples, *range);
            let wav = header::WavData{header: wav.header, samples: sub};
            writer::to_file(&mut file_out, &wav).unwrap();
        }
    }
}
