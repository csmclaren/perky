pub fn format_perc(decimal_places: usize, opt_value: Option<f64>) -> String {
    opt_value.map_or_else(
        || "n/a".to_string(),
        |value| format!("{:.*}%", decimal_places, value),
    )
}
