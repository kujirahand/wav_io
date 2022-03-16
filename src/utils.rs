// convert stereo to mono
pub fn stereo_to_mono(in_v: Vec<f32>) -> Vec<f32> {
    let mut result = vec![];
    let half = in_v.len() / 2;
    for i in 0..half {
        let lv = in_v[i * 2 + 0];
        let rv = in_v[i * 2 + 1];
        result.push((lv + rv) / 2.0);
    }
    result
}
