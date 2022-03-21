//! # wav_io
//! wav file reader and writer
//! 
//! ## Example
//! ```
//! use std::fs::File;
//! fn main() {
//!    // read
//!    let file_in = File::open("./i32.wav").unwrap();
//!    let mut wav = reader::read_from_file(file_in).unwrap();
//!    println!("header={:?}", wav.header);
//!    println!("samples.len={}", wav.samples.len());
//!    // write
//!    let mut file_out = File::create("./out.wav").unwrap();
//!    writer::write(&mut file_out, &mut wav).unwrap();   
//! }
//"" ```

pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod splitter;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    #[test]
    fn it_works() {
        let _ =reader::Reader::from_file(File::open("./test.wav").unwrap());
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
