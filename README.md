# wav_io 

This is a crate for reading and writing wav file, it can read 8, 16, 24 bits int, 16, 32 bits float.

## Example

```
use std::fs::File;

fn main() {
    // read
    let file_in = File::open("./i32.wav").unwrap();
    let mut wav = reader::read_from_file(file_in).unwrap();
    println!("header={:?}", wav.header);
    println!("samples.len={}", wav.samples.len());
    // write
    let mut file_out = File::create("./out.wav").unwrap();
    writer::write(&mut file_out, &mut wav).unwrap();   
}
```
