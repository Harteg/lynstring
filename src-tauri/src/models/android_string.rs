use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AndroidString {
    pub name: String,
    pub text: String,
    pub is_translatable: bool,
    pub is_commented_out: bool,
    pub line_number: i32,
    #[serde(default)]
    pub has_cdata: bool,
}
