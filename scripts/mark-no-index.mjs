/**
 * 在 src-tauri/target 放置 .metadata_never_index，讓 Spotlight 不索引建置產物
 * 用途：避免 Finder／Spotlight 搜尋出現 target 內的第二個 SessionHub.app
 * 每次打包前執行，因此 cargo clean 或重新 clone 後也會自動補上
 */
import { mkdirSync, writeFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const targetDir = resolve(root, 'src-tauri', 'target');

mkdirSync(targetDir, { recursive: true });
writeFileSync(resolve(targetDir, '.metadata_never_index'), '');

console.log('[mark-no-index] src-tauri/target 已排除 Spotlight 索引');
