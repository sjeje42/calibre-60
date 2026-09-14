use crate::notifications;
use crate::settings::{
    AlarmTone, MAX_ALARM_REPEAT_MS, MIN_ALARM_REPEAT_MS,
};
use eframe::egui;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::{
    f32::consts::TAU,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Copy, Debug)]
pub struct AlarmConfig {
    pub tone: AlarmTone,
    pub volume_percent: u8,
    pub repeat_ms: u64,
}

impl AlarmConfig {
    pub fn new(tone: AlarmTone, volume_percent: u8, repeat_ms: u64) -> Self {
        Self {
            tone,
            volume_percent,
            repeat_ms,
        }
        .normalized()
    }

    fn normalized(mut self) -> Self {
        self.volume_percent = self.volume_percent.min(100);
        self.repeat_ms = self
            .repeat_ms
            .clamp(MIN_ALARM_REPEAT_MS, MAX_ALARM_REPEAT_MS);
        self
    }

    fn volume(self, sound_enabled: bool) -> f32 {
        if sound_enabled {
            f32::from(self.volume_percent) / 100.0
        } else {
            0.0
        }
    }
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self::new(AlarmTone::Classic, 35, 1_000)
    }
}

pub enum AudioCommand {
    Arm(Instant),
    Cancel,
    Sound(bool),
    Configure(AlarmConfig),
    Test,
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
            // Keep the output stream alive for the entire worker lifetime.
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
            let mut alarm_sink: Option<Sink> = None;
            let mut preview_sink: Option<Sink> = None;
            let mut sound = true;
            let mut config = AlarmConfig::default();

            loop {
                let wait = deadline
                    .map(|end| end.saturating_duration_since(Instant::now()))
                    .unwrap_or(Duration::from_secs(60));

                match receiver.recv_timeout(wait) {
                    Ok(AudioCommand::Arm(end)) => {
                        stop_sink(&mut alarm_sink);
                        stop_sink(&mut preview_sink);
                        deadline = Some(end);
                    }
                    Ok(AudioCommand::Cancel) => {
                        deadline = None;
                        stop_sink(&mut alarm_sink);
                        stop_sink(&mut preview_sink);
                    }
                    Ok(AudioCommand::Sound(enabled)) => {
                        sound = enabled;
                        let volume = config.volume(sound);
                        if let Some(current) = &alarm_sink {
                            current.set_volume(volume);
                        }
                        if let Some(current) = &preview_sink {
                            current.set_volume(volume);
                        }
                    }
                    Ok(AudioCommand::Configure(next)) => {
                        config = next.normalized();
                        let was_ringing = alarm_sink.is_some();
                        stop_sink(&mut alarm_sink);
                        if was_ringing {
                            if let Some((_, handle)) = &audio {
                                match start_looping_alarm(handle, config, sound) {
                                    Ok(current) => alarm_sink = Some(current),
                                    Err(error) => {
                                        let _ = error_sender.send(error);
                                        ctx.request_repaint();
                                    }
                                }
                            }
                        }
                    }
                    Ok(AudioCommand::Test) => {
                        stop_sink(&mut preview_sink);
                        if sound {
                            if let Some((_, handle)) = &audio {
                                match start_preview(handle, config, sound) {
                                    Ok(current) => preview_sink = Some(current),
                                    Err(error) => {
                                        let _ = error_sender.send(error);
                                        ctx.request_repaint();
                                    }
                                }
                            }
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
                        notifications::countdown_finished();

                        if let Some((_, handle)) = &audio {
                            match start_looping_alarm(handle, config, sound) {
                                Ok(current) => alarm_sink = Some(current),
                                Err(error) => {
                                    let _ = error_sender.send(error);
                                }
                            }
                        }
                        ctx.request_repaint();
                    }
                }
            }

            stop_sink(&mut alarm_sink);
            stop_sink(&mut preview_sink);
        });

        Self {
            commands,
            errors,
            worker: Some(worker),
        }
    }

    pub fn send(&self, command: AudioCommand) {
        let _ = self.commands.send(command);
    }
}

fn stop_sink(sink: &mut Option<Sink>) {
    if let Some(current) = sink.take() {
        current.stop();
    }
}

fn start_looping_alarm(
    handle: &OutputStreamHandle,
    config: AlarmConfig,
    sound_enabled: bool,
) -> Result<Sink, String> {
    let current = Sink::try_new(handle)
        .map_err(|error| format!("Lecture de l'alarme impossible : {error}"))?;
    current.set_volume(config.volume(sound_enabled));
    let samples = build_samples(config.tone, config.repeat_ms);
    let source = rodio::buffer::SamplesBuffer::new(1, SAMPLE_RATE, samples);
    current.append(source.repeat_infinite());
    Ok(current)
}

fn start_preview(
    handle: &OutputStreamHandle,
    config: AlarmConfig,
    sound_enabled: bool,
) -> Result<Sink, String> {
    let current = Sink::try_new(handle)
        .map_err(|error| format!("Test de l'alarme impossible : {error}"))?;
    current.set_volume(config.volume(sound_enabled));
    // The preview is deliberately short: it demonstrates the selected tone
    // without waiting through a long repetition interval.
    let samples = build_samples(config.tone, 600);
    current.append(rodio::buffer::SamplesBuffer::new(
        1,
        SAMPLE_RATE,
        samples,
    ));
    Ok(current)
}

fn build_samples(tone: AlarmTone, total_ms: u64) -> Vec<f32> {
    let sample_count = ((u64::from(SAMPLE_RATE) * total_ms) / 1_000).max(1) as usize;
    (0..sample_count)
        .map(|index| {
            let t = index as f32 / SAMPLE_RATE as f32;
            tone_sample(tone, t).clamp(-1.0, 1.0)
        })
        .collect()
}

fn tone_sample(tone: AlarmTone, t: f32) -> f32 {
    match tone {
        AlarmTone::Classic => pulse(t, 0.0, 0.18, 880.0),
        AlarmTone::DoubleBeep => {
            pulse(t, 0.0, 0.12, 880.0) + pulse(t, 0.20, 0.32, 880.0)
        }
        AlarmTone::Chime => {
            pulse(t, 0.0, 0.16, 659.25) + pulse(t, 0.18, 0.42, 987.77)
        }
        AlarmTone::Digital => {
            pulse(t, 0.0, 0.09, 1_200.0)
                + pulse(t, 0.13, 0.22, 1_600.0)
                + pulse(t, 0.26, 0.35, 1_200.0)
        }
    }
}

fn pulse(t: f32, start: f32, end: f32, frequency: f32) -> f32 {
    if t < start || t >= end {
        return 0.0;
    }

    let local = t - start;
    let length = end - start;
    let envelope = (local / 0.008)
        .min((length - local) / 0.015)
        .clamp(0.0, 1.0);
    (TAU * frequency * local).sin() * envelope * 0.75
}

impl Drop for Alarm {
    fn drop(&mut self) {
        self.send(AudioCommand::Quit);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
