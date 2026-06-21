mod app;
mod i18n;
mod model;
mod views;

fn main() -> iced::Result {
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    iced::application(
        app::AppModel::new,
        app::AppModel::update,
        app::AppModel::view,
    )
    .title(app::AppModel::title)
    .theme(app::AppModel::theme)
    .window(iced::window::Settings {
        size: iced::Size::new(960.0, 640.0),
        min_size: Some(iced::Size::new(360.0, 180.0)),
        ..Default::default()
    })
    .run()
}
