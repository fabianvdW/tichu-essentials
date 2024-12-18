pub mod parsing_stats;
pub mod bomb_stats;
pub mod general_stats;

pub fn format_slice_abs_relative(slice: &[usize], divisor: usize) -> String {
    format!("{:?} (Avg. {:.4}, Abs: {:?}, Div: {})", slice.iter().map(|x| format!("{:.4}", 100.0 * (*x as f64 / divisor as f64))).collect::<Vec<_>>(),
            slice.iter().map(|x| 100.0 * (*x as f64 / divisor as f64)).sum::<f64>() / slice.len() as f64,
            slice,
            divisor)
}
pub fn format_slice_abs_relative2(slice: &[usize], divisor: &[usize]) -> String {
    format!("{:?} (Avg. {:.4}, Abs: {:?}, Div: {:?})", slice.iter().zip(divisor).map(|(x, divisor)| format!("{:.4}", 100.0 * (*x as f64 / *divisor as f64))).collect::<Vec<_>>(),
            slice.iter().zip(divisor).map(|(x, divisor)| 100.0 * (*x as f64 / *divisor as f64)).sum::<f64>() / slice.len() as f64,
            slice, divisor)
}
pub fn format_slice_abs_relative2_i64(slice: &[i64], divisor: &[usize]) -> String {
    format!("{:?} (Avg. {:.4}, Abs: {:?}, Div: {:?})", slice.iter().zip(divisor).map(|(x, divisor)| format!("{:.4}",  (*x as f64 / *divisor as f64))).collect::<Vec<_>>(),
            slice.iter().zip(divisor).map(|(x, divisor)|  (*x as f64 / *divisor as f64)).sum::<f64>() / slice.len() as f64,
            slice, divisor)
}