pub fn format_perc(decimal_places: usize, value_opt: Option<f64>) -> String {
    value_opt.map_or_else(
        || "n/a".to_string(),
        |value| format!("{:.*}%", decimal_places, value),
    )
}
