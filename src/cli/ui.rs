use crate::doctor::checks::CheckStatus;

pub fn header(title: &str) {
    println!("== {} ==", title);
}

pub fn info(message: impl AsRef<str>) {
    println!("[i] {}", message.as_ref());
}

pub fn step(message: impl AsRef<str>) {
    println!("[>] {}", message.as_ref());
}

pub fn ok(message: impl AsRef<str>) {
    println!("[✓] {}", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    println!("[!] {}", message.as_ref());
}

pub fn doctor_mark(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "PASS",
        CheckStatus::Warn => "WARN",
        CheckStatus::Fail => "FAIL",
    }
}
