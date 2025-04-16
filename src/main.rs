use freedesktop_desktop_entry::{Iter, default_paths, get_languages_from_env};
use freedesktop_icons::lookup;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::{
    Background, Color, ContentFit, Padding, Size, Subscription, Task, Theme, keyboard,
    widget::{button, column, container, image, row, svg, text, text_input},
};
use icon_loader::IconLoader;
use std::collections::HashSet;
use std::process::Command;

struct Astatine {
    search: String,
    applications: Vec<Application>,
    matcher: SkimMatcherV2,
    focus: usize,
    prev_focus: Option<usize>,
}

#[derive(Debug, Clone)]
enum Message {
    SearchChanged(String),
    KeyPressed(String),
}

trait MessageProcessor<T> {
    fn process(state: &mut Astatine, param: T) -> Task<Message>;
}

struct SearchChangedProcessor;
impl MessageProcessor<String> for SearchChangedProcessor {
    fn process(state: &mut Astatine, param: String) -> Task<Message> {
        state.search = param;
        state.prev_focus = None;
        state.focus = 0;
        Task::none()
    }
}

struct KeyPressedProcessor;
impl MessageProcessor<String> for KeyPressedProcessor {
    fn process(state: &mut Astatine, param: String) -> Task<Message> {
        match param.as_str() {
            "j" => {
                if let Some(prev_focus) = state.prev_focus {
                    state.focus = prev_focus;
                    state.prev_focus = None;
                }
                state.focus = state.focus.saturating_add(1);
            }
            "k" => {
                if let Some(prev_focus) = state.prev_focus {
                    state.focus = prev_focus;
                    state.prev_focus = None;
                }
                state.focus = state.focus.saturating_sub(1);
            }
            "i" => {
                state.prev_focus = Some(state.focus);
                state.focus = 0;
            }
            "/" => {
                state.prev_focus = Some(state.focus);
                state.focus = 0;
            }
            "<enter>" => {
                let filtered_applications = if state.search.is_empty() {
                    state.applications.clone()
                } else {
                    let mut matched_apps: Vec<(i64, Application)> = state
                        .applications
                        .iter()
                        .filter_map(|app| {
                            let score = state.matcher.fuzzy_match(&app.name, &state.search);

                            score.map(|s| (s, app.clone()))
                        })
                        .collect();

                    matched_apps.sort_by(|a, b| b.0.cmp(&a.0));

                    matched_apps.into_iter().map(|(_, app)| app).collect()
                };

                let exec = filtered_applications
                    .iter()
                    .enumerate()
                    .find(|(i, _)| i + 1 == state.focus)
                    .unwrap()
                    .1
                    .exec
                    .clone();

                execute_app_exec(exec);
            }
            _ => (),
        };

        if state.focus == 0 {
            return text_input::focus("search");
        }

        Task::none()
    }
}
impl Astatine {
    fn new() -> Self {
        Self {
            search: String::from(""),
            applications: get_applications(),
            matcher: SkimMatcherV2::default(),
            focus: 1,
            prev_focus: None,
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::SearchChanged(param) => SearchChangedProcessor::process(self, param),
            Message::KeyPressed(param) => KeyPressedProcessor::process(self, param),
        }
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
            .enumerate()
            .map(|(i, application)| {
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

                button(
                    row![
                        icon_widget,
                        text(name).align_y(iced::alignment::Vertical::Center)
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center)
                    .padding(Padding::from([2, 0])),
                )
                .style(move |_, _| button::Style {
                    background: if i + 1 == self.focus {
                        Some(Background::Color(Color::from_rgb8(169, 177, 214)))
                    } else {
                        None
                    },
                    border: iced::Border {
                        color: Color::from_rgba8(0, 0, 0, 0.0),
                        width: 1.0,
                        radius: iced::border::Radius::new(10),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba8(0, 0, 0, 0.0),
                        offset: iced::Vector::new(0.0, 0.0),
                        blur_radius: 0.0,
                    },
                    text_color: if i + 1 == self.focus {
                        Color::from_rgb8(26, 27, 38)
                    } else {
                        Color::from_rgb8(169, 177, 214)
                    },
                })
            })
            .fold(column![], |col, element| col.push(element));

        container(
            column![
                text_input("", &self.search)
                    .on_input(Message::SearchChanged)
                    .id("search"),
                application_list,
            ]
            .spacing(16),
        )
        .padding(Padding::from([12, 24]))
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, _| match key {
            keyboard::Key::Character(character) => Some(Message::KeyPressed(character.to_string())),
            keyboard::Key::Named(keyboard::key::Named::Enter) => {
                Some(Message::KeyPressed(String::from("<enter>")))
            }
            _ => None,
        })
    }
}

fn main() -> iced::Result {
    iced::application("Astatine", Astatine::update, Astatine::view)
        .window_size(Size::new(540.0, 648.0))
        .theme(|_| Theme::TokyoNight)
        .subscription(Astatine::subscription)
        .run_with(|| (Astatine::new(), iced::Task::none()))
}

fn execute_app_exec(exec: String) {
    let mut parts = exec.split_whitespace();
    if let Some(program) = parts.next() {
        let args: Vec<&str> = parts.collect();
        let _ = Command::new(program).args(args).spawn();
    } else {
        eprintln!("No command provided.");
    }
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
