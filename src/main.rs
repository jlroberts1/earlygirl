extern crate preferences;
use iced::widget::{
    button, center, container, mouse_area, opaque, progress_bar, row, stack, text, Column,
    Container, Row, Text,
};
use iced::{keyboard, time, Center, Color, Element, Length, Subscription, Theme};
use notify_rust::Notification;
use preferences::{AppInfo, Preferences};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

const APP_INFO: AppInfo = AppInfo {
    name: "Earlygirl",
    author: "Earlygirl",
};
const PREFS_KEY: &str = "earlygirl_preferences";

fn main() -> iced::Result {
    let window_settings = iced::window::Settings {
        size: iced::Size::new(650.0, 400.0),
        resizable: true,
        ..Default::default()
    };

    ::iced::application("Earlygirl", Earlygirl::update, Earlygirl::view)
        .theme(Earlygirl::theme)
        .subscription(Earlygirl::subscription)
        .window(window_settings)
        .run()
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct EarlyGirlPreferences {
    work_interval: f64,
    break_interval: f64,
    auto_start_work: bool,
    auto_start_break: bool,
}

impl Default for EarlyGirlPreferences {
    fn default() -> Self {
        Self {
            work_interval: 45.0 * 60.0,
            break_interval: 15.0 * 60.0,
            auto_start_work: false,
            auto_start_break: false,
        }
    }
}

struct Earlygirl {
    theme: Theme,
    current_timer_duration: f64,
    interval: f64,
    timer_type: TimerType,
    timer_state: TimerState,
    preferences: EarlyGirlPreferences,
    show_modal: bool,
}

impl Default for Earlygirl {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum TimerType {
    WorkTime,
    BreakTime,
}

#[derive(Default)]
enum TimerState {
    #[default]
    Idle,
    Ticking {
        last_tick: SystemTime,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Tick(SystemTime),
    ToggleSettings,
    WorkIntervalChanged(f64),
    BreakIntervalChanged(f64),
    AutoStartWorkChanged(bool),
    AutoStartBreakChanged(bool),
    Reset,
    SwitchWorkType,
}

impl Earlygirl {
    fn new() -> Self {
        let preferences = EarlyGirlPreferences::load(&APP_INFO, PREFS_KEY).unwrap_or_default();

        let timer_state = TimerState::Idle;
        let timer_type = TimerType::WorkTime;
        let interval = preferences.work_interval;

        Self {
            theme: Theme::default(),
            current_timer_duration: 0.0,
            interval,
            timer_type,
            timer_state,
            preferences,
            show_modal: false,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => match self.timer_state {
                TimerState::Idle => {
                    self.timer_state = TimerState::Ticking {
                        last_tick: SystemTime::now(),
                    };
                    self.current_timer_duration = 0.0;
                    self.set_interval_for_work_type()
                }
                TimerState::Ticking { .. } => {
                    self.timer_state = TimerState::Idle;
                }
            },
            Message::Tick(now) => {
                if let TimerState::Ticking { last_tick } = &mut self.timer_state {
                    if let Ok(time_elapsed) = now.duration_since(*last_tick) {
                        let elapsed_secs = time_elapsed.as_secs_f64();
                        self.current_timer_duration += elapsed_secs;
                        *last_tick = now;
                    }

                    if self.current_timer_duration >= self.interval {
                        self.send_notification();
                        self.toggle_work_type();
                    };
                }
            }
            Message::WorkIntervalChanged(new_interval) => {
                self.preferences.work_interval = new_interval * 60.0;
                self.write_preferences();
                self.set_interval_for_work_type();
            }
            Message::BreakIntervalChanged(new_interval) => {
                self.preferences.break_interval = new_interval * 60.0;
                self.write_preferences();
                self.set_interval_for_work_type();
            }
            Message::Reset => self.reset_timer(),
            Message::SwitchWorkType => self.toggle_work_type(),
            Message::ToggleSettings => self.show_modal = !self.show_modal,
            Message::AutoStartWorkChanged(new_value) => {
                self.preferences.auto_start_work = new_value;
                self.write_preferences();
            }
            Message::AutoStartBreakChanged(new_value) => {
                self.preferences.auto_start_break = new_value;
                self.write_preferences();
            }
        }
    }

    fn send_notification(&self) {
        let message = match self.timer_type {
            TimerType::WorkTime => "Time to get back to work!",
            TimerType::BreakTime => "Time for a break!",
        };
        let _ = Notification::new()
            .summary(message)
            .appname("Earlygirl")
            .show();
    }

    fn reset_timer(&mut self) {
        self.timer_state = TimerState::Idle;
        self.current_timer_duration = 0.0;
        self.set_interval_for_work_type();
    }

    fn toggle_work_type(&mut self) {
        match self.timer_type {
            TimerType::WorkTime => {
                self.timer_type = TimerType::BreakTime;
                self.interval = self.preferences.break_interval;
                if !self.preferences.auto_start_break {
                    self.timer_state = TimerState::Idle;
                }
            }
            TimerType::BreakTime => {
                self.timer_type = TimerType::WorkTime;
                self.interval = self.preferences.work_interval;
                if !self.preferences.auto_start_work {
                    self.timer_state = TimerState::Idle;
                }
            }
        };
        self.current_timer_duration = 0.0;
    }

    fn set_interval_for_work_type(&mut self) {
        match self.timer_type {
            TimerType::WorkTime => self.interval = self.preferences.work_interval,
            TimerType::BreakTime => self.interval = self.preferences.break_interval,
        }
    }

    fn write_preferences(&self) {
        let save_result = self.preferences.save(&APP_INFO, PREFS_KEY);
        assert!(save_result.is_ok());
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.timer_state {
            TimerState::Idle => Subscription::none(),
            TimerState::Ticking { .. } => {
                time::every(Duration::from_millis(10)).map(|_| Message::Tick(SystemTime::now()))
            }
        };

        fn handle_hotkey(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            use keyboard::key;

            match key.as_ref() {
                keyboard::Key::Named(key::Named::Space) => Some(Message::Toggle),
                keyboard::Key::Character("r") => Some(Message::Reset),
                _ => None,
            }
        }

        Subscription::batch(vec![tick, keyboard::on_key_press(handle_hotkey)])
    }

    fn settings_modal(&self) -> Element<Message> {
        const MINUTE: f64 = 60.0;
        let work_slider = iced::widget::slider(
            5.0..=60.0,
            self.preferences.work_interval / MINUTE,
            Message::WorkIntervalChanged,
        )
        .step(5)
        .width(200);
        let break_slider = iced::widget::slider(
            5.0..=60.0,
            self.preferences.break_interval / MINUTE,
            Message::BreakIntervalChanged,
        )
        .step(5)
        .width(200);

        let auto_start_work =
            iced::widget::checkbox("Auto start work", self.preferences.auto_start_work)
                .on_toggle(Message::AutoStartWorkChanged);

        let auto_start_break =
            iced::widget::checkbox("Auto start break", self.preferences.auto_start_break)
                .on_toggle(Message::AutoStartBreakChanged);
        let work_value = self.preferences.work_interval / MINUTE;
        let work_widget = row![Text::new(format!("{work_value} minutes"))].padding([0, 10]);
        let break_value = self.preferences.break_interval / MINUTE;
        let break_label = row![Text::new(format!("{break_value} minutes"))].padding([0, 10]);
        let close_button = button("Close").on_press(Message::ToggleSettings);
        Column::new()
            .spacing(20)
            .padding(20)
            .push(Text::new("Set Work Time"))
            .push(row![work_slider, work_widget,])
            .push(Text::new("Set Break Time"))
            .push(row![break_slider, break_label,])
            .push(auto_start_work)
            .push(auto_start_break)
            .push(close_button)
            .into()
    }

    fn view(&self) -> Element<Message> {
        const MINUTE: f64 = 60.0;
        const HOUR: f64 = 60.0 * MINUTE;

        let time_remaining = if self.interval <= self.current_timer_duration {
            0.0
        } else {
            self.interval - self.current_timer_duration
        };
        let duration = text!(
            "{:0>2}:{:0>2}:{:0>2}",
            (time_remaining / HOUR).floor(),
            ((time_remaining % HOUR) / MINUTE).floor(),
            (time_remaining % MINUTE).floor()
        )
        .size(80);

        let settings_button = {
            let label = match self.show_modal {
                true => "Hide Settings",
                false => "Show Settings",
            };
            timer_button(label, || Message::ToggleSettings)
        };

        let start_pause_button = {
            let label = match self.timer_state {
                TimerState::Idle => "Start",
                TimerState::Ticking { .. } => "Pause",
            };
            timer_button(label, || Message::Toggle)
        };

        let reset_button = timer_button("Reset", || Message::Reset);

        let switch_timer_type_button = timer_button("Switch", || Message::SwitchWorkType);

        let working_label = match self.timer_state {
            TimerState::Ticking { .. } => "Working!",
            TimerState::Idle => "Start Working!",
        };

        let timer_label = {
            let label = match self.timer_type {
                TimerType::WorkTime => working_label,
                TimerType::BreakTime => "Break Time!",
            };
            text(label).size(30)
        };

        let timer_progress = (self.current_timer_duration / self.interval) * 100.0;
        let progress_bar = progress_bar(0.0..=100.0, timer_progress as f32);

        let row = Row::new()
            .spacing(20)
            .push(start_pause_button)
            .push(switch_timer_type_button)
            .push(reset_button);

        let column = Column::new()
            .align_x(Center)
            .spacing(20)
            .padding(20)
            .push(timer_label)
            .push(duration)
            .push(progress_bar)
            .push(row)
            .push(settings_button);

        if self.show_modal {
            let model = container(self.settings_modal())
                .padding(10)
                .style(container::rounded_box);
            modal(column, model, Message::ToggleSettings)
        } else {
            Container::new(column)
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}

fn timer_button<'a>(
    text: impl Into<Element<'a, Message>>,
    on_press: impl Fn() -> Message + 'a,
) -> Element<'a, Message> {
    iced::widget::button::Button::new(text)
        .on_press(on_press())
        .into()
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
