// Copyright (c) 2026 Ztry8 (AslanD)
// Licensed under the Apache License, Version 2.0 (the "License");
// You may obtain a copy of the License at http://www.apache.org/licenses/LICENSE-2.0

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RUNTIME_HEADER: &str = include_str!("../../runtime/runtime.h");
const RUNTIME_SOURCE: &str = include_str!("../../runtime/runtime.c");

#[derive(Debug)]
pub struct BuildError(pub String);

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn build(c_source: &str, output_path: &Path) -> Result<PathBuf, BuildError> {
    let work_dir = make_temp_dir()?;

    let header_path = work_dir.join("cyl_runtime.h");
    let runtime_c_path = work_dir.join("cyl_runtime.c");
    let main_c_path = work_dir.join("cyl_main.c");

    fs::write(&header_path, RUNTIME_HEADER)
        .map_err(|e| BuildError(format!("could not write {}: {e}", header_path.display())))?;
    fs::write(&runtime_c_path, RUNTIME_SOURCE)
        .map_err(|e| BuildError(format!("could not write {}: {e}", runtime_c_path.display())))?;
    fs::write(&main_c_path, c_source)
        .map_err(|e| BuildError(format!("could not write {}: {e}", main_c_path.display())))?;

    let compiler = locate_compiler()?;

    let final_output = platform_executable_path(output_path);
    if let Some(parent) = final_output.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            BuildError(format!(
                "could not create output directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let mut cmd = Command::new(&compiler.binary);
    cmd.current_dir(&work_dir);

    cmd.arg("-std=c89");
    if compiler.supports_pedantic_flags {
        cmd.arg("-pedantic-errors");
    }

    cmd.arg(&main_c_path);
    cmd.arg(&runtime_c_path);
    cmd.arg("-lm");
    cmd.arg("-o");
    cmd.arg(&final_output);

    let result = cmd.output().map_err(|e| {
        BuildError(format!(
            "failed to run '{}': {e}",
            compiler.binary.display()
        ))
    })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        return Err(BuildError(format!(
            "C compilation failed using '{}':\n{stderr}{stdout}",
            compiler.binary.display()
        )));
    }

    let _ = fs::remove_dir_all(&work_dir);

    Ok(final_output)
}

pub fn compile_and_build(
    ast: &[crate::parser::AstNode],
    output_path: &Path,
) -> Result<PathBuf, BuildError> {
    let c_source = crate::codegen::compile(ast).map_err(BuildError)?;
    build(&c_source, output_path)
}

struct DetectedCompiler {
    binary: PathBuf,
    supports_pedantic_flags: bool,
}

fn locate_compiler() -> Result<DetectedCompiler, BuildError> {
    if let Ok(cc) = std::env::var("CYLIUM_CC") {
        let path = PathBuf::from(&cc);
        return Ok(DetectedCompiler {
            supports_pedantic_flags: looks_like_gcc_or_clang(&path),
            binary: path,
        });
    }

    if let Some(bundled) = bundled_tcc_path() {
        if bundled.is_file() {
            return Ok(DetectedCompiler {
                binary: bundled,
                supports_pedantic_flags: false, // tcc: never pass GCC-style pedantic flags
            });
        }
    }

    if let Some(path) = which("cc") {
        return Ok(DetectedCompiler {
            binary: path,
            supports_pedantic_flags: true,
        });
    }
    if let Some(path) = which("gcc") {
        return Ok(DetectedCompiler {
            binary: path,
            supports_pedantic_flags: true,
        });
    }
    if let Some(path) = which("clang") {
        return Ok(DetectedCompiler {
            binary: path,
            supports_pedantic_flags: true,
        });
    }
    if let Some(path) = which("tcc") {
        return Ok(DetectedCompiler {
            binary: path,
            supports_pedantic_flags: false,
        });
    }

    Err(BuildError(
        "no C compiler found. Install a C compiler (e.g. 'gcc', 'clang', or 'tcc'), \
         set the CYLIUM_CC environment variable to its path, or place a 'tcc' binary \
         next to the cylium executable."
            .to_owned(),
    ))
}

fn bundled_tcc_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let name = if cfg!(target_os = "windows") {
        "tcc.exe"
    } else {
        "tcc"
    };
    Some(dir.join(name))
}

fn looks_like_gcc_or_clang(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    !name.contains("tcc")
}

fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let candidate_name = if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_owned()
    };

    std::env::split_paths(&path_var).find_map(|dir| {
        let candidate = dir.join(&candidate_name);
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    })
}

fn platform_executable_path(output_path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") && output_path.extension().is_none() {
        output_path.with_extension("exe")
    } else {
        output_path.to_path_buf()
    }
}

fn make_temp_dir() -> Result<PathBuf, BuildError> {
    let base = std::env::temp_dir();
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = base.join(format!("cylium-build-{pid}-{nanos}"));
    fs::create_dir_all(&dir)
        .map_err(|e| BuildError(format!("could not create temp dir {}: {e}", dir.display())))?;
    Ok(dir)
}
