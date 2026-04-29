use super::*;

const REQUIRED_ITEMS: &[&str] = &["Amulet of the Ancients"];

impl Content {
    #[cfg(test)]
    pub fn load_from_str(s: &str) -> anyhow::Result<Self> {
        let content: Self = serde_json::from_str(s)?;
        content.validate()?;
        Ok(content)
    }

    pub fn load_from_dir(path: &std::path::Path) -> anyhow::Result<Self> {
        let t0 = std::time::Instant::now();

        let mut merged = Self::default();
        let mut monster_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut item_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut spell_titles: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut lore_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut feature_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        let mut yaml_files: Vec<std::path::PathBuf> = Vec::new();
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("yaml") {
                yaml_files.push(p.to_path_buf());
            }
        }
        yaml_files.sort();

        for file in &yaml_files {
            let s = std::fs::read_to_string(file)?;
            let partial: Self = serde_yml::from_str(&s)
                .map_err(|e| anyhow::anyhow!("{}: {}", file.display(), e))?;

            for m in partial.monsters {
                if !monster_names.insert(m.name.clone()) {
                    anyhow::bail!(
                        "duplicate monster name '{}' (found in {})",
                        m.name,
                        file.display()
                    );
                }
                merged.monsters.push(m);
            }
            for i in partial.items {
                if !item_names.insert(i.name.clone()) {
                    anyhow::bail!(
                        "duplicate item name '{}' (found in {})",
                        i.name,
                        file.display()
                    );
                }
                merged.items.push(i);
            }
            for sp in partial.spells {
                if !spell_titles.insert(sp.title.clone()) {
                    anyhow::bail!(
                        "duplicate spell title '{}' (found in {})",
                        sp.title,
                        file.display()
                    );
                }
                merged.spells.push(sp);
            }
            for ls in partial.lore {
                if !lore_ids.insert(ls.id.clone()) {
                    anyhow::bail!(
                        "duplicate lore id '{}' (found in {})",
                        ls.id,
                        file.display()
                    );
                }
                merged.lore.push(ls);
            }
            for f in partial.features {
                if !feature_names.insert(f.name.clone()) {
                    anyhow::bail!(
                        "duplicate feature name '{}' (found in {})",
                        f.name,
                        file.display()
                    );
                }
                merged.features.push(f);
            }
            merged.floor_events.extend(partial.floor_events);
            if let Some(pd) = partial.player {
                if merged.player.is_some() {
                    anyhow::bail!(
                        "duplicate [player] defaults section (found in {})",
                        file.display()
                    );
                }
                merged.player = Some(pd);
            }
        }

        let elapsed_ms = t0.elapsed().as_millis();
        if elapsed_ms > 200 {
            log::warn!(
                "Content::load_from_dir took {}ms — check for I/O bottlenecks or excessive content volume",
                elapsed_ms
            );
        } else {
            log::debug!("Content loaded in {}ms", elapsed_ms);
        }

        merged.validate()?;
        Ok(merged)
    }

    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_dir(std::path::Path::new("content/"))
    }

    pub(super) fn validate(&self) -> anyhow::Result<()> {
        for name in REQUIRED_ITEMS {
            if !self.items.iter().any(|i| i.name == *name) {
                anyhow::bail!("content is missing required item: \"{}\"", name);
            }
        }
        for raw in &self.spells {
            raw.validate()?;
        }
        Ok(())
    }
}
