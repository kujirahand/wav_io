pub mod header;
pub mod reader;
pub mod writer;

use std::fs::File;
use writer::Writer;
use reader::Reader;
use header::*;

fn main() {
    // file_in / file_out
    let file_in = File::open("./i32.wav").unwrap();
    let mut file_out = File::create("./test-out.wav").unwrap();
    
    // read
    println!("hello");
    let mut r =Reader::from_file(file_in).unwrap();
    r.read_header().unwrap();
    println!("{:?}", r.header);
    let samples = r.get_samples_f32(true);
    println!("samples={}", samples.len());

    // write
    let mut w = Writer::new();
    let mut head = r.header.unwrap();
    head.bits_per_sample = 24;
    head.sample_format = SampleFormat::Int;
    w.from_scratch(&head, &samples).unwrap();
    w.to_file(&mut file_out).unwrap();
} 

