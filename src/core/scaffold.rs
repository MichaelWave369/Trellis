use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub enum ScaffoldKind {
    Binary,
    Source,
}

impl ScaffoldKind {
    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "binary" => Ok(Self::Binary),
            "source" => Ok(Self::Source),
            _ => bail!(
                "unsupported scaffold kind '{}': use binary or source",
                value
            ),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::Source => "source",
        }
    }
}

pub fn scaffold_package(root: &Path, package_name: &str, kind: ScaffoldKind) -> Result<PathBuf> {
    let package_dir = root.join(package_name);
    if package_dir.exists() {
        bail!(
            "scaffold target already exists: {}. Choose another name or remove the directory first",
            package_dir.display()
        );
    }

    let bin_dir = package_dir.join("payload/bin");
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create scaffold path {}", bin_dir.display()))?;

    let bin_path = bin_dir.join(package_name);
    fs::write(
        &bin_path,
        format!(
            "#!/usr/bin/env sh\nset -eu\n\ncmd=\"${{1:-version}}\"\ncase \"$cmd\" in\n  version)\n    echo \"{} 0.1.0\"\n    ;;\n  *)\n    echo \"{} scaffold package. Edit payload/bin/{} for real behavior.\"\n    ;;\nesac\n",
            package_name, package_name, package_name
        ),
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&bin_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&bin_path, perms)?;
    }

    let spec_path = package_dir.join(format!("{}.trellis.yaml", package_name));
    let spec = format!(
        r#"schema_version: "0.2"
name: {name}
version: 0.1.0
description: "TODO describe {name} in one clear sentence"
homepage: https://example.org/{name}
kind: {kind}
source:
  type: local_dir
  path: payload
  signature: sig:{name}-local-dev
install:
  strategy: copy
  entries:
    - bin
bin:
  {name}: bin/{name}
dependencies: []
provenance:
  publisher: "TODO your org or handle"
  license: "TODO SPDX identifier"
  registry: vineyard-core
platform:
  os: [linux, macos]
  arch: [x86_64, aarch64]
health:
  notes: "TODO package health/maintenance notes"
"#,
        name = package_name,
        kind = kind.as_str(),
    );
    fs::write(&spec_path, spec)
        .with_context(|| format!("failed to write scaffold spec {}", spec_path.display()))?;

    Ok(package_dir)
}
