pub mod android_string;
pub mod language;
pub mod locale_names_generated;
pub mod plural;
pub mod project;
pub mod script;

// Re-export the structs for easier importing
pub use android_string::AndroidString;
pub use language::{get_language, get_language_emoji, get_language_name, Language, LANGUAGES};
pub use plural::{AndroidPlural, PluralItem};
pub use project::Project;
