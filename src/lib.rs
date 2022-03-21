/// # wav_io 
/// This is a crate for reading and writing wav file,
/// it can read 8, 16, 24 bits int, 16, 32 bits float.
/// ## Example
/// ```
/// ```
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
