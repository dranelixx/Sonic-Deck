#!/usr/bin/env node
/**
 * Sync version from version.json to package.json, Cargo.toml, and tauri.conf.json
 * Run this script before building to ensure all version files are in sync
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

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
 * Write a JSON file safely
 * @param {string} filePath - Path to the JSON file
 * @param {object} data - Data to write
 * @param {string} description - Description for logging
 */
function writeJsonFile(filePath, data, description) {
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
 * Write a file safely
 * @param {string} filePath - Path to the file
 * @param {string} content - Content to write
 * @param {string} description - Description for logging
 */
function writeFile(filePath, content, description) {
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

if (!version || typeof version !== 'string') {
  console.error('❌ Error: No valid version found in version.json');
  process.exit(1);
}

console.log(`📦 Syncing version to: ${version}`);

// 1. Update package.json
const packageJsonPath = path.join(__dirname, '../package.json');
const packageJson = readJsonFile(packageJsonPath, 'package.json');
packageJson.version = version;
writeJsonFile(packageJsonPath, packageJson, 'package.json');

// 2. Update src-tauri/Cargo.toml
// Use regex that targets only the [package] section to avoid modifying dependency versions
const cargoTomlPath = path.join(__dirname, '../src-tauri/Cargo.toml');
let cargoToml = readFile(cargoTomlPath, 'src-tauri/Cargo.toml');

// Match version line only within [package] section (before the next section starts)
const cargoVersionRegex = /(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m;
if (!cargoVersionRegex.test(cargoToml)) {
  console.error('❌ Error: Could not find version in [package] section of Cargo.toml');
  process.exit(1);
}
cargoToml = cargoToml.replace(cargoVersionRegex, `$1"${version}"`);
writeFile(cargoTomlPath, cargoToml, 'src-tauri/Cargo.toml');

// 3. Update src-tauri/tauri.conf.json
const tauriConfPath = path.join(__dirname, '../src-tauri/tauri.conf.json');
const tauriConf = readJsonFile(tauriConfPath, 'src-tauri/tauri.conf.json');
tauriConf.version = version;
writeJsonFile(tauriConfPath, tauriConf, 'src-tauri/tauri.conf.json');

console.log(`\n✨ Version sync complete: ${version}\n`);
