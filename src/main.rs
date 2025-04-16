use freedesktop_desktop_entry::{Iter, default_paths, get_languages_from_env};
use freedesktop_icons::lookup;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::{
    ContentFit, Padding, Size, Theme,
    widget::{column, container, image, row, svg, text, text_input},
};
use icon_loader::IconLoader;
use std::collections::HashSet;

struct Astatine {
    search: String,
    applications: Vec<Application>,
    matcher: SkimMatcherV2,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
}

impl Astatine {
    fn new() -> Self {
        Self {
            search: String::from(""),
            applications: get_applications(),
            matcher: SkimMatcherV2::default(),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::SearchChanged(search) => self.search = search,
        };
        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let filtered_applications = if self.search.is_empty() {
            self.applications.clone()
        } else {
            let mut matched_apps: Vec<(i64, Application)> = self
                .applications
                .iter()
                .filter_map(|app| {
                    let score = self.matcher.fuzzy_match(&app.name, &self.search);

                    score.map(|s| (s, app.clone()))
                })
                .collect();

            matched_apps.sort_by(|a, b| b.0.cmp(&a.0));

            matched_apps.into_iter().map(|(_, app)| app).collect()
        };

        let application_list = filtered_applications
            .iter()
            .map(|application| {
                let name = application.name.clone();

                let icon_widget: iced::Element<'_, Message> = match &application.icon {
                    Icon::Svg(path) => svg(path.clone())
                        .width(32)
                        .height(32)
                        .content_fit(ContentFit::ScaleDown)
                        .into(),
                    Icon::Image(path) => image(path.clone())
                        .width(32)
                        .height(32)
                        .content_fit(ContentFit::ScaleDown)
                        .into(),
                };

                row![
                    icon_widget,
                    text(name).align_y(iced::alignment::Vertical::Center)
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center)
                .padding(Padding::from([2, 0]))
            })
            .fold(column![], |col, element| col.push(element));

        container(
            column![
                text_input("", &self.search).on_input(Message::SearchChanged),
                application_list,
            ]
            .spacing(16),
        )
        .padding(Padding::from([12, 24]))
        .into()
    }
}

fn main() -> iced::Result {
    iced::application("Astatine", Astatine::update, Astatine::view)
        .window_size(Size::new(540.0, 648.0))
        .theme(|_| Theme::TokyoNight)
        .run_with(|| (Astatine::new(), iced::Task::none()))
}

#[derive(Clone)]
struct Application {
    name: String,
    exec: String,
    icon: Icon,
}

#[derive(Clone)]
enum Icon {
    Svg(String),
    Image(String),
}

fn get_applications() -> Vec<Application> {
    let locales = get_languages_from_env();
    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    let mut applications = Vec::new();
    let mut seen_execs = HashSet::new();

    let icon_loader = IconLoader::new_gtk().unwrap_or(IconLoader::new());
    let default_icon = icon_loader
        .load_icon("application-x-executable")
        .unwrap()
        .file_for_size(32)
        .path()
        .to_string_lossy()
        .into_owned();

    for entry in entries {
        let name = entry.name(&locales).unwrap().into_owned();
        // Exec is required but some entries ignore that
        let exec = entry.exec().unwrap_or("").to_string();
        let icon_name = entry.icon().unwrap_or("").to_string();

        if name.is_empty() || exec.is_empty() || !seen_execs.insert(exec.clone()) {
            continue;
        }

        let icon = if !icon_name.is_empty() {
            let path = lookup(&icon_name)
                .with_size(32)
                .find()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            if !path.is_empty() {
                if path.ends_with(".svg") {
                    Icon::Svg(path)
                } else {
                    Icon::Image(path)
                }
            } else {
                Icon::Svg(default_icon.clone())
            }
        } else {
            Icon::Svg(default_icon.clone())
        };

        applications.push(Application { name, exec, icon });
    }

    applications
}

