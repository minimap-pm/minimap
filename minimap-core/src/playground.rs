//! NOTE: This is just a testing playground, it'll be removed in the future.

use minimap_core::*;

fn main() {
	let repo = Remote::open("git@github.com:Qix-/test-minimap.git").unwrap();

	let new_name = std::env::args()
		.nth(1)
		.unwrap_or_else(|| "test".to_string());

	println!("current name: {:?}", repo.name());
	println!("setting name to '{new_name}'");
	repo.set_name(&new_name).unwrap();
	println!("new name: {:?}", repo.name());
}
