//! Sections page: consolidates section categories and expands to sub-types.

use crate::app::{AppModel, Message, format_size};
use crate::fl;
use crate::model::matches_category;
use crate::views::{cell, divider, header_label, icons, numeric_cell, widths};

use iced::widget::{button, column, row, scrollable, space, text};
use iced::{Alignment, Element, Length};

pub fn sections_view(model: &AppModel) -> Element<'_, Message> {
    if model.section_categories.is_empty() {
        return crate::views::empty_view(model);
    }

    let header = row![
        space::horizontal(),
        header_label(fl!("column-size"), widths::SIZE, Alignment::End),
        header_label(fl!("symbols"), widths::SYMBOLS, Alignment::End),
    ];

    let mut rows: Vec<Element<'_, Message>> =
        Vec::with_capacity(model.section_categories.len() * 2);
    for category in &model.section_categories {
        rows.push(divider());
        let is_open = model.expanded_section.as_deref() == Some(&category.name);
        let arrow = if is_open {
            icons::EXPANDED
        } else {
            icons::COLLAPSED
        };

        let cat_row = button(
            row![
                text(format!("{arrow}  {}", category.name))
                    .size(14)
                    .width(Length::Fill),
                numeric_cell(format_size(category.total_size), widths::SIZE),
                numeric_cell(category.num_symbols.to_string(), widths::SYMBOLS),
            ]
            .padding(0),
        )
        .on_press(Message::ToggleSection(category.name.clone()))
        .width(Length::Fill)
        .style(button::text)
        .padding(0);
        rows.push(cat_row.into());

        if is_open {
            let mut sub_rows: Vec<Element<'_, Message>> = Vec::new();
            for section in &model.section_groups {
                if matches_category(&category.name, &section.name) {
                    sub_rows.push(
                        row![
                            cell(
                                format!("    {}", section.name),
                                Length::Fill,
                                Alignment::Start
                            ),
                            numeric_cell(format_size(section.total_size), widths::SIZE),
                            numeric_cell(section.num_symbols.to_string(), widths::SYMBOLS),
                        ]
                        .into(),
                    );
                }
            }
            if !sub_rows.is_empty() {
                rows.push(column(sub_rows).spacing(1).into());
            }
        }
    }

    let body = scrollable(column(rows).spacing(0)).height(Length::Fill);

    column![header, divider(), body]
        .spacing(0)
        .padding([0, 10])
        .height(Length::Fill)
        .into()
}
