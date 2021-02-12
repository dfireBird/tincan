use crossterm::event::{self, Event as CEvent, KeyEvent};

use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const TICK_RATE: u64 = 250;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<Event<KeyEvent>>,
    poll_handle: thread::JoinHandle<()>,
}

impl Events {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(TICK_RATE);

        let poll_handle = thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // poll for tick raet duration, if no events send tick event
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap() {
                    if let CEvent::Key(key) = event::read().unwrap() {
                        tx.send(Event::Input(key)).unwrap();
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).unwrap();
                    last_tick = Instant::now();
                }
            }
        });

        Events { rx, poll_handle }
    }

    pub fn next(&self) -> Result<Event<KeyEvent>, mpsc::RecvError> {
        self.rx.recv()
    }
}
