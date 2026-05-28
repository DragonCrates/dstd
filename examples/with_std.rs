// TODOs for this example:
// 1) panic handler and global allocator should be optional
// Preferable solution - activated by dstd main, but could be a feature flag
fn main() {
    std::println!("This is print from std");
    dstd::println!("This is print from dstd");
}
