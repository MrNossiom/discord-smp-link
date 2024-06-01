//! Build script

fn main() {
	println!("cargo:rerun-if-changed=migrations");
}
