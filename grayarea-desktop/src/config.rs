use serde::{Deserialize};

#[derive(Deserialize)]
pub struct PipelineConfig {
	pub functions: Vec<Module>,
}

#[derive(Deserialize)]
pub struct Module {
	pub name: String,
	pub config: std::path::PathBuf
}
