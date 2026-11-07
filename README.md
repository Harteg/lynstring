<p align="center">
  <img src="docs/logo.png" alt="LynString logo" width="256" />
</p>

<h3 align="center">Stupid Simple App Localization</h3>

<p align="center">
  <a href="#run-it">Run it</a> · <a href="https://github.com/harteg/lynstring/issues">Issues</a> · <a href="LICENSE">GPL-3.0</a>
</p>

---

LynString is a desktop app (macOS, Windows, Linux) that scans your Android project, finds missing or
outdated translations, and writes them straight back into your `res/values-*/strings.xml` files.
No copy-paste. No external tools.

<p align="center">
  <img src="docs/screenshots/project.png" alt="LynString showing 40 missing translations and 1 modified string" width="380">
  &nbsp;
  <img src="docs/screenshots/up-to-date.png" alt="LynString after translating: everything up to date" width="380">
</p>

> Free and open source · Bring your own [Anthropic API key](https://console.anthropic.com/settings/keys) · No account, no telemetry

LynString started as a paid app. It's now free and open source and runs entirely on your machine,
translating with Claude through your own API key.

## Features

- Detects missing translations across all locales and fills them with one click, or adds new languages
- Watches your project and detects modified strings, then offers to re-translate just those
- Preserves placeholders (`%1$s`, `%2$d`, etc.), `<xliff:g>` tags, HTML formatting and CDATA
- Escapes special characters correctly for Android (apostrophes, quotes, `@`, `?`, invalid `\u` escapes)
- Checks that languages with their own script (Hindi, Kannada, Arabic, Japanese, …) really come back
  in that script, and retries a batch if the model drifted into another language
- Supports protected terms (brand names that must stay untranslated) and project-specific context
- Finds unused strings and strings that only exist in translations
- Writes directly to your project files
- Powered by Claude: pick Sonnet 5.5 (default), Haiku 5.5 (cheapest) or Opus 5.5; 100+ languages
- Dark / light mode 🌙☀️

## Requirements

- macOS, Windows or Linux
- [Node.js](https://nodejs.org/) 20 or newer and [pnpm](https://pnpm.io/installation)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- The [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS
  (Xcode Command Line Tools on macOS, WebView2 + MSVC on Windows, WebKitGTK on Linux)
- An [Anthropic API key](https://console.anthropic.com/settings/keys)

## Run it

```bash
git clone https://github.com/harteg/lynstring.git
cd lynstring
pnpm install
pnpm tauri:dev
```

The first build compiles the Rust backend and takes a few minutes; later starts are fast.

When the window opens:

1. Paste your Anthropic API key, pick a model and click **Save**. The key is tested before it's saved.
2. Click **Add Project** and choose your Android app's folder (the one containing `app/src/main/res`).
3. Pick the languages to translate and start.

You can change the key or model any time under **Settings → Change API key or model**.

### Build an installable app

```bash
pnpm tauri build
```

The app bundle (`.app`/`.dmg`, `.msi`/`.exe`, `.deb`/`.AppImage`) ends up in
`src-tauri/target/release/bundle/`. Builds are unsigned, so macOS will ask you to confirm the first
time you open it (right-click → Open).

## API key, models and cost

LynString calls `https://api.anthropic.com/v1/messages` directly with your key. There's no account,
telemetry or server of its own; the only other request is for the UI fonts from Google Fonts.

| Model | Price per million tokens (input / output) | Notes |
|---|---|---|
| Claude Sonnet 5.5 | $2 / $10 | Default. Good quality in all languages |
| Claude Haiku 5.5 | $0.10 / $0.50 | Cheapest; fine for most European languages |
| Claude Opus 5.5 | $4 / $20 | Most careful |

As a rough guide, translating ~800 strings into one language costs around $0.50 on Sonnet for a
European language and up to a few dollars for languages with non-Latin scripts (they use more
output tokens). Haiku costs about 20× less.

The key is stored in plain text in the app's data folder (`settings.json`, readable only by your user):

- macOS: `~/Library/Application Support/dev.lynstring.app/`
- Windows: `%APPDATA%\dev.lynstring.app\`
- Linux: `~/.local/share/dev.lynstring.app/`

If no key is saved, LynString falls back to the `ANTHROPIC_API_KEY` environment variable (or a `.env`
file in the directory you start it from, see `.env.example`).

## Development

```bash
pnpm check                    # Svelte/TypeScript type check
cd src-tauri && cargo test    # Rust unit and integration tests
```

`cargo test` includes one end-to-end test that calls the real API; it's skipped unless
`ANTHROPIC_API_KEY` is set.

Project layout:

```
src/                     SvelteKit frontend (Svelte 5, Tailwind, shadcn-svelte)
  lib/services/          translation flow, string loading
  lib/stores/            app state (projects, settings, history, …)
  lib/components/        UI
src-tauri/src/
  commands/translation.rs   batching, prompts, Anthropic API calls, Android escaping
  commands/strings.rs       reading and writing strings.xml
  commands/app_settings.rs  API key and model settings
  models/script.rs          expected script per language (wrong-language guard)
src-tauri/bindings/      TypeScript types generated from Rust (ts-rs)
```

After changing a Rust type that's exported to TypeScript, regenerate the bindings with
`cd src-tauri && cargo test export_bindings`.

## Privacy

No personal information, source code, crash logs or usage statistics are collected. Your string
content is sent to Anthropic for translation with your own API key, and nothing else leaves your
machine (apart from loading the UI fonts from Google Fonts).

## Feedback, issues and contributing

If you run into a bug or have a feature request, [open an issue](https://github.com/harteg/lynstring/issues).
Pull requests are welcome; please run `pnpm check` and `cargo test` before opening one.

## License

[GPL-3.0](LICENSE). You can use, modify and share LynString freely; if you distribute a modified
version, it must also be open source under the GPL.
