use freedesktop_desktop_entry::{Iter, default_paths, get_languages_from_env};
use freedesktop_icons::lookup;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::{
    ContentFit, Size,
    widget::{column, image, row, svg, text, text_input},
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

        let icon_loader = IconLoader::new_gtk().unwrap_or(IconLoader::new());

        let application_list = filtered_applications
            .iter()
            .map(|application| {
                let name = application.name.clone();
                let icon = application.icon.clone();

                let icon_widget: iced::Element<'_, Message> = if icon.is_empty() {
                    let icon = icon_loader
                        .load_icon("application-x-executable")
                        .unwrap()
                        .file_for_size(32)
                        .path()
                        .to_string_lossy()
                        .into_owned();

                    svg(icon)
                        .width(32)
                        .height(32)
                        .content_fit(ContentFit::ScaleDown)
                        .into()
                } else {
                    image(icon)
                        .width(32)
                        .height(32)
                        .content_fit(ContentFit::ScaleDown)
                        .into()
                };

                row![icon_widget, text(name).center()].spacing(10)
            })
            .fold(column![], |col, element| col.push(element));

        column![
            text_input("", &self.search).on_input(Message::SearchChanged),
            application_list,
        ]
        .into()
    }
}

fn main() -> iced::Result {
    iced::application("Astatine", Astatine::update, Astatine::view)
        .window_size(Size::new(720.0, 640.0))
        .run_with(|| (Astatine::new(), iced::Task::none()))
}

#[derive(Clone)]
struct Application {
    name: String,
    exec: String,
    icon: String,
}

fn get_applications() -> Vec<Application> {
    let locales = get_languages_from_env();
    let entries = Iter::new(default_paths())
        .entries(Some(&locales))
        .collect::<Vec<_>>();

    let mut applications = Vec::new();
    let mut seen_execs = HashSet::new();

    for entry in entries {
        let name = entry
            .name(&locales)
            .unwrap_or(std::borrow::Cow::Borrowed(""))
            .to_string();
        let exec = entry.exec().unwrap_or("").to_string();
        let icon_name = entry.icon().unwrap_or("").to_string();

        if name.is_empty() || exec.is_empty() || !seen_execs.insert(exec.clone()) {
            continue;
        }

        let icon = if !icon_name.is_empty() {
            lookup(&icon_name)
                .with_size(32)
                .find()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
        } else {
            String::new()
        };

        applications.push(Application { name, exec, icon });
    }

    applications
}

