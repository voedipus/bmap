//! Domain types and pure functions for parsing and grouping MAP file entries.

use mapfile_parser::MapFile;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// A single symbol extracted from a MAP file.
///
/// `path_str` is the pre-computed `filepath.to_string_lossy()`.
#[derive(Debug, Clone)]
pub struct MapEntry {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub section_type: String,
    pub path_str: String,
}

/// Aggregated size and count for a group of symbols.
#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub name: String,
    pub total_size: u64,
    pub num_symbols: usize,
}

impl GroupSummary {
    fn add(&mut self, size: u64) {
        self.total_size += size;
        self.num_symbols += 1;
    }
}

/// Broad ELF-style section categories used for grouping and summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionCategory {
    Text,
    Data,
    Rodata,
    Bss,
    Other,
}

impl SectionCategory {
    fn classify(section: &str) -> Self {
        if section.starts_with(".text") {
            Self::Text
        } else if section.starts_with(".data") {
            Self::Data
        } else if section.starts_with(".rodata") {
            Self::Rodata
        } else if section.starts_with(".bss") || section.starts_with(".sbss") {
            Self::Bss
        } else {
            Self::Other
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Text => ".text",
            Self::Data => ".data",
            Self::Rodata => ".rodata",
            Self::Bss => ".bss",
            Self::Other => "other",
        }
    }
}

/// Aggregated statistics for the whole loaded file.
#[derive(Debug, Clone, Default)]
pub struct FileSummary {
    pub total_size: u64,
    pub text_size: u64,
    pub data_size: u64,
    pub rodata_size: u64,
    pub bss_size: u64,
    pub other_size: u64,
    pub num_symbols: usize,
    pub num_files: usize,
    pub num_sections: usize,
}

/// Pre-computed groups and summary from a single pass over entries.
#[derive(Debug, Clone, Default)]
pub struct Aggregate {
    pub module_groups: Vec<GroupSummary>,
    pub archive_groups: Vec<GroupSummary>,
    pub section_groups: Vec<GroupSummary>,
    pub section_categories: Vec<GroupSummary>,
    pub summary: FileSummary,
}

/// Compute all groups and the summary in ONE pass over entries.
pub fn aggregate(entries: &[MapEntry], show_debug: bool) -> Aggregate {
    let mut modules: BTreeMap<String, GroupSummary> = BTreeMap::new();
    let mut archives: BTreeMap<String, GroupSummary> = BTreeMap::new();
    let mut sections: BTreeMap<String, GroupSummary> = BTreeMap::new();
    let mut categories: BTreeMap<String, GroupSummary> = BTreeMap::new();
    let mut files = BTreeSet::new();
    let mut section_names = BTreeSet::new();
    let mut total_size = 0u64;
    let mut text_size = 0u64;
    let mut data_size = 0u64;
    let mut rodata_size = 0u64;
    let mut bss_size = 0u64;
    let mut other_size = 0u64;
    let mut num_symbols = 0usize;

    for e in entries {
        if !show_debug && is_debug_section(&e.section_type) {
            continue;
        }

        let size = e.size;
        num_symbols += 1;
        total_size += size;

        let cat = SectionCategory::classify(&e.section_type);
        match cat {
            SectionCategory::Text => text_size += size,
            SectionCategory::Data => data_size += size,
            SectionCategory::Rodata => rodata_size += size,
            SectionCategory::Bss => bss_size += size,
            SectionCategory::Other => other_size += size,
        }

        let path = &e.path_str;

        // Module key
        modules
            .entry(path.clone())
            .or_insert_with(|| GroupSummary {
                name: path.clone(),
                total_size: 0,
                num_symbols: 0,
            })
            .add(size);

        // Archive key
        let archive_key = if let Some(p) = path.find('(') {
            path[..p].to_string()
        } else {
            path.clone()
        };
        archives
            .entry(archive_key.clone())
            .or_insert_with(|| GroupSummary {
                name: archive_key,
                total_size: 0,
                num_symbols: 0,
            })
            .add(size);

        // Section key
        sections
            .entry(e.section_type.clone())
            .or_insert_with(|| GroupSummary {
                name: e.section_type.clone(),
                total_size: 0,
                num_symbols: 0,
            })
            .add(size);

        // Category key
        let cat_name = cat.name().to_string();
        categories
            .entry(cat_name.clone())
            .or_insert_with(|| GroupSummary {
                name: cat_name,
                total_size: 0,
                num_symbols: 0,
            })
            .add(size);

        files.insert(path.clone());
        section_names.insert(e.section_type.clone());
    }

    fn sort_groups(mut g: Vec<GroupSummary>) -> Vec<GroupSummary> {
        g.sort_by_key(|b| std::cmp::Reverse(b.total_size));
        g
    }

    Aggregate {
        module_groups: sort_groups(modules.into_values().collect()),
        archive_groups: sort_groups(archives.into_values().collect()),
        section_groups: sort_groups(sections.into_values().collect()),
        section_categories: sort_groups(categories.into_values().collect()),
        summary: FileSummary {
            total_size,
            text_size,
            data_size,
            rodata_size,
            bss_size,
            other_size,
            num_symbols,
            num_files: files.len(),
            num_sections: section_names.len(),
        },
    }
}

pub fn build_symbol_entries(map: &MapFile) -> Vec<MapEntry> {
    let mut entries = Vec::new();
    for segment in &map.segments_list {
        for section in &segment.sections_list {
            if is_system_library(&section.filepath) {
                continue;
            }
            for sym in &section.symbols {
                entries.push(MapEntry {
                    name: sym.name.clone(),
                    address: sym.vram,
                    size: sym.size,
                    section_type: section.section_type.clone(),
                    path_str: section.filepath.to_string_lossy().into_owned(),
                });
            }
        }
    }
    entries
}

/// True if the path belongs to a standard system/runtime library.
fn is_system_library(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains("libm.a")
        || s.contains("libgcc")
        || s.contains("libc.a")
        || s.contains("libc_nano.a")
        || s.contains("libnosys")
        || s.contains("libstdc++")
        || s.contains("libsupc++")
        || s.contains("crtbegin")
        || s.contains("crtend")
        || s.contains("crti")
        || s.contains("crtn")
}

pub fn matches_category(category: &str, section: &str) -> bool {
    SectionCategory::classify(section).name() == category
}

// ─── Drill-down helpers ───────────────────────────────────────

/// Return entry indices matching a group path, respecting the debug filter.
pub fn drill_indices(entries: &[MapEntry], group_path: &str, show_debug: bool) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            if !show_debug && is_debug_section(&e.section_type) {
                return false;
            }
            let p = &e.path_str;
            if p == group_path {
                return true;
            }
            if let Some(pos) = p.find('(') {
                return &p[..pos] == group_path;
            }
            false
        })
        .map(|(i, _)| i)
        .collect()
}

/// True for sections that contain debug info or toolchain metadata.
pub fn is_debug_section(section: &str) -> bool {
    section.starts_with(".debug_")
        || section == ".comment"
        || section.starts_with(".note")
        || section == ".stab"
        || section.starts_with(".stabstr")
        || section.starts_with(".ARM.attributes")
        || section.starts_with(".ARM.exidx")
        || section.starts_with(".ARM.extab")
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// Extracts the source object name from an archive path like `libfoo.a(foo.o)`.
pub fn source_name(object_path: &str) -> String {
    let inner = if let Some(start) = object_path.find('(') {
        if let Some(end) = object_path.find(')') {
            &object_path[start + 1..end]
        } else {
            &object_path[start + 1..]
        }
    } else {
        object_path
    };
    PathBuf::from(inner)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Extracts the archive or file name from an object path.
pub fn archive_name(object_path: &str) -> String {
    if let Some(p) = object_path.find('(') {
        let archive = &object_path[..p];
        PathBuf::from(archive)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| archive.to_string())
    } else {
        PathBuf::from(object_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| object_path.to_string())
    }
}
