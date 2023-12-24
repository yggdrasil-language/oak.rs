import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.join(__dirname, '..');
const EXAMPLES_DIR = path.join(ROOT_DIR, 'examples');
const LOG_FILE = path.join(ROOT_DIR, 'check-structures.log');
const MAX_LINES = 1000;
const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'formatter', 'print', 'lsp', 'mcp'];
const ALLOWED_SRC_FILES = ['lib.rs', 'main.rs', 'mod.rs'];

// Violation object structure:
// {
//     file: string,
//     line: number,
//     message: string,
//     type: 'size' | 'naming' | 'structure' | 'directory'
// }

const violations = [];
const logLines = [];

function log(message = '') {
    console.log(message);
    logLines.push(message);
}

function checkFile(filePath) {
    const relativePath = path.relative(EXAMPLES_DIR, filePath);
    if (relativePath.startsWith('oak-c4' + path.sep) || relativePath.startsWith('oak-uml' + path.sep)) {
        return;
    }
    const content = fs.readFileSync(filePath, 'utf-8');
    const lines = content.split('\n');
    const parts = relativePath.split(path.sep);

    const srcIndex = parts.indexOf('src');
    if (srcIndex !== -1 && srcIndex < parts.length - 1) {
        const itemAfterSrc = parts[srcIndex + 1];
        const fullItemPath = path.join(EXAMPLES_DIR, ...parts.slice(0, srcIndex + 2));
        const isDirectory = fs.statSync(fullItemPath).isDirectory();

        if (isDirectory) {
            if (!ALLOWED_SRC_DIRS.includes(itemAfterSrc)) {
                violations.push({
                    file: relativePath,
                    line: 0,
                    message: `Invalid directory structure: "src/${itemAfterSrc}" is not in the allowed list (${ALLOWED_SRC_DIRS.join(', ')})`,
                    type: 'directory'
                });
            }
        } else {
            if (!ALLOWED_SRC_FILES.includes(itemAfterSrc) && !itemAfterSrc.endsWith('.md')) {
                violations.push({
                    file: relativePath,
                    line: 0,
                    message: `Invalid file location: "src/${itemAfterSrc}" should be moved to a suitable subdirectory (e.g. lsp)`,
                    type: 'directory'
                });
            }
        }
    }

    if (lines.length > MAX_LINES) {
        violations.push({
            file: relativePath,
            line: 1,
            message: `File too large (${lines.length} lines), exceeds limit of ${MAX_LINES} lines`,
            type: 'size'
        });
    }

    const isAstFile = filePath.includes(path.sep + 'ast' + path.sep);
    const isParserFile = filePath.includes(path.sep + 'parser' + path.sep);
    const isBuilderFile = filePath.includes(path.sep + 'builder' + path.sep);

    const fileName = path.basename(filePath);
    const isTestFile = filePath.includes(path.sep + 'tests' + path.sep) || fileName.startsWith('test_') || fileName === 'test.rs';

    if (!isTestFile && fileName !== 'mod.rs' && fileName !== 'lib.rs' && fileName !== 'element_type.rs' && !fileName.endsWith('.md')) {
        if (isAstFile && !fileName.endsWith('_nodes.rs')) {
            violations.push({
                file: relativePath,
                line: 0,
                message: `Invalid AST file naming: "${fileName}" should end with "_nodes.rs" (e.g., expression_nodes.rs)`,
                type: 'structure'
            });
        }
        if (isParserFile && !fileName.startsWith('parse_') && !fileName.endsWith('_parser.rs')) {
            violations.push({
                file: relativePath,
                line: 0,
                message: `Invalid Parser file naming: "${fileName}" should start with "parse_" (e.g., parse_expression.rs)`,
                type: 'structure'
            });
        }
        if (isBuilderFile && !fileName.startsWith('build_') && !fileName.endsWith('_builder.rs')) {
            violations.push({
                file: relativePath,
                line: 0,
                message: `Invalid Builder file naming: "${fileName}" should start with "build_" (e.g., build_root.rs)`,
                type: 'structure'
            });
        }
    }
}

function walkDir(dir) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const fullPath = path.join(dir, file);
        const stat = fs.statSync(fullPath);
        if (stat.isDirectory()) {
            walkDir(fullPath);
        } else if (file.endsWith('.rs')) {
            checkFile(fullPath);
        }
    }
}

log(`Checking directory: ${EXAMPLES_DIR} ...`);
walkDir(EXAMPLES_DIR);

if (violations.length === 0) {
    log('No structural issues found!');
} else {
    log(`Found ${violations.length} structural issues:\n`);

    const grouped = violations.reduce((acc, v) => {
        if (!acc[v.file]) acc[v.file] = [];
        acc[v.file].push(v);
        return acc;
    }, {});

    for (const [file, fileViolations] of Object.entries(grouped)) {
        log(`File: ${file}`);
        fileViolations.sort((a, b) => a.line - b.line).forEach(v => {
            log(`  [L${v.line}] [${v.type}] ${v.message}`);
        });
        log('');
    }
}

fs.writeFileSync(LOG_FILE, logLines.join('\n'), 'utf-8');
console.log(`\nLog saved to: ${LOG_FILE}`);
