#!/usr/bin/env node
/**
 * Fetches CLDR locale display names and generates a Rust module for fallback lookups.
 * Run: pnpm update-locale-names
 *
 * Source: Unicode CLDR (https://github.com/unicode-org/cldr-json)
 */

import { writeFileSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import https from 'https';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');

const CLDR_BASE =
  'https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-localenames-full/main/en';
const LANGUAGES_URL = `${CLDR_BASE}/languages.json`;
const TERRITORIES_URL = `${CLDR_BASE}/territories.json`;

function escapeRustString(s) {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

function shouldSkipKey(key) {
  return (
    key.includes('-alt-') ||
    key.includes('-menu') ||
    key.includes('-variant') ||
    key === 'und' ||
    key === 'zxx' ||
    key === 'mul'
  );
}

// Skip meta/group territory codes that aren't useful for locale display
const SKIP_TERRITORIES = new Set([
  '001', '002', '003', '005', '009', '011', '013', '014', '015', '017', '018',
  '019', '021', '029', '030', '034', '035', '039', '053', '054', '057', '061',
  '142', '143', '145', '150', '151', '154', '155', '202',
  'AC', 'CP', 'CQ', 'DG', 'EA', 'EU', 'EZ', 'IC', 'QO', 'TA', 'UN', 'XA', 'XB', 'ZZ',
]);

function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, (res) => {
        let data = '';
        res.on('data', (chunk) => (data += chunk));
        res.on('end', () => {
          try {
            resolve(JSON.parse(data));
          } catch (e) {
            reject(e);
          }
        });
      })
      .on('error', reject);
  });
}

async function main() {
  console.log('Fetching CLDR data...');
  const [languagesData, territoriesData] = await Promise.all([
    fetchJson(LANGUAGES_URL),
    fetchJson(TERRITORIES_URL),
  ]);

  const languages = languagesData?.main?.en?.localeDisplayNames?.languages ?? {};
  const territories = territoriesData?.main?.en?.localeDisplayNames?.territories ?? {};

  const languageEntries = [];
  for (const [code, name] of Object.entries(languages)) {
    if (shouldSkipKey(code) || !name || typeof name !== 'string') continue;
    // Keep compound codes (en-US, es-419) as-is for direct lookup; lowercase base codes
    const key = code.includes('-') ? code : code.toLowerCase();
    languageEntries.push([key, name]);
  }

  const territoryEntries = [];
  for (const [code, name] of Object.entries(territories)) {
    if (shouldSkipKey(code) || !name || typeof name !== 'string') continue;
    if (SKIP_TERRITORIES.has(code)) continue;
    territoryEntries.push([code, name]);
  }

  // Add Android format aliases for compound codes (en-US -> en-rUS)
  const androidAliases = [];
  for (const [code, name] of languageEntries) {
    if (code.includes('-') && !code.startsWith('en-r') && !code.startsWith('es-r')) {
      const androidCode = code.replace(
        /([a-z]+)-([A-Za-z0-9]+)/,
        (_, lang, region) => `${lang}-r${region.toUpperCase()}`
      );
      if (androidCode !== code) {
        androidAliases.push([androidCode, name]);
      }
    }
  }

  const outputPath = join(rootDir, 'src-tauri', 'src', 'models', 'locale_names_generated.rs');

  const rustContent = `//! Auto-generated from Unicode CLDR. Do not edit manually.
//! Run: pnpm update-locale-names
//!
//! Provides fallback display names for locale codes not in our curated LANGUAGES list.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Language code -> display name (from CLDR languages.json)
    pub static ref CLDR_LANGUAGE_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
${languageEntries.map(([k, v]) => `        m.insert("${escapeRustString(k)}", "${escapeRustString(v)}");`).join('\n')}
${androidAliases.map(([k, v]) => `        m.insert("${escapeRustString(k)}", "${escapeRustString(v)}");`).join('\n')}
        m
    };

    /// Territory/region code -> display name (from CLDR territories.json)
    pub static ref CLDR_TERRITORY_NAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
${territoryEntries.map(([k, v]) => `        m.insert("${escapeRustString(k)}", "${escapeRustString(v)}");`).join('\n')}
        m
    };
}

/// Compose a display name from language + region (e.g. en-rUS -> "English (United States)")
pub fn compose_locale_name(lang_code: &str, region_code: &str) -> Option<String> {
    let lang_name = CLDR_LANGUAGE_NAMES.get(lang_code)?;
    let region_name = CLDR_TERRITORY_NAMES
        .get(region_code)
        .or_else(|| {
            let upper = region_code.to_uppercase();
            if upper != region_code {
                CLDR_TERRITORY_NAMES.get(upper.as_str())
            } else {
                None
            }
        })?;
    Some(format!("{} ({})", lang_name, region_name))
}
`;

  writeFileSync(outputPath, rustContent);
  console.log(`✓ Generated ${outputPath}`);
  console.log(`  - ${languageEntries.length + androidAliases.length} language entries`);
  console.log(`  - ${territoryEntries.length} territory entries`);
}

main().catch((err) => {
  console.error('Failed to update locale names:', err);
  process.exit(1);
});
