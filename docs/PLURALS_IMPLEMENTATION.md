# Implementing Plurals & String-Arrays

Plural resources (`<plurals>`) are currently detected and warned about but **not translated**. String-arrays (`<string-array>`) are not yet detected or supported. This document captures the plan for full implementation of both.

## Current State

- **Scanner**: `detect_plurals_in_xml()` in `src-tauri/src/commands/strings.rs` finds plural names via regex
- **Warning**: NewProjectDialog shows an orange notice when plurals are detected
- **Data**: `AllStringsData.plurals_detected: Vec<String>` lists names only (no content)
- **Tests**: `test_parse_plurals` and `test_write_plurals` exist in `parsing_tests.rs` and `writing_tests.rs` but are disabled (`plurals = false` in `test_config.toml`)

## Android Plural Format

```xml
<plurals name="apples">
    <item quantity="zero">No apples</item>
    <item quantity="one">%1$d apple</item>
    <item quantity="two">%1$d apples</item>
    <item quantity="few">%1$d apples</item>
    <item quantity="many">%1$d apples</item>
    <item quantity="other">%1$d apples</item>
</plurals>
```

CLDR quantity categories: `zero`, `one`, `two`, `few`, `many`, `other`. Not all languages need all; `other` is the fallback. Plurals can live in `strings.xml` or `plurals.xml`.

## Recommended Data Model

**Separate type + parallel storage** (not a unified enum):

```rust
pub struct PluralItem {
    pub quantity: String,  // "zero" | "one" | "two" | "few" | "many" | "other"
    pub text: String,
    pub has_cdata: bool,
}

pub struct AndroidPlural {
    pub name: String,
    pub items: Vec<PluralItem>,
    pub is_translatable: bool,
    pub is_commented_out: bool,
}

// AllStringsData becomes:
pub struct AllStringsData {
    pub strings_by_language: HashMap<String, Vec<AndroidString>>,
    pub plurals_by_language: HashMap<String, Vec<AndroidPlural>>,  // NEW
    pub language_display: HashMap<String, LanguageDisplayInfo>,
    pub plurals_detected: Vec<String>,  // can remove once full impl exists
}
```

## Implementation Checklist

### 1. Rust – Models & Parsing

- [ ] Add `PluralItem` and `AndroidPlural` in `src-tauri/src/models/` (or extend `android_string.rs`)
- [ ] Add `parse_plurals_xml(xml_content: &str) -> Result<Vec<AndroidPlural>, String>`
- [ ] Parse `<plurals name="...">` and child `<item quantity="...">` with CDATA/escaping like strings
- [ ] Integrate into `read_strings_xml` or add `read_plurals_xml`; merge into `load_project_data`
- [ ] Enable and fix `test_parse_plurals`

### 2. Rust – Writing

- [ ] Add `write_plurals_to_file()` that emits correct XML
- [ ] In merge/overwrite flows, preserve plurals when updating strings (or update plurals alongside strings)
- [ ] Handle both `strings.xml` (inline plurals) and `plurals.xml` – start with `strings.xml` only
- [ ] Enable and fix `test_write_plurals`

### 3. Translation Pipeline

- [ ] Treat each plural `<item>` as a separate translatable unit
- [ ] Include quantity context in prompts: e.g. "plural: apples, quantity: one"
- [ ] Map translated items back to `AndroidPlural` by name + quantity
- [ ] Preserve placeholders (`%1$d`, etc.) per item same as strings

### 4. Frontend

- [ ] Update `AllStringsData` TypeScript bindings
- [ ] Extend string table / translation UI to show plurals (grouped by name, expandable per quantity)
- [ ] Include plurals in translation selection, progress, and merge logic

### 5. Unused Strings

- [ ] Scan for `getQuantityString(R.plurals.xyz, n)` (and Kotlin/Java variants) when detecting unused plurals

## Files to Modify

| Area | Files |
|------|-------|
| Models | `src-tauri/src/models/android_string.rs` or new `plural.rs` |
| Parsing | `src-tauri/src/commands/strings.rs` |
| Writing | `src-tauri/src/commands/strings.rs` |
| Translation | `src-tauri/src/commands/translation.rs` |
| Tests | `parsing_tests.rs`, `writing_tests.rs`, `test_config.toml` |
| Frontend | Project data store, string table components |

## String-Arrays (Future)

String-arrays share the same "one name, multiple items" pattern as plurals but are simpler:

```xml
<string-array name="planets">
    <item>Mercury</item>
    <item>Venus</item>
    <item>Earth</item>
</string-array>
```

- **No quantity attribute** – just ordered items, so easier than plurals
- **Same translation flow** – each `<item>` is a separate translatable unit
- **Reuse patterns** – `AndroidStringArray` with `items: Vec<String>` (or `Vec<ArrayItem>` if we need metadata)
- **Priority**: Implement plurals first (more common), then string-arrays using the same architecture

Consider adding `detect_string_arrays_in_xml()` and a warning similar to plurals before full support. String-arrays can live in `strings.xml` or `arrays.xml` – scan all XML in `values*/` if we want full coverage.

## Fixture

Use `tests/fixtures/android_project_basic/app/src/main/res/values/strings.xml` – it contains an `apples` plural block for testing.
