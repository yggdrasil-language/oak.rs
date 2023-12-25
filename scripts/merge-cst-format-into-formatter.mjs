#!/usr/bin/env node
/**
 * Merge `oak-typescript/src/cst_format/` into `oak-typescript/src/formatter/`
 * and scrub leftover `cst_*` module names.
 *
 * Target layout (only `formatter/` + `printer/` at src root):
 *   formatter/
 *     source.rs        ← cst_format/mod.rs body (or rename from cst_source.rs)
 *     bridge.rs        ← cst_format/formatter_bridge.rs
 *     trivia_guard.rs  ← cst_format/trivia_guard.rs
 *     red_tree/        ← cst_format/red_tree/
 *     options.rs       ← absorbs former CstFormatOptions helpers
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
    out = out.replaceAll("super::cst_source::", "super::source::");
    out = out.replaceAll("cst_source::", "source::");
    out = out.replaceAll("super::cst_options::CstFormatOptions", "super::options::FormatOptions");
    out = out.replaceAll("cst_options::CstFormatOptions", "options::FormatOptions");
    out = out.replaceAll("CstFormatOptions", "FormatOptions");
    out = out.replaceAll("use super::cst_options::FormatOptions;", "use super::options::FormatOptions;");
    out = out.replaceAll(
        "use super::{options::FormatOptions, red_tree::TypeScriptRedTreeFormatter};",
        "use super::{options::FormatOptions, red_tree::TypeScriptRedTreeFormatter};",
    );
    out = out.replaceAll(
        "use super::{options::FormatOptions, source::format_source_file};",
        "use super::{options::FormatOptions, source::format_source_file};",
    );
    out = out.replaceAll("(`cst_format`, `print`, `red_tree`)", "(`red_tree`, `printer`)");
    out = out.replaceAll("(`cst_format`, `print`)", "(`red_tree`, `printer`)");
    out = out.replaceAll("not a separate `cst_format` tree", "private submodules only");
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
//! Product entry: [\`format_source\`] with [\`FormatOptions\`]. CST rules live in private submodules.

mod bridge;
mod engine;
mod error;
mod options;
mod red_tree;
mod source;
pub(crate) mod trivia_guard;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
`;
    write(file, content);
}

function patchLibRs() {
    const file = path.join(SRC, "lib.rs");
    let content = read(file);
    content = content.replaceAll("mod cst_format;\n", "").replaceAll("mod cst_format;\r\n", "");
    write(file, content);
}

function patchFormatterOptions() {
    const file = path.join(FORMATTER, "options.rs");
    let content = read(file);
    content = content.replace("use crate::cst_format::CstFormatOptions;", "");
    content = content.replace("use super::cst_options::CstFormatOptions;", "");
    content = content.replace(
        /    pub\(crate\) fn cst_options\(&self\) -> CstFormatOptions \{\s*CstFormatOptions \{ indent_width: self\.indent_width, line_width: self\.line_width \}\s*\}/,
        "",
    );

    const layoutHelpers = `
    pub(crate) fn printer_config(&self) -> oak_pretty_print::PrinterConfig {
        oak_pretty_print::PrinterConfig::new()
            .with_indent_style(oak_pretty_print::IndentStyle::Spaces(self.indent_width))
            .with_max_width(self.line_width)
    }

    pub(crate) fn finalize_output(&self, source: &str, body: String) -> String {
        let mut config = self.printer_config();
        config.insert_final_newline = source.ends_with('\\n');
        let doc = oak_pretty_print::document::Document::text(body);
        oak_pretty_print::Printer::new(config).print(&doc)
    }
`;

    if (!content.includes("fn printer_config")) {
        content = content.replace(/\n}\s*$/, `${layoutHelpers}\n}\n`);
    }

    if (!content.includes("use oak_pretty_print")) {
        content = `use oak_pretty_print::{IndentStyle, Printer, PrinterConfig, document::Document};\n\n${content}`;
    }

    write(file, content);
}

function patchFormatterEngine() {
    const file = path.join(FORMATTER, "engine.rs");
    let content = read(file);
    content = content.replace("crate::cst_format::format_source", "super::source::format_source");
    content = content.replace("super::cst_source::format_source", "super::source::format_source");
    content = content.replace(/let cst = options\.cst_options\(\);\s*\n\s*/g, "");
    content = content.replace(/let cst = options\.cst_options\(\);\s*\r\n\s*/g, "");
    content = content.replace(/super::source::format_source\(source, &cst\)/g, "super::source::format_source(source, options)");
    content = content.replace(/crate::cst_format::format_source\(source, &cst\)/g, "super::source::format_source(source, options)");
    write(file, content);
}

function patchBridge() {
    const file = path.join(FORMATTER, "bridge.rs");
    if (!fs.existsSync(file)) {
        return;
    }
    let content = read(file);
    content = content.replace(
        "use super::{CstFormatOptions, format_source_file};",
        "use super::{options::FormatOptions, source::format_source_file};",
    );
    content = content.replace(
        "use super::{cst_options::CstFormatOptions, cst_source::format_source_file};",
        "use super::{options::FormatOptions, source::format_source_file};",
    );
    write(file, content);
}

function patchRedTreeMod() {
    const file = path.join(FORMATTER, "red_tree", "mod.rs");
    if (!fs.existsSync(file)) {
        return;
    }
    let content = read(file);
    content = content.replace(
        "use super::{CstFormatOptions, trivia_guard};",
        "use super::{options::FormatOptions, trivia_guard};",
    );
    content = content.replace(
        "use super::{cst_options::CstFormatOptions, trivia_guard};",
        "use super::{options::FormatOptions, trivia_guard};",
    );
    write(file, content);
}

function patchCheckStructures() {
    const file = path.join(ROOT, "scripts", "check-structures.mjs");
    if (!fs.existsSync(file)) {
        return;
    }
    let content = read(file);
    const layouts = [
        ["const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'lsp', 'mcp'];", "formatter', 'printer"],
        ["const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'formatter', 'print', 'lsp', 'mcp'];", "formatter', 'printer"],
    ];
    for (const [needle, token] of layouts) {
        if (content.includes(needle)) {
            content = content.replace(
                needle,
                "const ALLOWED_SRC_DIRS = ['ast', 'builder', 'parser', 'lexer', 'language', 'formatter', 'printer', 'lsp', 'mcp'];",
            );
            write(file, content);
            return;
        }
    }
    if (!content.includes("'formatter'")) {
        throw new Error("check-structures.mjs ALLOWED_SRC_DIRS pattern not found");
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

function mergeDirectory() {
    if (!fs.existsSync(CST)) {
        return false;
    }

    moveFile(path.join(CST, "trivia_guard.rs"), path.join(FORMATTER, "trivia_guard.rs"));
    moveFile(path.join(CST, "formatter_bridge.rs"), path.join(FORMATTER, "bridge.rs"));
    moveDir(path.join(CST, "red_tree"), path.join(FORMATTER, "red_tree"));

    let cstMod = read(path.join(CST, "mod.rs"));
    cstMod = stripCstModDeclarations(cstMod);
    cstMod = rewriteRustImports(cstMod);
    write(path.join(FORMATTER, "source.rs"), cstMod);

    const cstOptions = path.join(CST, "options.rs");
    if (fs.existsSync(cstOptions)) {
        write(path.join(FORMATTER, "cst_options.rs"), read(cstOptions));
    }

    fs.rmSync(CST, { recursive: true, force: true });
    return true;
}

function scrubLegacyNames() {
    const cstSource = path.join(FORMATTER, "cst_source.rs");
    const source = path.join(FORMATTER, "source.rs");
    if (fs.existsSync(cstSource) && !fs.existsSync(source)) {
        fs.renameSync(cstSource, source);
    }

    const cstOptions = path.join(FORMATTER, "cst_options.rs");
    if (fs.existsSync(cstOptions)) {
        fs.rmSync(cstOptions, { force: true });
    }
}

function main() {
    const merged = mergeDirectory();
    scrubLegacyNames();

    if (!merged && !fs.existsSync(path.join(FORMATTER, "source.rs"))) {
        console.log(`skip: no cst_format directory and formatter/source.rs already present under ${OAK_TS}`);
        return;
    }

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

    console.log(`merged cst_format into formatter (no cst_* modules) under ${OAK_TS}`);
}

main();
