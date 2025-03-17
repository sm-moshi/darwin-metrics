use darwin_metrics::utils::ffi::SmcKey;

fn main() {
    let key = SmcKey::from_chars(['T', 'A', '0', 'P']);
    println!("SmcKey: {}", key.to_string());
    assert_eq!(key.to_string(), "TA0P");
    println!("Test passed!");
}
