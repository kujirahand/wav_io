/// resample
use crate::utils;

/// resample audio sample rate
pub fn linear(samples: Vec<f32>, channels: u16, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    // same rate
    if cur_rate == new_rate {
        return samples.clone();
    }
    // check channels
    if channels == 1 {
        if cur_rate < new_rate {
            return linear_upsamle(samples, cur_rate, new_rate);
        } else {
            return linear_downsample(samples, cur_rate, new_rate);
        }
    }
    let (mut l_samples, mut r_samples) = utils::split_stereo_wave(samples);
    l_samples = linear(l_samples, 1, cur_rate, new_rate);
    r_samples = linear(r_samples, 1, cur_rate, new_rate);
    utils::join_stereo_wave(l_samples, r_samples)
}

fn linear_upsamle(samples: Vec<f32>, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    let scale = cur_rate as f32 / new_rate as f32;
    let cur_size = samples.len();
    let new_size = (cur_size as f32 / scale) as usize;
    let mut output = Vec::with_capacity(new_size);
    let mut pos = 0.0;
    for _i in 0..new_size {
        let mut in_pos = pos as usize;
        let mut prop = pos - in_pos as f32;
        if in_pos >= cur_size - 1 {
            in_pos = cur_size - 2;
            prop = 1.0;
        }
        let v = samples[in_pos] * (1.0 - prop) + samples[in_pos + 1] * prop;
        output.push(v);
        pos += scale;
    }
    output
}
fn linear_downsample(samples: Vec<f32>, cur_rate: u32, new_rate: u32) -> Vec<f32> {
    let scale = new_rate as f32 / cur_rate as f32;
    let cur_size = samples.len();
    let new_size = (cur_size as f32 / scale) as usize;
    let mut output = Vec::with_capacity(new_size);
    let mut in_pos = 0;
    let mut out_pos = 0;
    let mut pos = 0.0;
    let mut sum = 0.0;
    while out_pos < new_size {
        if in_pos >= cur_size { break; }
        let val = samples[in_pos];
        in_pos += 1;
        let mut next_pos = pos + scale;
        if next_pos >= 1.0 {
            sum += val * (1.0 - pos);
            output.push(sum);
            next_pos -= 1.0;
            sum = next_pos * val;
        } else {
            sum += scale * val;
        }
        pos = next_pos;
        if (in_pos >= cur_size) && (out_pos < new_size) {
            output.push(sum / pos);
            out_pos += 1;
        }
    }
    output
}
