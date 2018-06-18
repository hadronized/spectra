#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Msg {
  Close,
}
