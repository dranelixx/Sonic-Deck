#!/usr/bin/env node
/**
 * Sync version from version.json to package.json, Cargo.toml, and tauri.conf.json
 * Run this script before building to ensure all version files are in sync
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Parse CLI arguments
const args = process.argv.slice(2);
const isDryRun = args.includes('--dry-run');

/**
 * SemVer pattern for SonicDeck build versions
 * Format: X.Y.Z or X.Y.Z-N where N is 0 (alpha), 1 (beta), or 2 (rc)
 * Examples: "1.0.0", "0.7.0-0", "0.8.0-1", "0.9.0-2"
 */
const SEMVER_PATTERN = /^\d+\.\d+\.\d+(-[0-2])?$/;

/**
 * Validate version string against SemVer pattern
 * @param {string} version - Version string to validate
 * @returns {boolean} True if valid
 */
function isValidVersion(version) {
  return typeof version === 'string' && SEMVER_PATTERN.test(version);
}

/**
 * Read and parse a JSON file safely
 * @param {string} filePath - Path to the JSON file
 * @param {string} description - Description for error messages
 * @returns {object} Parsed JSON content
 */
function readJsonFile(filePath, description) {
  try {
    const content = fs.readFileSync(filePath, 'utf-8');
    return JSON.parse(content);
  } catch (err) {
    console.error(`❌ Error reading ${description} (${filePath}):`, err.message);
    process.exit(1);
  }
}

/**
 * Write a JSON file safely (or show diff in dry-run mode)
 * @param {string} filePath - Path to the JSON file
 * @param {object} data - Data to write
 * @param {string} description - Description for logging
 * @param {string} oldVersion - Previous version for diff display
 * @param {string} newVersion - New version for diff display
 */
function writeJsonFile(filePath, data, description, oldVersion, newVersion) {
  if (isDryRun) {
    if (oldVersion === newVersion) {
      console.log(`  ${description}: already at ${newVersion}`);
    } else {
      console.log(`  ${description}: ${oldVersion} → ${newVersion}`);
    }
    return;
  }
  try {
    fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n', 'utf-8');
    console.log(`✅ Updated ${description}`);
  } catch (err) {
    console.error(`❌ Error writing ${description} (${filePath}):`, err.message);
    process.exit(1);
  }
}

/**
 * Read a file safely
 * @param {string} filePath - Path to the file
 * @param {string} description - Description for error messages
 * @returns {string} File content
 */
function readFile(filePath, description) {
  try {
    return fs.readFileSync(filePath, 'utf-8');
  } catch (err) {
    console.error(`❌ Error reading ${description} (${filePath}):`, err.message);
    process.exit(1);
  }
}

/**
 * Write a file safely (or show diff in dry-run mode)
 * @param {string} filePath - Path to the file
 * @param {string} content - Content to write
 * @param {string} description - Description for logging
 * @param {string} oldVersion - Previous version for diff display
 * @param {string} newVersion - New version for diff display
 */
function writeFile(filePath, content, description, oldVersion, newVersion) {
  if (isDryRun) {
    if (oldVersion === newVersion) {
      console.log(`  ${description}: already at ${newVersion}`);
    } else {
      console.log(`  ${description}: ${oldVersion} → ${newVersion}`);
    }
    return;
  }
  try {
    fs.writeFileSync(filePath, content, 'utf-8');
    console.log(`✅ Updated ${description}`);
  } catch (err) {
    console.error(`❌ Error writing ${description} (${filePath}):`, err.message);
    process.exit(1);
  }
}

// Read version from version.json
const versionFilePath = path.join(__dirname, '../version.json');
const versionData = readJsonFile(versionFilePath, 'version.json');
const { version } = versionData;

// Validate version format
if (!isValidVersion(version)) {
  console.error(`❌ Error: Invalid version format "${version}"`);
  console.error('   Expected: X.Y.Z or X.Y.Z-N where N is 0 (alpha), 1 (beta), or 2 (rc)');
  console.error('   Examples: "1.0.0", "0.7.0-0", "0.8.0-1", "0.9.0-2"');
  process.exit(1);
}

if (isDryRun) {
  console.log(`🔍 Dry run: Would sync version to ${version}\n`);
} else {
  console.log(`📦 Syncing version to: ${version}\n`);
}

// 1. Update package.json
const packageJsonPath = path.join(__dirname, '../package.json');
const packageJson = readJsonFile(packageJsonPath, 'package.json');
const oldPackageVersion = packageJson.version;
packageJson.version = version;
writeJsonFile(packageJsonPath, packageJson, 'package.json', oldPackageVersion, version);

// 2. Update src-tauri/Cargo.toml
// Use regex that targets only the [package] section to avoid modifying dependency versions
const cargoTomlPath = path.join(__dirname, '../src-tauri/Cargo.toml');
let cargoToml = readFile(cargoTomlPath, 'src-tauri/Cargo.toml');

// Extract current version from Cargo.toml
const cargoVersionExtractRegex = /\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m;
const cargoMatch = cargoToml.match(cargoVersionExtractRegex);
const oldCargoVersion = cargoMatch ? cargoMatch[1] : 'unknown';

// Match version line only within [package] section (before the next section starts)
const cargoVersionRegex = /(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m;
if (!cargoVersionRegex.test(cargoToml)) {
  console.error('❌ Error: Could not find version in [package] section of Cargo.toml');
  process.exit(1);
}
cargoToml = cargoToml.replace(cargoVersionRegex, `$1"${version}"`);
writeFile(cargoTomlPath, cargoToml, 'src-tauri/Cargo.toml', oldCargoVersion, version);

// 3. Update src-tauri/tauri.conf.json
const tauriConfPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
const tauriConf = readJsonFile(tauriConfPath, 'src-tauri/tauri.conf.json');
const oldTauriVersion = tauriConf.version;
tauriConf.version = version;
writeJsonFile(tauriConfPath, tauriConf, 'src-tauri/tauri.conf.json', oldTauriVersion, version);

if (isDryRun) {
  console.log(`\n✨ Dry run complete. No files were modified.\n`);
} else {
  console.log(`\n✨ Version sync complete: ${version}\n`);
}
