use std::{
    io::{self, Result, Stderr, Write},
    time::{Duration, Instant},
};

use crossbeam::channel::Receiver;
use crossterm::{
    cursor, execute,
    style::{self, Color, PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
};

pub fn stats_loop(silent: bool, stats_rx: Receiver<usize>) -> Result<()> {
    let mut total_bytes = 0;
    let start = Instant::now();
    let mut timer = Timer::new();
    loop {
        // todo: receive the vector of bytes
        let num_bytes = stats_rx.recv().unwrap();
        timer.update();
        let rate_per_second = num_bytes as f64 / timer.delta.as_secs_f64();

        let mut stderr = io::stderr();
        total_bytes += num_bytes;
        if !silent && timer.ready {
            timer.ready = false;

            output_progress(
                &mut stderr,
                total_bytes,
                start.elapsed().as_secs().as_time(),
                rate_per_second,
            );
            // eprint!(
            //     "\r{} {} [{:.0}b/s]",
            //     total_bytes,
            //     start.elapsed().as_secs().as_time(),
            //     rate_per_second
            // );
        }

        if num_bytes == 0 {
            break;
        }
    }
    if !silent {
        eprintln!();
    }

    Ok(())
}

trait TimeOutput {
    fn as_time(&self) -> String;
}

impl TimeOutput for u64 {
    fn as_time(&self) -> String {
        let (hours, left) = (*self / 3600, *self % 3600);
        let (minutes, seconds) = (left / 60, left % 60);

        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

struct Timer {
    last_instant: Instant,
    delta: Duration,     //difference between now and last_instant
    period: Duration,    //How often we want the timer to go off, e.g every 1sec
    countdown: Duration, //how much time till the timer goes off next
    ready: bool,         // Indicates that timer is ready or gone off
}

impl Timer {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_instant: now,
            delta: Duration::default(),
            period: Duration::from_millis(1000),
            countdown: Duration::default(),
            ready: true,
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        self.delta = now - self.last_instant;
        self.last_instant = now;
        self.countdown = self.countdown.checked_sub(self.delta).unwrap_or_else(|| {
            self.ready = true;
            self.period
        });
    }
}

fn output_progress(stderr: &mut Stderr, bytes: usize, elapsed: String, rate: f64) {
    let bytes = style::style(format!("{} ", bytes)).with(Color::Red).bold();
    let elapsed = style::style(elapsed).with(Color::Green).bold();
    let rate = style::style(format!(" [{:.0}b/s]", rate)).with(Color::Blue).bold().italic();

    let _ = execute!(
        stderr,
        cursor::MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        PrintStyledContent(bytes),
        PrintStyledContent(elapsed),
        PrintStyledContent(rate)
    );

    let _ = stderr.flush();
}
