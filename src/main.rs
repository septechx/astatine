use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::{
    Size,
    widget::{column, text, text_input},
};
use std::fs;

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
            .map(|application| -> iced::Element<'_, Message> {
                text(application.name.clone()).into()
            })
            .fold(column![], |col, element| col.push(element));

        column![
            text_input("", &self.search).on_input(Message::SearchChanged),
            application_list
        ]
        .into()
    }
}

fn main() -> iced::Result {
    iced::application("Astatine", Astatine::update, Astatine::view)
        .window_size(Size::new(720.0, 640.0))
        .run_with(|| (Astatine::new(), iced::Task::none()))
}

#[derive(Default, Clone)]
struct Application {
    name: String,
    exec: String,
}

impl Application {
    fn from(desktop_entry: String) -> Option<Self> {
        let mut application = Self::default();
        let mut is_application = false;
        let mut should_display = true;
        for line in desktop_entry.lines() {
            if line.starts_with("Name=") {
                application.name = get_desktop_entry_line_value(line);
            } else if line.starts_with("Exec=") {
                application.exec = get_desktop_entry_line_value(line);
            } else if line.starts_with("NoDisplay=true") || line.starts_with("Hidden=true") {
                should_display = false;
            } else if line.starts_with("Type=Application") {
                is_application = true;
            }
        }
        if is_application
            && should_display
            && !application.name.is_empty()
            && !application.exec.is_empty()
        {
            Some(application)
        } else {
            None
        }
    }
}

fn get_desktop_entry_line_value(line: &str) -> String {
    line.split_once("=").unwrap().1.to_string()
}

fn get_applications() -> Vec<Application> {
    let mut applications = Vec::new();

    let paths = vec![
        String::from("/usr/share/applications"),
        String::from("/var/lib/flatpak/exports/share/applications"),
        format!(
            "{}/.local/share/flatpak/exports/share/applications",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for path in paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if !path.to_string_lossy().ends_with(".desktop") {
                        continue;
                    }

                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Some(app) = Application::from(content) {
                            applications.push(app);
                        }
                    }
                }
            }
        }
    }

    applications
}
