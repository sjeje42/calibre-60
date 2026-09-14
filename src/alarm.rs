use eframe::egui;
use rodio::{OutputStream, Sink, Source};
use std::{
    f32::consts::TAU,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub enum AudioCommand {
    Arm(Instant),
    Cancel,
    Sound(bool),
    Quit,
}

/// Keeps the alarm independent of window repainting and visibility.
pub struct Alarm {
    commands: Sender<AudioCommand>,
    pub errors: Receiver<String>,
    worker: Option<JoinHandle<()>>,
}

impl Alarm {
    pub fn new(ctx: egui::Context) -> Self {
        let (commands, receiver) = mpsc::channel();
        let (error_sender, errors) = mpsc::channel();

        let worker = thread::spawn(move || {
            // Keep the output stream alive for the entire playback.
            let audio = match OutputStream::try_default() {
                Ok(output) => Some(output),
                Err(error) => {
                    let _ = error_sender.send(format!(
                        "Sortie audio indisponible : {error}"
                    ));
                    ctx.request_repaint();
                    None
                }
            };

            let mut deadline: Option<Instant> = None;
            let mut sink: Option<Sink> = None;
            let mut sound = true;

            loop {
                let wait = deadline
                    .map(|end| end.saturating_duration_since(Instant::now()))
                    .unwrap_or(Duration::from_secs(60));

                match receiver.recv_timeout(wait) {
                    Ok(AudioCommand::Arm(end)) => {
                        if let Some(previous) = sink.take() {
                            previous.stop();
                        }
                        deadline = Some(end);
                    }
                    Ok(AudioCommand::Cancel) => {
                        deadline = None;
                        if let Some(previous) = sink.take() {
                            previous.stop();
                        }
                    }
                    Ok(AudioCommand::Sound(enabled)) => {
                        sound = enabled;
                        if let Some(current) = &sink {
                            current.set_volume(if sound { 0.35 } else { 0.0 });
                        }
                    }
                    Ok(AudioCommand::Quit)
                    | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {
                        let expired = deadline
                            .map(|end| Instant::now() >= end)
                            .unwrap_or(false);
                        if !expired {
                            continue;
                        }
                        deadline = None;

                        if let Some((_, handle)) = &audio {
                            match Sink::try_new(handle) {
                                Ok(current) => {
                                    // 180 ms tone, then silence; repeat every second.
                                    // A short envelope avoids hard audio clicks.
                                    let samples: Vec<f32> = (0..44_100)
                                        .map(|index| {
                                            let t = index as f32 / 44_100.0;
                                            if t >= 0.18 {
                                                return 0.0;
                                            }
                                            let envelope = (t / 0.008)
                                                .min((0.18 - t) / 0.015)
                                                .clamp(0.0, 1.0);
                                            (TAU * 880.0 * t).sin() * envelope
                                        })
                                        .collect();
                                    let source = rodio::buffer::SamplesBuffer::new(
                                        1, 44_100, samples,
                                    );
                                    current.set_volume(if sound { 0.35 } else { 0.0 });
                                    current.append(source.repeat_infinite());
                                    sink = Some(current);
                                }
                                Err(error) => {
                                    let _ = error_sender.send(format!(
                                        "Lecture de l'alarme impossible : {error}"
                                    ));
                                }
                            }
                        }
                        ctx.request_repaint();
                    }
                }
            }

            if let Some(current) = sink {
                current.stop();
            }
        });

        Self { commands, errors, worker: Some(worker) }
    }

    pub fn send(&self, command: AudioCommand) {
        let _ = self.commands.send(command);
    }
}

impl Drop for Alarm {
    fn drop(&mut self) {
        self.send(AudioCommand::Quit);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
