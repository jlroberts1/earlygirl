use iced::keyboard;
use iced::time;
use iced::widget::{button, center, column, progress_bar, row, text};
use iced::{Center, Element, Subscription};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    iced::application("Timer", Timer::update, Timer::view)
        .subscription(Timer::subscription)
        .run()
}

struct Timer {
    current_timer_duration: Duration,
    timer_progress: f32,
    interval: Duration,
    state: State,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            current_timer_duration: Duration::new(0, 0),
            timer_progress: 100 as f32,
            interval: Duration::new(10, 0),
            state: State::Idle,
        }
    }
}

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Ticking {
        last_tick: Instant,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Reset,
    Tick(Instant),
}
fn calculate_progress(current_timer_duration: Duration, interval: Duration) -> f32 {
    if interval.as_secs() == 0 {
        return 0.0;
    }

    let current_secs = current_timer_duration.as_secs() as f32;
    let interval_secs = interval.as_secs() as f32;

    let progress_percentage = if current_secs > interval_secs {
        100.0
    } else {
        (current_secs as f32 / interval_secs as f32) * 100.0
    };

    progress_percentage.clamp(0.0, 100.0)
}

impl Timer {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggle => match self.state {
                State::Idle => {
                    self.state = State::Ticking {
                        last_tick: Instant::now(),
                    };
                }
                State::Ticking { .. } => {
                    self.state = State::Idle;
                }
            },
            Message::Tick(now) => {
                if let State::Ticking { last_tick } = &mut self.state {
                    self.current_timer_duration += now - *last_tick;
                    *last_tick = now;
                    if self.current_timer_duration == self.interval {
                        self.state = State::Idle
                    };
                    self.timer_progress =
                        calculate_progress(self.current_timer_duration, self.interval);
                }
            }
            Message::Reset => {
                self.current_timer_duration = Duration::default();
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = match self.state {
            State::Idle => Subscription::none(),
            State::Ticking { .. } => time::every(Duration::from_millis(10)).map(Message::Tick),
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

    fn view(&self) -> Element<Message> {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        let time_remaining = if (self.interval.as_secs() <= self.current_timer_duration.as_secs()) {
            0
        } else {
            self.interval.as_secs() - self.current_timer_duration.as_secs()
        };

        let duration = text!(
            "{:0>2}:{:0>2}:{:0>2}",
            time_remaining / HOUR,
            (time_remaining % HOUR) / MINUTE,
            time_remaining % MINUTE,
        )
        .size(40);

        let button = |label| button(text(label).align_x(Center)).padding(10).width(80);

        let toggle_button = {
            let label = match self.state {
                State::Idle => "Start",
                State::Ticking { .. } => "Stop",
            };
            button(label).on_press(Message::Toggle)
        };

        let row = row![duration, toggle_button].align_y(Center).spacing(20);

        let content = column![progress_bar(0.0..=100.0, self.timer_progress), row]
            .align_x(Center)
            .padding(20)
            .spacing(20);
        center(content).into()
    }
}
