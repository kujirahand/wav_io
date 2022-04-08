/// Wav file Splitter
#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug,Copy, Clone,PartialEq)]
pub struct WavSplitOption {
    pub is_debug: bool,
    pub min_silence_level: f32,
    pub min_silence_duration: f32,
    pub min_keep_duration: f32,
}
impl WavSplitOption {
    pub fn new() -> Self {
        Self {
            is_debug: true,
            min_silence_level: 0.25, // 0.25,
            min_silence_duration: 0.07,
            min_keep_duration: 0.01,
        }
    }
}

/// normalize samples
pub fn normalize_i(samples: &mut Vec<f32>) -> Vec<i16> {
    let mut result: Vec<i16> = Vec::with_capacity(samples.len());
    let imax = std::i16::MAX as f32;
    // get max value
    let mut max: f32 = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; }
    }
    // mul
    let r = 1.0 / max as f32;
    for i in 0..samples.len() {
        let v = r * samples[i];
        let iv = (v * imax) as i16;
        result.push(iv);
    }
    result
}

pub fn normalize_f(samples: &Vec<f32>) -> Vec<f32> {
    let mut result: Vec<f32> = Vec::with_capacity(samples.len());
    let mut max = 0.0;
    for v in samples.iter() {
        let av = v.abs();
        if av > max { max = av; } 
    }
    let r = 1.0 / max;
    for i in 0..samples.len() {
        let v = r * samples[i];
        result.push(v);
    }
    result
}

fn get_max(a: isize, b: isize) -> isize {
    if a > b { a } else { b }
}
fn get_min(a: isize, b: isize) -> isize {
    if a < b { a } else { b }
}

/// split wave data
pub fn split_samples(samples: &mut Vec<f32>, sample_rate: u32, opt: &WavSplitOption) -> Vec<WavSplitRange> {
    let silence_thresh = opt.min_silence_level;
    let min_silence_len = (sample_rate as f32 * opt.min_silence_duration) as usize;
    let min_keep_len = (sample_rate as f32 * opt.min_keep_duration) as usize;
    let mut result_vec = vec![];
    let samples_len = samples.len();
    if opt.is_debug {
        println!("silence_thresh={}",silence_thresh);
        println!("min_silence_len={}",min_silence_len);
        println!("min_keep_len={}",min_keep_len);
    }
    // normalize
    let samples = normalize_i(samples);
    if opt.is_debug {
        println!("normalized={}", samples.len());
    }
    let mut out_ranges = detect_nonsilent(&samples, min_silence_len, silence_thresh, opt.is_debug);
    if out_ranges.len() == 0 {
        return vec![WavSplitRange{start:0, end:samples_len}];
    }
    if opt.is_debug { println!("detect_nonsilent/len={}", out_ranges.len()); }
    // check margin
    for i in 0..out_ranges.len() {
        out_ranges[i].0 = get_max(0, out_ranges[i].0 as isize - min_keep_len as isize) as usize;
        out_ranges[i].1 = get_min(samples_len as isize, out_ranges[i].1 as isize + min_keep_len as isize) as usize;
    }
    if opt.is_debug { println!("checked margin"); }
    for i in 0..(out_ranges.len() - 1) {
        let last_end = out_ranges[i].1;
        let next_start = out_ranges[i+1].0;
        if next_start < last_end {
            out_ranges[i+0].1 = (last_end + next_start) / 2;
            out_ranges[i+1].0 = out_ranges[i+0].1;
        }
    }
    for r in out_ranges.iter() {
        result_vec.push(WavSplitRange{
            start: r.0,
            end: r.1,
        });
        if opt.is_debug {
            println!("- split={}s to {}s", r.0 as f32 / 11600.0, r.1 as f32 / 11600.0);
        }
    }
    if opt.is_debug { println!("result={:?}", result_vec); }
    result_vec
}

#[allow(dead_code)]
pub fn calc_rms2_prepare(samples: &Vec<i32>) -> Vec<i32> {
    let mut result: Vec<i32> = Vec::with_capacity(samples.len());
    for i in 0..samples.len() {
        let v:i32 = samples[i];
        let vv:i32 = v.wrapping_mul(v);
        result.push(vv);
    }
    result
}

#[allow(dead_code)]
fn calc_rms2_i(samples: &Vec<i32>, start:usize, size: usize) -> f32 {
    let last = start + size;
    let mut total: isize = 0;
    for i in start..last {
        total += samples[i] as isize;
    }
    if total == 0 { return 0.0; }
    ((total as usize / size) as f32).sqrt()
}

#[allow(dead_code)]
fn calc_rms(samples: &Vec<i16>, start:usize, size: usize) -> i16 {
    let last = start + size;
    let mut total: isize = 0;
    for i in start..last {
        let v = samples[i] as isize;
        let v2 = v * v;
        total += v2;
    }
    ((total / size as isize) as f32).sqrt() as i16
}

fn detect_silence(samples: &Vec<i16>, min_silence_len: usize, silence_thresh: f32, is_debug: bool) -> Vec<(usize,usize)> {
    let mut result: Vec<(usize, usize)> = vec![];
    let samples_len = samples.len();
    if samples_len < min_silence_len { return result; }
    let last_silene_start = samples_len - min_silence_len;
    let check_range = 0..=last_silene_start;
    let step = 2;
    let silence_thresh_i = (silence_thresh * std::i16::MAX as f32) as i16;
    if is_debug { println!("@@ detect_silence/silence_thresh_i={}/{}", silence_thresh_i, std::i16::MAX); }
    let mut silence_starts:Vec<usize> = Vec::with_capacity(samples_len);
    //
    for i in check_range {
        if i % step != 0 { continue; }
        let rms = calc_rms(&samples, i, min_silence_len);
        if rms <= silence_thresh_i {
            silence_starts.push(i);
            // if is_debug { println!("@@ silence={} : {}/{}", i, rms, silence_thresh_i); }
        }
    }
    if silence_starts.len() == 0 { return result; }
    if is_debug { println!("@@ calc_rms.end/len={}", silence_starts.len()); }
    
    let mut prev_i = silence_starts[0];
    let mut cur_range_start = prev_i;
    for i in 1..silence_starts.len() {
        let silent_i = silence_starts[i];
        let is_continue = silent_i == prev_i + 1;
        let silence_has_gap = silent_i > (prev_i + min_silence_len);
        if (!is_continue) && silence_has_gap {
            result.push((cur_range_start, prev_i + min_silence_len));
            cur_range_start = silent_i;
        }
        prev_i = silent_i;
    }
    result.push((cur_range_start, prev_i + min_silence_len));
    if is_debug {
        for r in result.iter() {
            println!("- detect_silence/split={}s to {}s", r.0 as f32 / 11600.0, r.1 as f32 / 11600.0);
        }
    }
    result
}

fn detect_nonsilent(samples: &Vec<i16>, min_silence_len: usize, silence_thresh: f32, is_debug: bool) -> Vec<(usize, usize)> {
    let mut result: Vec<(usize, usize)> = vec![];
    let samples_len = samples.len();
    if is_debug { println!("@@detect_silent.begin"); }
    let silent_ranges = detect_silence(samples, min_silence_len, silence_thresh, is_debug);
    if silent_ranges.len() == 0 {
        result.push((0, samples_len));
        return result;
    }
    if is_debug { println!("@@detect_silent.end"); }
    // whole audio is silent?
    if silent_ranges[0].0 == 0 && silent_ranges[0].1 == samples_len {
        return result;
    }
    let mut prev_end_i = 0;
    let mut last_end_i = 0;
    for (start_i, end_i) in silent_ranges.iter() {
        result.push((prev_end_i, *start_i));
        prev_end_i = *end_i;
        last_end_i = *end_i;
    }
    if last_end_i != samples_len {
        result.push((prev_end_i, samples_len));
    }
    if result[0].0 == 0 && result[0].1 == 0 {
        result.remove(0);
    }
    result
}


pub fn sub_samples(samples: &Vec<f32>, range: WavSplitRange) -> Vec<f32> {
    let mut result = Vec::with_capacity(range.end - range.start);
    for i in range.start..range.end {
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
        let f1 = vec![0.2, 0.6, 2.0];
        let f2 = normalize_f(&f1);
        assert_eq!(f2, vec![0.1, 0.3, 1.0]);
    }
/*
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
*/
}
