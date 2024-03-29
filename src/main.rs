pub mod header;
pub mod reader;
pub mod writer;
pub mod utils;
pub mod resample;
pub mod splitter;
pub mod tone;

#[derive(Debug,Clone,PartialEq)]
pub struct CommandOpt {
    pub is_debug: bool,
    pub command: String,
    pub filename: Option<String>,
    pub arg1: Option<String>,
    pub arg2: Option<String>,
}

fn main() {
    let mut cmd = CommandOpt{
        is_debug: false,
        command: String::from("?"),
        filename: None,
        arg1: None,
        arg2: None,
    };
    for (i, s) in std::env::args().enumerate() {
        if i == 0 { continue; }
        // command
        if i == 1 {
            cmd.command = String::from(s);
            continue;
        }
        // debug
        if s == "--debug" || s == "-d" {
            cmd.is_debug = true;
            continue;
        }
        // filename
        if cmd.filename == None {
            cmd.filename = Some(String::from(s));
            continue;
        }
        // arg
        if cmd.arg1 == None {
            cmd.arg1 = Some(String::from(s));
            continue;
        }
        if cmd.arg2 == None {
            cmd.arg2 = Some(String::from(s));
            continue;
        }
    }
    if cmd.command == "info" {
        return command_info(cmd);
    }
    if cmd.command == "mml" {
        return command_mml(cmd);
    }
    if cmd.command == "split" {
        return command_split(cmd);
    }
    if cmd.command == "resample" {
        return command_resample(cmd);
    }
    if cmd.command == "?" || cmd.command == "help" {
        return show_help();
    }
    show_help();
}

fn command_info(cmd: CommandOpt) {
    if cmd.is_debug {
        println!("{:?}", cmd);
    }
    if cmd.filename == None {
        println!("[Usage] wav_io info [file]");
        return;
    }
    let wav = match reader::from_file_str(&cmd.filename.unwrap()) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("[Error] {}", e);
            return;
        }
    };
    println!("header={:?}", wav.header);
    println!("samples.len={}", wav.samples.len());
 }

 fn command_mml(cmd: CommandOpt) {
    if cmd.is_debug {
        println!("{:?}", cmd);
    }
    if cmd.filename == None || cmd.arg1 == None {
        println!("[Usage] wav_io mml [file] [mml]");
        return;
    }
    // melody
    let mut header = header::WavHeader::new_mono();
    let mut samples = vec![];
    let opt = tone::ToneOptions::new();
    header.sample_rate = opt.sample_rate;
    tone::write_mml(&mut samples, &cmd.arg1.unwrap(), &opt);
    let time_s = samples.len() as f32 / opt.sample_rate as f32;
    let mut file_out = std::fs::File::create(cmd.filename.unwrap()).unwrap();
    writer::to_file(&mut file_out, &header::WavData{header, samples}).unwrap();
    println!("mml.sec={}", time_s);
 }

 use std::io::Write;
 fn command_split(cmd: CommandOpt) {
    if cmd.is_debug {
        println!("{:?}", cmd);
    }
    if cmd.filename == None || cmd.arg1 == None {
        println!("[Usage] wav_io split [file] [outdir]");
        return;
    }
    // get path
    let wavfile = cmd.filename.unwrap();
    let outdir = cmd.arg1.unwrap();
    // mkdir?
    if !std::path::Path::new(&outdir).exists() {
        match std::fs::create_dir(&outdir) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("[Error] Could not mkdir: {:?}", e);
                return;
            }
        }
    }
    // read in file
    let file_in = std::fs::File::open(wavfile).unwrap();
    let mut wav = reader::from_file(file_in).unwrap();
    println!("header={:?}", wav.header);
    // convert to mono
    let mut samples = wav.samples;
    if wav.header.channels >= 2 {
        samples = utils::stereo_to_mono(samples);
        wav.header.channels = 1;
    }
    // split
    let opt = splitter::WavSplitOption::new();
    let range_vec = splitter::split_samples(&mut samples, wav.header.sample_rate, &opt);
    let mut json = String::from("[");
    // save to dir
    for (i, range) in range_vec.iter().enumerate() {
        let name = format!("{}.wav", i);
        let fname = format!("{}/{}", &outdir, &name);
        println!("- {}", fname);
        let mut file_out = std::fs::File::create(&fname).unwrap();
        let sub = splitter::sub_samples(&samples, *range);
        let wav = header::WavData{header: wav.header.clone(), samples: sub};
        writer::to_file(&mut file_out, &wav).unwrap();
        // add to object
        json += &format!("{{ \"no\":{}, \"file\":\"{}\", \"start\":{}, \"end\":{} }}", i, name, range.start, range.end);
        if i != range_vec.len() - 1 { json += ",\n"; } else { json += "\n"; }
    }
    json += "]";
    // save split info
    let jsonfile = &format!("{}/split.json", outdir);
    let mut file = std::fs::File::create(jsonfile).unwrap();
    write!(file, "{}", json).unwrap();
}

fn command_resample(cmd: CommandOpt) {
    if cmd.is_debug {
        println!("{:?}", cmd);
    }
    if cmd.filename == None || cmd.arg1 == None || cmd.arg2 == None {
        println!("[Usage] wav_io resample [in] [rate] [out]");
        return;
    }
    // get path
    let infile = cmd.filename.unwrap();
    let rate_str = cmd.arg1.unwrap();
    let outfile = cmd.arg2.unwrap();
    let new_sample_rate:u32 = rate_str.parse().unwrap_or(0);
    if new_sample_rate < 100 {
        println!("[Error] sample_rate must be over 100");
        return;
    }
    let file_in = std::fs::File::open(infile).unwrap();
    let mut wav = reader::from_file(file_in).unwrap();
    let mut file_out = std::fs::File::create(outfile).unwrap();
    if wav.samples.len() == 0 {
        println!("[Error] resample failed.");
        return;
    }
    let samples = resample::linear(wav.samples, wav.header.channels, wav.header.sample_rate, new_sample_rate);
    wav.header.sample_rate = new_sample_rate;
    writer::to_file(&mut file_out, &header::WavData{header:wav.header, samples}).unwrap();
    println!("ok.");
}

fn show_help() {
    println!("*--- * --- * --- * --- * ---*");
    println!("| wav_io <command>");
    println!("*--- * --- * --- * --- * ---*");
    println!("[Usage]");
    println!("wav_io help                  show help");
    println!("wav_io info [file]           show file info");
    println!("wav_io mml [file] [mml]      write melody by mml");
    println!("wav_io split [file] [outdir] split wav by silence");
    println!("wav_io resample [in] [rate] [out] resample av file");
}

