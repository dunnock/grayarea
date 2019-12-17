use serde::{Deserialize};

#[derive(Deserialize)]
pub struct PipelineConfig {
	pub stages: Vec<Stage>,
}

#[derive(Deserialize)]
pub struct Stage {
	pub name: String,
	pub config: std::path::PathBuf
}
