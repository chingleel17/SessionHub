/**
 * 從 package.json 讀取版本號，同步寫入 src-tauri/Cargo.toml
 * 用途：打包前確保版本號一致，只需維護 package.json 一個地方
 */
import { readFileSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
const version = pkg.version;

const cargoPath = resolve(root, 'src-tauri', 'Cargo.toml');
const cargo = readFileSync(cargoPath, 'utf8');
const updated = cargo.replace(/^version = "[\d.]+"/m, `version = "${version}"`);
writeFileSync(cargoPath, updated);

console.log(`[sync-version] ${version} → Cargo.toml`);
