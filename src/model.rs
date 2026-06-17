use mapfile_parser::MapFile;
use std::path::PathBuf;

/// Flat row for table display.
#[derive(Debug, Clone)]
pub struct MapEntry {
    pub name: String,
    pub address: u64,
    pub size: u64,
    pub section_type: String,
    pub filepath: PathBuf,
    #[allow(dead_code)]
    pub entry_type: EntryType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    Symbol,
}

/// Aggregated summary for a group (module or section).
#[derive(Debug, Clone)]
pub struct GroupSummary {
    pub name: String,
    pub total_size: u64,
    pub text_size: u64,
    pub data_size: u64,
    pub rodata_size: u64,
    pub bss_size: u64,
    pub num_symbols: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortColumn {
    Name,
    Address,
    Size,
    Percentage,
}

/// Computed summary for the whole file.
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

pub fn parse_map_file(contents: &str) -> MapFile {
    MapFile::new_from_map_str(contents)
}

/// Build flat symbol entries from a MapFile, skipping system/compiler libraries.
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
                    entry_type: EntryType::Symbol,
                });
            }
        }
    }
    entries
}

/// Check if a filepath refers to a system/compiler support library.
fn is_system_library(path: &PathBuf) -> bool {
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

/// Build group summaries by module (object file path).
pub fn group_by_module(entries: &[MapEntry]) -> Vec<GroupSummary> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, GroupSummary> = BTreeMap::new();
    for e in entries {
        let key = e.filepath.to_string_lossy().to_string();
        let g = groups.entry(key.clone()).or_insert(GroupSummary {
            name: key,
            total_size: 0,
            text_size: 0,
            data_size: 0,
            rodata_size: 0,
            bss_size: 0,
            num_symbols: 0,
        });
        g.total_size += e.size;
        g.num_symbols += 1;
        let sec = e.section_type.as_str();
        if sec.starts_with(".text") {
            g.text_size += e.size;
        } else if sec.starts_with(".data") {
            g.data_size += e.size;
        } else if sec.starts_with(".rodata") {
            g.rodata_size += e.size;
        } else if sec.starts_with(".bss") || sec.starts_with(".sbss") {
            g.bss_size += e.size;
        }
    }
    let mut result: Vec<GroupSummary> = groups.into_values().collect();
    result.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    result
}

/// Build group summaries by section type.
pub fn group_by_section(entries: &[MapEntry]) -> Vec<GroupSummary> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, GroupSummary> = BTreeMap::new();
    for e in entries {
        let key = e.section_type.clone();
        let g = groups.entry(key.clone()).or_insert(GroupSummary {
            name: key,
            total_size: 0,
            text_size: 0,
            data_size: 0,
            rodata_size: 0,
            bss_size: 0,
            num_symbols: 0,
        });
        g.total_size += e.size;
        g.num_symbols += 1;
        let sec = e.section_type.as_str();
        if sec.starts_with(".text") {
            g.text_size += e.size;
        } else if sec.starts_with(".data") {
            g.data_size += e.size;
        } else if sec.starts_with(".rodata") {
            g.rodata_size += e.size;
        } else if sec.starts_with(".bss") || sec.starts_with(".sbss") {
            g.bss_size += e.size;
        }
    }
    let mut result: Vec<GroupSummary> = groups.into_values().collect();
    result.sort_by(|a, b| b.total_size.cmp(&a.total_size));
    result
}

/// Compute overall file summary from all entries.
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
        let sec = e.section_type.as_str();
        if sec.starts_with(".text") {
            s.text_size += e.size;
        } else if sec.starts_with(".data") {
            s.data_size += e.size;
        } else if sec.starts_with(".rodata") {
            s.rodata_size += e.size;
        } else if sec.starts_with(".bss") || sec.starts_with(".sbss") {
            s.bss_size += e.size;
        } else {
            s.other_size += e.size;
        }
    }
    s.num_files = files.len();
    s.num_sections = sections.len();
    s
}

/// Filter entries by search query (case-insensitive substring match).
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

/// Sort entries by column.
pub fn sort_entries(entries: &mut [MapEntry], col: SortColumn, ascending: bool) {
    entries.sort_by(|a, b| {
        let ord = match col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Address => a.address.cmp(&b.address),
            SortColumn::Size => a.size.cmp(&b.size),
            SortColumn::Percentage => a.size.cmp(&b.size),
        };
        if ascending { ord } else { ord.reverse() }
    });
}

/// Sort group summaries by total size.
pub fn sort_groups(groups: &mut [GroupSummary], ascending: bool) {
    groups.sort_by(|a, b| {
        let ord = a.total_size.cmp(&b.total_size);
        if ascending { ord } else { ord.reverse() }
    });
}
