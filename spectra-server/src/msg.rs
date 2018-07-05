use std::path::PathBuf;

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Msg {
  /// Close the server.
  Close,
  /// Load a texture from the filesystem.
  LoadTexture(PathBuf),
  /// Enter empty mode.
  EmptyMode,
  /// Enter shader toy mode with the given shader program.
  ShaderToy(String),
}
