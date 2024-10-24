use iced::keyboard;
use iced::time;
use iced::widget::{button, column, progress_bar, row, text, Column, Container, Text};
use iced::{Center, Element, Length, Subscription, Theme};
use notify_rust::Notification;
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    ::iced::application("Earlygirl", Earlygirl::update, Earlygirl::view)
        .theme(Earlygirl::theme)
        .subscription(Earlygirl::subscription)
        .run()
}

struct Earlygirl {
    theme: Theme,
    current_timer_duration: f64,
    interval: f64,
    timer_type: TimerType,
    timer_state: TimerState,
    work_interval: f64,
    break_interval: f64,
    show_modal: bool,
}

impl Default for Earlygirl {
    fn default() -> Self {
        Self {
            theme: Theme::CatppuccinMocha,
            current_timer_duration: 0.0,
            interval: 10.0 * 60.0,
            timer_type: TimerType::WorkTime,
            timer_state: TimerState::Idle,
            work_interval: 40.0 * 60.0,
            break_interval: 5.0 * 60.0,
            show_modal: false,
        }
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
        last_tick: Instant,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Tick(Instant),
    ToggleSettings,
    WorkIntervalChanged(f64),
    BreakIntervalChanged(f64),
    Reset,
    SwitchWorkType,
}

impl TimerType {
    fn update(&mut self) {
        match self {
            TimerType::WorkTime => *self = TimerType::BreakTime,
            TimerType::BreakTime => *self = TimerType::WorkTime,
        }
    }
}

impl Earlygirl {
    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => match self.timer_state {
                TimerState::Idle => {
                    self.timer_state = TimerState::Ticking {
                        last_tick: Instant::now(),
                    };
                    self.current_timer_duration = 0.0;
                    self.interval = match self.timer_type {
                        TimerType::WorkTime => self.work_interval,
                        TimerType::BreakTime => self.break_interval,
                    };
                }
                TimerState::Ticking { .. } => {
                    self.timer_state = TimerState::Idle;
                }
            },
            Message::Tick(now) => {
                if let TimerState::Ticking { last_tick } = &mut self.timer_state {
                    let time_elapsed = now.duration_since(*last_tick).as_secs_f64();
                    self.current_timer_duration += time_elapsed;
                    *last_tick = now;

                    if self.current_timer_duration >= self.interval {
                        self.timer_state = TimerState::Idle;
                        self.timer_type.update();
                        self.current_timer_duration = 0.0;
                        self.interval = match self.timer_type {
                            TimerType::WorkTime => self.work_interval,
                            TimerType::BreakTime => self.break_interval,
                        };
                        let summary_label = match self.timer_type {
                            TimerType::WorkTime => "Time to get back to work!",
                            TimerType::BreakTime => "Time for a break!",
                        };
                        let _ = Notification::new()
                            .summary(summary_label)
                            .appname("Earlygirl")
                            .show();
                    };
                }
            }
            Message::WorkIntervalChanged(new_interval) => {
                self.work_interval = new_interval * 60.0;
                if let TimerType::WorkTime = self.timer_type {
                    self.interval = self.work_interval;
                }
            }
            Message::BreakIntervalChanged(new_interval) => {
                self.break_interval = new_interval * 60.0;
                if let TimerType::BreakTime = self.timer_type {
                    self.interval = self.break_interval;
                }
            }
            Message::Reset => {
                self.timer_state = TimerState::Idle;
                self.current_timer_duration = 0.0;
                self.interval = self.work_interval;
            }
            Message::SwitchWorkType => {
                self.timer_type.update();
                self.timer_state = TimerState::Idle;
                self.current_timer_duration = 0.0;
                self.interval = match self.timer_type {
                    TimerType::WorkTime => self.work_interval,
                    TimerType::BreakTime => self.break_interval,
                };
            }
            Message::ToggleSettings => {
                self.show_modal = !self.show_modal;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.timer_state {
            TimerState::Idle => Subscription::none(),
            TimerState::Ticking { .. } => time::every(Duration::from_millis(10)).map(Message::Tick),
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
            self.work_interval / MINUTE,
            Message::WorkIntervalChanged,
        )
        .step(5)
        .width(200);

        let break_slider = iced::widget::slider(
            5.0..=60.0,
            self.break_interval / MINUTE,
            Message::BreakIntervalChanged,
        )
        .step(5)
        .width(200);

        Column::new()
            .spacing(20)
            .padding(20)
            .push(Text::new("Set Work Time (minutes)"))
            .push(work_slider)
            .push(Text::new("Set Break Time(minutes)"))
            .push(break_slider)
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
            button(text(label)).on_press(Message::ToggleSettings)
        };

        let start_pause_button = {
            let label = match self.timer_state {
                TimerState::Idle => "Start",
                TimerState::Ticking { .. } => "Pause",
            };
            button(label)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => {
                            button::Style::default().with_background(palette.success.strong.color)
                        }
                        _ => button::primary(theme, status),
                    }
                })
                .on_press(Message::Toggle)
        };

        let reset_button = button("Reset")
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => {
                        button::Style::default().with_background(palette.secondary.strong.color)
                    }
                    _ => button::primary(theme, status),
                }
            })
            .on_press(Message::Reset);

        let switch_timer_type_button = button("Switch").on_press(Message::SwitchWorkType);

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

        let row = row![start_pause_button, switch_timer_type_button, reset_button,].spacing(20);
        let mut content = column![timer_label, duration, progress_bar, row,]
            .align_x(Center)
            .padding(20)
            .spacing(20);

        content = content.push(settings_button);

        if self.show_modal {
            let modal = self.settings_modal();
            content = content.push(modal);
        }

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
