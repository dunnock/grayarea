use anyhow::Context;
use serde::Deserialize;
use tokio::fs::read;

/// Pipeline configuration:
///
/// # Example
/// ```yml
/// functions:
///   - name: "send"
///     config: "send.yml"
///   - name: "receive"
///     config: "receive.yml"
/// ```
#[derive(Deserialize)]
pub struct PipelineConfig {
    pub functions: Vec<PipelineModule>,
}

#[derive(Deserialize)]
pub struct PipelineModule {
    pub name: String,
    pub config: std::path::PathBuf,
}

impl PipelineModule {
    pub async fn load_config(&self) -> anyhow::Result<ModuleConfig> {
        let buf = read(self.config.clone())
            .await
            .with_context(|| format!("Could not read config at {:?}", self.config))?;
        Ok(serde_yaml::from_slice(buf.as_slice()).expect("Malformed config"))
    }
}

/// Module configuration:
///
/// # Example
/// ```yml
/// name: "send"
/// kind: "processor"
/// module:
///   path: "send.wasm"
/// args: ["-v"]
/// output:
///   topics:
///     - "topic1"
/// ```
#[derive(Deserialize)]
pub struct ModuleConfig {
    #[serde(default = "empty_args")]
    pub args: Vec<String>,
    pub kind: ModuleKind,
    pub module: Module,
    pub stream: Option<StreamOneOf>,
    pub input: Option<Input>,
    pub output: Option<Output>,
}

#[derive(Deserialize)]
pub enum ModuleKind {
    #[serde(alias = "input")]
    Input,
    #[serde(alias = "processor")]
    Processor,
}

#[derive(Deserialize)]
pub enum StreamOneOf {
    #[serde(alias = "websocket")]
    WebSocket(WebSocketConfig),
}

#[derive(Deserialize)]
pub struct Output {
    pub topics: Vec<String>,
}

#[derive(Deserialize)]
pub struct Input {
    pub topic: String,
}

#[derive(Deserialize)]
pub struct WebSocketConfig {
    pub url: url::Url,
}

#[derive(Deserialize)]
pub enum Module {
    #[serde(alias = "path")]
    Path(std::path::PathBuf),
}

fn empty_args() -> Vec<String> {
    vec![]
}

impl ModuleConfig {
    pub fn args_as_bytes(&self) -> Vec<Vec<u8>> {
        self.args.iter().map(|a| a.as_bytes().to_vec()).collect()
    }

    pub async fn load_wasm_bytes(&self) -> anyhow::Result<Vec<u8>> {
        match &self.module {
            Module::Path(path) => read(path.clone())
                .await
                .with_context(|| format!("Could not read WASM plugin at {:?}", path)),
        }
    }

    pub fn topics(&self) -> anyhow::Result<Vec<String>> {
        self.output
            .as_ref()
            .map(|Output { topics }| topics.clone())
            .ok_or_else(|| anyhow::anyhow!("Missing output topics configuration"))
    }
}
