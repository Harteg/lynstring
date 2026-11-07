# Plurals Implementation Report

A comprehensive report on the Android plural resources (`<plurals>`) implementation in Lynstring.

---

## 1. General Approach

Plurals are **flattened into the existing string workflow** using composite keys of the form `pluralName|quantity`. This approach:

- **Unifies the data flow**: Both regular strings and plural items use `AndroidString` (or equivalent) with a `name` field. Plural items use `name` = `"pluralName|quantity"` (e.g. `apples|one`, `messages_count|zero`).
- **Minimizes translation changes**: The translation pipeline treats plural items as regular strings. The AI receives and returns `<string name="apples|one">%1$d apple</string>`. No special prompt formatting or response parsing is required for plurals.
- **Reconstructs on write**: When writing to XML, the backend splits composite keys by `|`, groups items by plural name, and emits proper `<plurals>` blocks with `<item quantity="...">` children.
- **Preserves metadata**: `is_translatable`, `has_cdata`, and quantity are carried through from parse to write.

---

## 2. What Was Implemented

### 2.1 Models

| File | Type | Description |
|------|------|-------------|
| `src-tauri/src/models/plural.rs` | `PluralItem` | Single item: `quantity`, `text`, `has_cdata` |
| `src-tauri/src/models/plural.rs` | `AndroidPlural` | Full plural: `name`, `items`, `is_translatable`, `is_commented_out` |
| `src-tauri/bindings/` | `PluralItem.ts`, `AndroidPlural.ts` | TypeScript bindings (ts-rs generated) |

`AllStringsData` was extended with:
- `plurals_by_language: HashMap<String, Vec<AndroidPlural>>` – plurals per language
- `plurals_detected: Vec<String>` – names of plurals found in default strings (retained for backwards compatibility)

### 2.2 Parsing

**File**: `src-tauri/src/commands/strings.rs`

- **`parse_plurals_xml(xml_content: &str) -> Result<Vec<AndroidPlural>, String>`**
  - Regex-based extraction of `<plurals name="...">` blocks
  - Parses `translatable="false"` from attributes
  - Skips plurals inside XML comments
  - For each block, extracts `<item quantity="...">` children
  - Parses text content, detects and strips CDATA, sets `has_cdata`
  - Returns `Vec<AndroidPlural>` with all quantities preserved

- **`read_resources_xml()`**
  - Reads strings.xml, calls `parse_strings_xml` and `parse_plurals_xml`
  - Returns `(Vec<AndroidString>, Vec<AndroidPlural>)`

- **`load_project_data()`**
  - Merges plurals per language into `plurals_by_language`
  - Fills `plurals_detected` from default plurals

### 2.3 Writing

**File**: `src-tauri/src/commands/strings.rs`

- **`write_strings_to_file()`** / **`write_strings_to_file_merge()`**
  - Splits input strings into:
    - **Regular strings**: `!s.name.contains('|')`
    - **Plural items**: `s.name.contains('|')` with format `pluralName|quantity`
  - Builds `plurals_map: HashMap<String, AndroidPlural>` from flattened plural items
  - Writes strings as `<string>` and plurals as `<plurals>` with `<item quantity="...">`
  - Preserves `translatable="false"` and CDATA when emitting plurals
  - Handles merge mode: updates existing plurals in place, appends new ones
  - Preserves order and indent of existing content where possible

### 2.4 Translation Integration

**File**: `src-tauri/src/commands/translation.rs`

- **Input**: `Vec<AndroidString>` including flattened plurals (e.g. `name: "apples|one"`)
- **Format**: Sent to AI as `<string name="apples|one">%1$d apple</string>`
- **Output**: Parsed and mapped back by `name`; `write_strings_to_file_merge` reconstructs `<plurals>` blocks
- **Metadata**: `has_cdata` and `is_translatable` are preserved from source when writing
- No plural-specific logic in prompts or response parsing; plural items are handled like normal strings.

### 2.5 Frontend

| File | Changes |
|------|---------|
| `src/lib/types/strings.ts` | `AllStringsData` extended with `plurals_by_language`, `plurals_detected` |
| `src/lib/stores/project-data.ts` | `processStringsData()` flattens plurals into rows with keys `pluralName|quantity`; plural items appear in string table; `pluralsDetected` stored and used |
| `src/lib/components/dialogs/NewProjectDialog.svelte` | Shows warning when `plurals_detected` non-empty (text: "Plural translation is not supported yet") – **note**: message may be outdated now that plurals are implemented |

---

## 3. Tests Added

### Parsing tests (`parsing_tests.rs`)

| Test | Verifies |
|------|----------|
| `test_parse_plurals` | Parses `apples` plural with 6 quantity items (zero, one, two, few, many, other); preserves `%1$d`; parses `messages_count` with CDATA and `has_cdata` |
| `test_parse_plurals_placeholders` | Parses `plural_placeholders`; preserves `%1$s`, `%2$d`, `%3$f`, `%%` |
| `test_parse_plurals_translatable_false` | Parses `plural_not_translatable`; `is_translatable` is false when `translatable="false"` |
| `test_parse_plurals_empty_item` | Parses `plural_empty_item`; empty `<item quantity="other">` yields empty string |

### Writing tests (`writing_tests.rs`)

| Test | Verifies |
|------|----------|
| `test_write_plurals` | Round-trip: parse plurals, flatten to `name|quantity`, write; output contains `<plurals>`, all 6 quantities preserved for `apples` |
| `test_write_plurals_preserves_placeholders` | Round-trip preserves `%1$d`, `%1$s`, `%2$d`, `%3$f`, `%%` in plural items |
| `test_write_plurals_preserves_cdata` | Round-trip preserves `has_cdata` and HTML content in `messages_count` zero item |

### Translation tests (`translation_tests.rs`)

| Test | Verifies |
|------|----------|
| `test_translation_plurals_write` | Flattens plurals to `name|quantity`; mock-translates; writes via merge; parses written file; `apples|one` contains expected translation ("manzana") |

### Configuration

- `test_config.toml`: `plurals = true` for parsing, writing, and translation
- `test_config.rs`: `plurals` fields in `ParsingConfig`, `WritingConfig`, `TranslationConfig`

---

## 4. Fixture Additions

All plurals added to `tests/fixtures/android_project_basic/app/src/main/res/values/strings.xml`:

| Plural name | Purpose |
|-------------|---------|
| `apples` | Full CLDR quantities (zero, one, two, few, many, other); basic placeholders `%1$d` |
| `messages_count` | CDATA in zero item (`<![CDATA[No <b>messages</b>]]>`); mixed placeholders; `translatable="true"` |
| `plural_placeholders` | Placeholder preservation: `%1$s`, `%2$d`, `%3$f`, `%%` in one and other |
| `plural_not_translatable` | `translatable="false"` – should be excluded from translation |
| `plural_empty_item` | Empty `<item quantity="other">` – edge case for empty text |
| `complex_plural` | Placeholders, HTML in CDATA, multiple quantities |

---

## 5. Edge Cases Covered

| Edge case | Handling |
|-----------|----------|
| **Placeholders** | `%1$s`, `%2$d`, `%3$f`, `%1$d` preserved through parse/write; tests in `test_parse_plurals_placeholders`, `test_write_plurals_preserves_placeholders` |
| **Literal `%%`** | Preserved; tested in `plural_placeholders` |
| **CDATA** | Detected and stripped during parse; `has_cdata` set; re-emitted as `<![CDATA[...]]>` on write; tested in `messages_count`, `complex_plural`, `test_write_plurals_preserves_cdata` |
| **translatable="false"** | Parsed into `is_translatable: false`; excluded from frontend table; write preserves attribute; tested in `test_parse_plurals_translatable_false` |
| **Empty items** | Parsed as empty string; written as `<item quantity="other"></item>`; tested in `test_parse_plurals_empty_item` |
| **Commented plurals** | Skipped via comment-range detection in `parse_plurals_xml` |
| **Merge mode** | Existing plurals updated in place; new plurals appended; string order and structure preserved |

---

## Summary

Plurals are fully integrated into the string workflow via composite keys `pluralName|quantity`. Parsing, writing, translation, and the frontend all support plural resources. Tests cover parsing, writing, round-trip, placeholder preservation, CDATA, `translatable="false"`, and empty items. The fixture set provides targeted coverage for these scenarios.
