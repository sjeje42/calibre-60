use notify_rust::{Notification, Timeout};

pub fn countdown_finished() {
    let _ = Notification::new()
        .summary("Calibre 60")
        .body("Le compte à rebours est terminé.")
        .timeout(Timeout::Milliseconds(8_000))
        .show();
}
