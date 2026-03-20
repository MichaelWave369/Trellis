use anyhow::{bail, Result};
use regex::Regex;
use std::path::Path;

use super::package::{PackageKind, PackageSpec, SourceType};

pub fn validate(spec: &PackageSpec) -> Result<()> {
    validate_name(&spec.name)?;
    validate_version(&spec.version)?;

    if spec.description.trim().is_empty() {
        bail!("description cannot be empty");
    }
    if !spec.homepage.starts_with("http://") && !spec.homepage.starts_with("https://") {
        bail!("homepage must start with http:// or https://");
    }

    validate_source(spec)?;

    if spec.install.strategy != "copy" {
        bail!("install.strategy must be 'copy' in v0.2");
    }
    if spec.install.entries.is_empty() {
        bail!("install.entries must contain at least one entry");
    }

    if spec.kind == PackageKind::Binary && spec.bin.is_empty() {
        bail!("binary packages must define at least one exposed bin entry");
    }

    if spec.provenance.publisher.trim().is_empty()
        || spec.provenance.license.trim().is_empty()
        || spec.provenance.registry.trim().is_empty()
    {
        bail!("provenance.publisher, provenance.license, and provenance.registry are required");
    }

    validate_platform(spec)?;
    validate_post_install(spec)?;

    Ok(())
}

pub fn validate_name(name: &str) -> Result<()> {
    let re = Regex::new(r"^[a-z0-9][a-z0-9-]{1,63}$")?;
    if !re.is_match(name) {
        bail!(
            "invalid package name '{}': use lowercase letters, digits, hyphen, length 2-64",
            name
        );
    }
    Ok(())
}

pub fn validate_version(version: &str) -> Result<()> {
    let re = Regex::new(r"^\d+\.\d+\.\d+([-.][0-9A-Za-z.]+)?$")?;
    if !re.is_match(version) {
        bail!(
            "invalid version '{}': expected semver-like format (for example 1.2.3 or 1.2.3-alpha)",
            version
        );
    }
    Ok(())
}

fn validate_source(spec: &PackageSpec) -> Result<()> {
    if Path::new(&spec.source.path).is_absolute() {
        bail!("source.path must be relative to spec directory");
    }
    if spec.source.path.contains("..") {
        bail!("source.path cannot contain '..'");
    }

    match spec.source.source_type {
        SourceType::File | SourceType::Dir | SourceType::Archive => {}
    }

    if let Some(sum) = &spec.source.checksum_sha256 {
        let re = Regex::new(r"^[a-fA-F0-9]{64}$")?;
        if !re.is_match(sum) {
            bail!("source.checksum_sha256 must be a 64-char hex SHA-256");
        }
    }

    Ok(())
}

fn validate_platform(spec: &PackageSpec) -> Result<()> {
    if let Some(platform) = &spec.platform {
        for os in &platform.os {
            match os.as_str() {
                "linux" | "macos" | "windows" => {}
                _ => bail!("unsupported platform.os value '{}'", os),
            }
        }
        for arch in &platform.arch {
            match arch.as_str() {
                "x86_64" | "aarch64" => {}
                _ => bail!("unsupported platform.arch value '{}'", arch),
            }
        }
    }
    Ok(())
}

fn validate_post_install(spec: &PackageSpec) -> Result<()> {
    if let Some(post_install) = &spec.post_install {
        if post_install.policy != "allowlisted" {
            bail!("post_install.policy must be 'allowlisted' when post_install is set");
        }

        let allowed = ["echo", "true"];
        let command_name = post_install
            .command
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !allowed.contains(&command_name) {
            bail!(
                "post_install.command '{}' is not allowed in v0.2 (allowed: echo, true)",
                command_name
            );
        }
    }
    Ok(())
}

pub fn platform_matches(spec: &PackageSpec) -> bool {
    let Some(platform) = &spec.platform else {
        return true;
    };

    let os_ok = platform.os.is_empty() || platform.os.iter().any(|os| os == std::env::consts::OS);
    let arch_ok = platform.arch.is_empty()
        || platform
            .arch
            .iter()
            .any(|arch| arch == std::env::consts::ARCH);
    os_ok && arch_ok
}
