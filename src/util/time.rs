pub fn format_seconds_f64(seconds: f64, decimal_places: usize) -> String {
    const SECONDS_PER_DAY: f64 = 86400.0;
    const SECONDS_PER_HOUR: f64 = 3600.0;
    const SECONDS_PER_MINUTE: f64 = 60.0;
    let mut s = String::new();
    let negative = seconds < 0.0;
    let total_seconds = seconds.abs();
    let days = (total_seconds / SECONDS_PER_DAY).floor() as u64;
    let hours = ((total_seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR).floor() as u64;
    let minutes = ((total_seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE).floor() as u64;
    let seconds = total_seconds % SECONDS_PER_MINUTE;
    if negative {
        s.push('-');
    }
    if days > 0 {
        s += &format!("{}d ", days);
    }
    if hours > 0 {
        s += &format!("{}h ", hours);
    }
    if minutes > 0 {
        s += &format!("{}m ", minutes);
    }
    s += &format!("{:.*}s", decimal_places, seconds);
    s
}
