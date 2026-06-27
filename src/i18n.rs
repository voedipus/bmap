//! Simple compile-time translations for English and Russian.

use gpui::SharedString;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Russian,
}

impl Language {
    pub fn detect() -> Self {
        match sys_locale::get_locale() {
            Some(locale) if locale.starts_with("ru") => Language::Russian,
            _ => Language::English,
        }
    }
}

/// Looks up a translated string for the given language, falling back to the key.
pub fn t(key: &str, lang: Language) -> SharedString {
    STRINGS.get(&(lang, key)).copied().unwrap_or(key).into()
}

type Key = (Language, &'static str);

static STRINGS: LazyLock<std::collections::HashMap<Key, &'static str>> = LazyLock::new(|| {
    let mut m = std::collections::HashMap::new();
    for (k, v) in EN {
        m.insert((Language::English, *k), *v);
    }
    for (k, v) in RU {
        m.insert((Language::Russian, *k), *v);
    }
    m
});

const EN: &[(&str, &str)] = &[
    ("app-title", "bmap"),
    ("description", "MAP file memory analyzer"),
    ("repository", "https://github.com/bmap"),
    ("about", "About"),
    ("view", "View"),
    ("open", "Open"),
    ("open-map-file", "Open MAP File"),
    ("search-placeholder", "Search symbols..."),
    ("by-module", "By Module"),
    ("by-section", "By Section"),
    ("all-symbols", "All Symbols"),
    ("summary", "Summary"),
    ("files", "Source Files"),
    ("modules", "Modules"),
    ("sections", "Sections"),
    ("search-by-source", "Search by source..."),
    ("search-by-module", "Search by module..."),
    ("total-size", "Total Size"),
    ("code-size", "Code (.text)"),
    ("data-size", "Data (.data/.rodata)"),
    ("bss-size", "BSS (.bss)"),
    ("no-file-loaded", "No MAP file loaded."),
    (
        "open-instruction",
        "Click \"Open\" to load a linker MAP file.",
    ),
    ("column-name", "Name"),
    ("column-address", "Address"),
    ("column-size", "Size"),
    ("column-percentage", "% of Total"),
    ("column-file", "File"),
    ("error-loading", "Error loading file"),
    ("format-gnu", "GNU ld"),
    ("format-lld", "Clang LLD"),
    ("format-metrowerks", "Metrowerks"),
    ("format-unknown", "Unknown"),
    ("symbols", "Symbols"),
    ("column-source", "Source"),
    ("column-module", "Module"),
    ("column-archive", "Archive"),
    ("column-section", "Section"),
    ("label-debug", "Debug symbols"),
    ("label-other", "Other"),
    ("label-object-files", "Object files"),
    ("label-section-types", "Section types"),
    ("label-no-matches", "No matching symbols."),
    ("label-back-to-files", "Back to files"),
];

const RU: &[(&str, &str)] = &[
    ("app-title", "bmap"),
    ("description", "Анализатор MAP-файлов"),
    ("repository", "https://github.com/bmap"),
    ("about", "О программе"),
    ("view", "Вид"),
    ("open", "Открыть"),
    ("open-map-file", "Открыть MAP-файл"),
    ("search-placeholder", "Поиск символов..."),
    ("by-module", "По модулям"),
    ("by-section", "По секциям"),
    ("all-symbols", "Все символы"),
    ("summary", "Сводка"),
    ("files", "Исходные файлы"),
    ("modules", "Модули"),
    ("sections", "Секции"),
    ("search-by-source", "Поиск по исходному файлу..."),
    ("search-by-module", "Поиск по модулю..."),
    ("total-size", "Общий размер"),
    ("code-size", "Код (.text)"),
    ("data-size", "Данные (.data/.rodata)"),
    ("bss-size", "BSS (.bss)"),
    ("no-file-loaded", "MAP-файл не загружен."),
    (
        "open-instruction",
        "Нажмите \"Открыть\" для загрузки MAP-файла.",
    ),
    ("column-name", "Имя"),
    ("column-address", "Адрес"),
    ("column-size", "Размер"),
    ("column-percentage", "% от общего"),
    ("column-file", "Файл"),
    ("error-loading", "Ошибка загрузки файла"),
    ("format-gnu", "GNU ld"),
    ("format-lld", "Clang LLD"),
    ("format-metrowerks", "Metrowerks"),
    ("format-unknown", "Неизвестно"),
    ("symbols", "Символы"),
    ("column-source", "Исходный файл"),
    ("column-module", "Модуль"),
    ("column-archive", "Архив"),
    ("column-section", "Секция"),
    ("label-debug", "Отладочные символы"),
    ("label-other", "Прочее"),
    ("label-object-files", "Объектные файлы"),
    ("label-section-types", "Типы секций"),
    ("label-no-matches", "Совпадений не найдено."),
    ("label-back-to-files", "Назад к файлам"),
];
