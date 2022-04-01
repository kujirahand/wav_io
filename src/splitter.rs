/// Wav file Splitter

#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitOption {
    pub use_margin: bool,
    pub margin_sec: f32,
    pub min_silence_level: f32,
    pub min_silence_duration: f32,
}
impl WavSplitOption {
    pub fn new() -> Self {
        Self {
            use_margin: true,
            margin_sec: 0.1,
            min_silence_level: 0.05, // 0.05
            min_silence_duration: 0.5, // about 0.4 - 0.7
        }
    }
}


/// normalize samples
pub fn normalize(samples: &mut Vec<f32>) {
    let mut max = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; } 
    }
    let r = 1.0 / max;
    for i in 0..samples.len() {
        samples[i] = r * samples[i];
    }
}

fn get_max(a: isize, b: isize) -> isize {
    if a > b { a } else { b }
}

/// Status for Splitter
#[derive(Debug,Clone,PartialEq)]
enum SplitStatus {
    FirstSilence,
    Silence,
    LoudSound,
    FindingSilence,
}

/// split wave data
pub fn split_samples(samples: &mut Vec<f32>, sample_rate: u32, opt: &WavSplitOption) -> Vec<WavSplitRange> {
    let th_silence = opt.min_silence_level;
    let min_silence_duration = opt.min_silence_duration;

    let mut result_vec = vec![];

    let mut max_val = 0.0;
    let mut si_start = 0;
    let mut last = 0;
    let min_length = ((30.0 / 441000.0) * sample_rate as f32) as usize;

    let mut go_split = |i_begin: usize, i_end: usize| {
        let i_margin = if opt.use_margin {
            (opt.margin_sec * sample_rate as f32).floor() as usize
        } else { 0 };
        // calc margin time
        let i_begin_margin = get_max(0, i_begin as isize - i_margin as isize) as usize;
        let i_end_margin = i_end;
        let new_size = i_end_margin - i_begin_margin + 1;
        if new_size < min_length { return }
        // println!("split={}", i_begin);
        // result
        let r = WavSplitRange {
            start: i_begin_margin,
            end: i_end_margin,
            // start_nanotime: i_begin_margin / sample_rate as usize,
            // end_nanotime: i_end_margin / sample_rate as usize,
        };
        result_vec.push(r);
    };

    normalize(samples);

    let mut status: SplitStatus = SplitStatus::FirstSilence;
    
    for (i, v) in samples.iter().enumerate() {
        let av = v.abs();
        if av > max_val { max_val = av; }
        
        match status {
            SplitStatus::FirstSilence => {
                // silence ended?
                if av > th_silence {
                    //println!("First silence end={}", i);
                    last = i;
                    status = SplitStatus::LoudSound;
                }
            },
            SplitStatus::Silence => {
                // silence ended?
                if av > th_silence {
                    //println!("silence end={}", i);
                    status = SplitStatus::LoudSound;
                }
            },
            SplitStatus::LoudSound => {
                // Find silence start
                if av < th_silence {
                    si_start = i;
                    status = SplitStatus::FindingSilence;
                }
            },
            SplitStatus::FindingSilence => {
                // Check silence length
                if av > th_silence {
                    let duration = (i - si_start) as f32 / sample_rate as f32;
                    if duration > min_silence_duration {
                        go_split(last, i - 1);
                        last = i;
                        status = SplitStatus::Silence;
                        si_start = 0;
                    }
                }
            }
        }
        //println!("@{:02}:status={:?}", i, status);
    }
    // last wav?
    if (samples.len() - last) > min_length {
        go_split(last, samples.len() - 1);
    }

    result_vec
}

pub fn sub_samples(samples: &Vec<f32>, range: WavSplitRange) -> Vec<f32> {
    let mut result = Vec::with_capacity(range.end - range.start);
    for i in range.start..=range.end {
        let v = samples[i];
        result.push(v);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_normalize() {
        let mut f1 = vec![0.2, 0.6, 2.0];
        normalize(&mut f1);
        assert_eq!(f1, vec![0.1, 0.3, 1.0]);
    }

    #[test]
    fn test_split1() {
        let mut samples = vec![0.0,0.0,0.0, 1.0,0.9,0.8];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 1);
        let part = sub_samples(&samples, res[0]);
        assert_eq!(part, vec![1.0,0.9,0.8]);
    }

    #[test]
    fn test_split2() {
        let mut samples = vec![0.0,0.0,0.0, 0.8,0.8,0.8, 0.0,0.0,0.0, 0.8,0.8,0.4];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 2);
        if res.len() >= 2 {
            let part = sub_samples(&samples, res[0]);
            assert_eq!(part, vec![1.0, 1.0, 1.0, 0.0,0.0,0.0]);
            let part = sub_samples(&samples, res[1]);
            assert_eq!(part, vec![1.0, 1.0, 0.5]);
        }
    }

    #[test]
    fn test_split3() {
        let mut samples = vec![0.0, 0.8,0.8,0.8, 0.0,0.0, 0.8,0.8,0.4];
        let res = split_samples(&mut samples, 3, &WavSplitOption::new());
        assert_eq!(res.len(), 2);
        if res.len() >= 2 {
            let part = sub_samples(&samples, res[0]);
            assert_eq!(part, vec![1.0, 1.0, 1.0, 0.0,0.0]);
            let part = sub_samples(&samples, res[1]);
            assert_eq!(part, vec![1.0, 1.0, 0.5]);
        }
    }
}
