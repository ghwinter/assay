//! crash_demo —— 故意崩溃的示例目标，用于验证崩溃证据自动采集（O-022，D-011）
fn main() {
    println!("about to abort");
    std::process::abort();
}
