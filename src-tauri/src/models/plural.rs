use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A single item within a plural resource (e.g. quantity="one" with text "%1$d apple").
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PluralItem {
    pub quantity: String, // "zero" | "one" | "two" | "few" | "many" | "other"
    pub text: String,
    #[serde(default)]
    pub has_cdata: bool,
}

/// A full Android plural resource (e.g. <plurals name="apples">).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AndroidPlural {
    pub name: String,
    pub items: Vec<PluralItem>,
    pub is_translatable: bool,
    pub is_commented_out: bool,
}
