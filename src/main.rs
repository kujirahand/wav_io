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
}

/*
fn main() {
    iotest();
    // split
    let file_in = File::open("./i32.wav").unwrap();
    let mut wav = reader::read_from_file(file_in).unwrap();
    
    println!("===split=== {:?}", wav.header);
    let sub_vec = splitter::split_samples(&mut wav.samples, wav.header.sample_rate, true);
    for (i, range) in sub_vec.iter().enumerate() {
        let mut w = Writer::new();
        let parts = splitter::sub_samples(&wav.samples, *range);
        w.from_scratch(&wav.header, &parts).unwrap();
        let fname = format!("./sub-{}.wav", i);
        println!("split.saved={}", fname);
        let mut f = File::create(fname).unwrap();
        w.to_file(&mut f).unwrap();
    }
} 

fn iotest() {
    // file_in / file_out
    let file_in = File::open("./i32.wav").unwrap();
    let mut file_out = File::create("./test-out.wav").unwrap();
    
    // read
    println!("hello");
    let mut r =Reader::from_file(file_in).unwrap();
    r.read_header().unwrap();
    println!("{:?}", r.header);
    let samples = r.get_samples_f32().unwrap();
    println!("samples={}", samples.len());

    // write
    let mut w = Writer::new();
    let mut head = r.header.unwrap();
    let samples_mono = utils::stereo_to_mono(samples);

    head.channels = 1;
    head.bits_per_sample = 32;
    head.sample_format = SampleFormat::Int;
    let mut samples2 = utils::resample(samples_mono, head.sample_rate, 44_800);
    head.sample_rate = 32_000;
    w.from_scratch(&head, &samples2).unwrap();
    w.to_file(&mut file_out).unwrap();
}

*/