use anyhow::Result;
use chrono::Utc;

use crate::core::paths::TrellisPaths;

use super::config::{ensure_registry_config, resolve_source_path};
use super::index::{
    build_package, write_index, MirrorSummary, RegistryIndex, RegistrySummary, SkippedSpec,
};

#[derive(Debug, Clone)]
pub struct RegistrySyncReport {
    pub index: RegistryIndex,
}

pub fn sync_registry(
    paths: &TrellisPaths,
    registry_root_override: Option<&std::path::Path>,
) -> Result<RegistrySyncReport> {
    let registry_root = registry_root_override.unwrap_or(&paths.default_registry_root);
    let mut config = ensure_registry_config(&paths.registry_sources, registry_root)?;

    if registry_root_override.is_some() {
        for source in &mut config.sources {
            if source.official && source.enabled {
                source.location = registry_root.to_string_lossy().to_string();
            }
        }
        super::config::write_registry_config(&paths.registry_sources, &config)?;
    }

    let mut registries = Vec::new();
    let mut packages = Vec::new();
    let mut skipped: Vec<SkippedSpec> = Vec::new();

    for source in config.sources.into_iter().filter(|source| source.enabled) {
        let source_path = resolve_source_path(&paths.home, &source.location);
        let scan = super::index::scan_registry(&source.name, &source_path);
        let package_count = scan.entries.len();

        packages.extend(scan.entries.iter().map(build_package));
        skipped.extend(scan.skipped);

        registries.push(RegistrySummary {
            name: source.name,
            source_path: source_path.to_string_lossy().to_string(),
            package_count,
            skipped_count: skipped
                .iter()
                .filter(|s| s.registry == scan.metadata.name)
                .count(),
            revision: scan.metadata.revision,
            refreshed_at: Utc::now(),
            mirrors: source
                .mirrors
                .into_iter()
                .map(|m| MirrorSummary {
                    name: m.name,
                    location: m.location,
                    priority: m.priority,
                    enabled: m.enabled,
                })
                .collect(),
        });
    }

    packages.sort_by(|a, b| {
        a.registry
            .cmp(&b.registry)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.version.cmp(&b.version))
    });

    let index = RegistryIndex {
        schema_version: "0.3".to_string(),
        generated_at: Utc::now(),
        registries,
        packages,
        skipped,
    };

    let index_path = paths.registry_index.clone();
    write_index(&index, &index_path)?;
    Ok(RegistrySyncReport { index })
}
