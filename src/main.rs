pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod splitter;

use std::fs::File;

fn main() {
    // read
    let file_in = File::open("./test.wav").unwrap();
    let mut wav = reader::from_file(file_in).unwrap();
    println!("header={:?}", wav.header);
    println!("samples.len={}", wav.samples.len());
    
    // write
    let mut file_out = File::create("./test-out.wav").unwrap();
    writer::to_file(&mut file_out, &mut wav).unwrap(); 
    
    // resample
    let new_sample_rate = 8_000;
    let mut file_out_16000 = File::create("./test-out8000.wav").unwrap();
    let samples2 = utils::resample(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
    let mut wav2 = header::WavData{header: wav.header, samples: samples2};
    wav2.header.sample_rate = new_sample_rate;
    writer::to_file(&mut file_out_16000, &mut wav2).unwrap();
    println!("resample.writer={:?}", wav2.header);

    // split
    let file_in = File::open("./test.wav").unwrap();
    let mut wav = reader::from_file(file_in).unwrap();
    let mut samples = wav.samples;
    if wav.header.channels >= 2 {
        samples = utils::stereo_to_mono(samples);
        wav.header.channels = 1;
    }
    let range_vec = splitter::split_samples(&mut samples, wav.header.sample_rate, true);
    for (i, range) in range_vec.iter().enumerate() {
        let fname = format!("sub-{}.wav", i);
        println!("split_samples={}", fname);
        let mut file_out = File::create(fname).unwrap();
        let sub = splitter::sub_samples(&samples, *range);
        let mut wav = header::WavData{header: wav.header, samples: sub};
        writer::to_file(&mut file_out, &mut wav).unwrap();
    }
}

