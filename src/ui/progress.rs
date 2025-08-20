use core::iter;

pub fn create_progress_bar(length: usize, value: f32) -> String {
    const BLOCKS: [char; 9] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];
    const NUM_BLOCKS: usize = BLOCKS.len() - 1;
    let progress_in_blocks = value.clamp(0.0, 1.0) * (length as f32);
    let full_blocks = progress_in_blocks.floor() as usize;
    let partial_blocks = progress_in_blocks - full_blocks as f32;
    let index = (partial_blocks * NUM_BLOCKS as f32).round() as usize;
    let mut s = String::with_capacity(length);
    s.extend(iter::repeat('█').take(full_blocks));
    if full_blocks < length {
        s.push(BLOCKS[index]);
        s.extend(iter::repeat(' ').take(length - full_blocks - 1));
    }
    s
}
