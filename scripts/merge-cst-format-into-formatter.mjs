#!/usr/bin/env node
/**
 * Merge `oak-typescript/src/cst_format/` into `oak-typescript/src/formatter/`.
 *
 * Target layout (formatter is the only CST formatting module; `print` stays separate):
 *   formatter/
 *     cst_options.rs   ← cst_format/options.rs
 *     cst_source.rs    ← cst_format/mod.rs (body only)
 *     bridge.rs        ← cst_format/formatter_bridge.rs
 *     trivia_guard.rs  ← cst_format/trivia_guard.rs
 *     red_tree/        ← cst_format/red_tree/
 *
 * Usage:
 *   node scripts/merge-cst-format-into-formatter.mjs
 *   node scripts/merge-cst-format-into-formatter.mjs --oak-typescript=examples/oak-typescript
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, "..");

const argRoot = process.argv.find((a) => a.startsWith("--oak-typescript="))?.slice("--oak-typescript=".length);
const OAK_TS = path.resolve(ROOT, argRoot ?? "examples/oak-typescript");
const SRC = path.join(OAK_TS, "src");
const CST = path.join(SRC, "cst_format");
const FORMATTER = path.join(SRC, "formatter");

function read(file) {
    return fs.readFileSync(file, "utf8");
}

function write(file, content) {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, content, "utf8");
}

function moveFile(from, to) {
    if (!fs.existsSync(from)) {
        throw new Error(`missing source file: ${from}`);
    }
    write(to, read(from));
    fs.rmSync(from, { force: true });
}

function moveDir(from, to) {
    if (!fs.existsSync(from)) {
        throw new Error(`missing source directory: ${from}`);
    }
    if (fs.existsSync(to)) {
        throw new Error(`destination already exists: ${to}`);
    }
    fs.renameSync(from, to);
}

function rewriteRustImports(source) {
    let out = source;
    out = out.replaceAll("crate::cst_format::", "crate::formatter::");
    out = out.replaceAll("crate::{cst_format::", "crate::{formatter::");
    out = out.replaceAll("cst_format::{", "formatter::{");
    out = out.replaceAll("//! Bridge from `cst_format`", "//! Bridge from formatter CST");
    out = out.replaceAll("(`cst_format`, `print`, `red_tree`)", "(`red_tree`, `print`)");
    out = out.replaceAll("(`cst_format`, `print`)", "(`red_tree`, `print`)");
    return out;
}

function stripCstModDeclarations(source) {
    return source
        .replace(/^mod formatter_bridge;\r?\n/, "")
        .replace(/^mod options;\r?\n/, "")
        .replace(/^mod red_tree;\r?\n/, "")
        .replace(/^mod trivia_guard;\r?\n/, "")
        .replace(/^pub use options::CstFormatOptions;\r?\n\r?\n/, "")
        .replace(/^use red_tree::TypeScriptRedTreeFormatter;\r?\n\r?\n/, "use super::red_tree::TypeScriptRedTreeFormatter;\n\n");
}

function patchFormatterMod() {
    const file = path.join(FORMATTER, "mod.rs");
    const content = `//! Public formatter API for TypeScript/JavaScript source text.
//!
//! Product entry: [\`format_source\`] with [\`FormatOptions\`]. CST rules and
//! transitional AST output stay in private submodules.

mod bridge;
pub(crate) mod cst_options;
mod cst_source;
mod engine;
mod error;
mod options;
mod red_tree;
pub(crate) mod trivia_guard;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;

pub(crate) use cst_options::CstFormatOptions;
`;
    write(file, content);
}

function patchLibRs() {
    const file = path.join(SRC, "lib.rs");
    let content = read(file);
    content = content.replace(/^mod cst_format;\r?\n/, "");
    write(file, content);
}

function patchFormatterOptions() {
    const file = path.join(FORMATTER, "options.rs");
    let content = read(file);
    content = content.replace("use crate::cst_format::CstFormatOptions;", "use super::cst_options::CstFormatOptions;");
    write(file, content);
}

function patchFormatterEngine() {
    const file = path.join(FORMATTER, "engine.rs");
    let content = read(file);
    content = content.replace("crate::cst_format::format_source", "super::cst_source::format_source");
    write(file, content);
}

function patchBridge() {
    const file = path.join(FORMATTER, "bridge.rs");
    let content = read(file);
    content = content.replace(
        "use super::{CstFormatOptions, format_source_file};",
        "use super::{cst_options::CstFormatOptions, cst_source::format_source_file};",
    );
    write(file, content);
}

function patchRedTreeMod() {
    const file = path.join(FORMATTER, "red_tree", "mod.rs");
    let content = read(file);
    content = content.replace(
        "use super::{CstFormatOptions, trivia_guard};",
        "use super::{cst_options::CstFormatOptions, trivia_guard};",
    );
    write(file, content);
}

function patchCheckStructures() {
    const file = path.join(ROOT, "scripts", "check-structures.mjs");
    let content = read(file);
    const needle = "const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'lsp', 'mcp'];";
    const replacement =
        "const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'formatter', 'print', 'lsp', 'mcp'];";
    if (!content.includes(needle) && !content.includes("'formatter'")) {
        throw new Error("check-structures.mjs ALLOWED_SRC_DIRS pattern not found");
    }
    if (content.includes(needle)) {
        content = content.replace(needle, replacement);
        write(file, content);
    }
}

function walkRsFiles(dir, out = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            walkRsFiles(full, out);
        } else if (entry.name.endsWith(".rs")) {
            out.push(full);
        }
    }
    return out;
}

function main() {
    if (!fs.existsSync(CST)) {
        console.log(`skip: ${CST} already merged`);
        return;
    }

    moveFile(path.join(CST, "options.rs"), path.join(FORMATTER, "cst_options.rs"));
    moveFile(path.join(CST, "trivia_guard.rs"), path.join(FORMATTER, "trivia_guard.rs"));
    moveFile(path.join(CST, "formatter_bridge.rs"), path.join(FORMATTER, "bridge.rs"));
    moveDir(path.join(CST, "red_tree"), path.join(FORMATTER, "red_tree"));

    let cstMod = read(path.join(CST, "mod.rs"));
    cstMod = stripCstModDeclarations(cstMod);
    cstMod = rewriteRustImports(cstMod);
    write(path.join(FORMATTER, "cst_source.rs"), cstMod);
    fs.rmSync(CST, { recursive: true, force: true });

    patchFormatterMod();
    patchLibRs();
    patchFormatterOptions();
    patchFormatterEngine();
    patchBridge();
    patchRedTreeMod();
    patchCheckStructures();

    for (const file of walkRsFiles(SRC)) {
        const original = read(file);
        const updated = rewriteRustImports(original);
        if (updated !== original) {
            write(file, updated);
        }
    }

    console.log(`merged cst_format -> formatter under ${OAK_TS}`);
}

main();
