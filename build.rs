fn main() {
        

    // If debug_assertions are enabled, tell Cargo to enable our custom feature
    if cfg!(debug_assertions) {
        println!("cargo::rustc-cfg=feature=\"file_watcher\"");
    }
}
