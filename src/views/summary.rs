//! Summary page: high-level file and section statistics.

use crate::app::{AppModel, Message, format_size};
use crate::fl;

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

pub fn summary_view(model: &AppModel) -> Element<'_, Message> {
    if model.all_entries.is_empty() {
        return crate::views::empty_view(model);
    }

    let s = &model.summary;

    let file_info: Element<'_, Message> = if let Some(path) = &model.file_path {
        text(format!("File: {}", path.display())).into()
    } else {
        text("").into()
    };

    let items: Vec<(String, String)> = vec![
        (fl!("total-size"), format_size(s.total_size)),
        (fl!("code-size"), format_size(s.text_size)),
        (fl!("data-size"), format_size(s.data_size + s.rodata_size)),
        (fl!("bss-size"), format_size(s.bss_size)),
        (fl!("label-other"), format_size(s.other_size)),
        (fl!("symbols"), s.num_symbols.to_string()),
        (fl!("label-object-files"), s.num_files.to_string()),
        (fl!("label-section-types"), s.num_sections.to_string()),
    ];

    let rows: Vec<Element<'_, Message>> = items
        .into_iter()
        .map(|(label, value)| {
            row![
                text(label).width(Length::Fixed(180.0)),
                text(value).size(13)
            ]
            .padding([4, 0])
            .into()
        })
        .collect();

    let section_title = container(text(fl!("summary")).size(20)).padding([0, 10]);

    column![file_info, section_title, column(rows).spacing(10)]
        .spacing(20)
        .padding([0, 10])
        .height(Length::Fill)
        .into()
}
