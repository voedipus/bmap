//! Domain types and pure functions for parsing and grouping MAP file entries.

use mapfile_parser::MapFile;
use std::path::{Path, PathBuf};

/// A single symbol extracted from a MAP file.
#[derive(Debug, Clone)]
pub struct MapEntry {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub section_type: String,
    pub filepath: PathBuf,
}

/// Aggregated size and count for a group of symbols.
#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub name: String,
    pub total_size: u64,
    pub num_symbols: usize,
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
                    filepath: section.filepath.clone(),
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

/// Groups entries by an extracted string key and computes per-group totals.
fn group_by(entries: &[MapEntry], key_fn: fn(&MapEntry) -> String) -> Vec<GroupSummary> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, GroupSummary> = BTreeMap::new();
    for e in entries {
        let key = key_fn(e);
        let g = groups.entry(key.clone()).or_insert(GroupSummary {
            name: key,
            total_size: 0,
            num_symbols: 0,
        });
        g.total_size += e.size;
        g.num_symbols += 1;
    }
    let mut result: Vec<GroupSummary> = groups.into_values().collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.total_size));
    result
}

pub fn group_by_module(entries: &[MapEntry]) -> Vec<GroupSummary> {
    group_by(entries, |e| e.filepath.to_string_lossy().to_string())
}

pub fn group_by_archive(entries: &[MapEntry]) -> Vec<GroupSummary> {
    group_by(entries, |e| {
        let path = e.filepath.to_string_lossy().to_string();
        if let Some(p) = path.find('(') {
            path[..p].to_string()
        } else {
            path
        }
    })
}

pub fn group_by_section(entries: &[MapEntry]) -> Vec<GroupSummary> {
    group_by(entries, |e| e.section_type.clone())
}

pub fn group_section_categories(entries: &[MapEntry]) -> Vec<GroupSummary> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, GroupSummary> = BTreeMap::new();
    for e in entries {
        let cat = SectionCategory::classify(&e.section_type)
            .name()
            .to_string();
        let g = groups.entry(cat.clone()).or_insert(GroupSummary {
            name: cat,
            total_size: 0,
            num_symbols: 0,
        });
        g.total_size += e.size;
        g.num_symbols += 1;
    }
    let mut result: Vec<GroupSummary> = groups.into_values().collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.total_size));
    result
}

pub fn matches_category(category: &str, section: &str) -> bool {
    SectionCategory::classify(section).name() == category
}

pub fn compute_file_summary(entries: &[MapEntry]) -> FileSummary {
    let mut s = FileSummary::default();
    use std::collections::BTreeSet;
    let mut files = BTreeSet::new();
    let mut sections = BTreeSet::new();
    for e in entries {
        s.total_size += e.size;
        s.num_symbols += 1;
        files.insert(e.filepath.to_string_lossy().to_string());
        sections.insert(e.section_type.clone());
        match SectionCategory::classify(&e.section_type) {
            SectionCategory::Text => s.text_size += e.size,
            SectionCategory::Data => s.data_size += e.size,
            SectionCategory::Rodata => s.rodata_size += e.size,
            SectionCategory::Bss => s.bss_size += e.size,
            SectionCategory::Other => s.other_size += e.size,
        }
    }
    s.num_files = files.len();
    s.num_sections = sections.len();
    s
}

/// Filters entries by name, path, or section type using a case-insensitive query.
pub fn filter_entries(entries: &[MapEntry], query: &str) -> Vec<MapEntry> {
    if query.is_empty() {
        return entries.to_vec();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            e.name.to_lowercase().contains(&q)
                || e.filepath.to_string_lossy().to_lowercase().contains(&q)
                || e.section_type.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

/// Sorts entries by size in the given direction.
pub fn sort_entries(entries: &mut [MapEntry], ascending: bool) {
    entries.sort_by(|a, b| {
        let ord = a.size.cmp(&b.size);
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}

/// Sorts group summaries by total size in the given direction.
pub fn sort_groups(groups: &mut [GroupSummary], ascending: bool) {
    groups.sort_by(|a, b| {
        let ord = a.total_size.cmp(&b.total_size);
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });
}

/// Removes debug and toolchain metadata sections from the entry list.
pub fn filter_debug_entries(entries: &[MapEntry]) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| !is_debug_section(&e.section_type))
        .cloned()
        .collect()
}

/// True for sections that contain debug info or toolchain metadata.
fn is_debug_section(section: &str) -> bool {
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

/// Filters entries whose file path matches the selected group.
///
/// A group may be either an exact object file path or the outer archive path
/// of an entry like `libfoo.a(foo.o)`.
pub fn filter_by_group_path(entries: &[MapEntry], group_path: &str) -> Vec<MapEntry> {
    entries
        .iter()
        .filter(|e| {
            let entry_path = e.filepath.to_string_lossy();
            if entry_path == group_path {
                return true;
            }
            if let Some(p) = entry_path.find('(') {
                return &entry_path[..p] == group_path;
            }
            false
        })
        .cloned()
        .collect()
}
