use crate::i18n::Language;
use notify_rust::{Notification, Timeout};

pub fn countdown_finished(language: Language) {
    let _ = Notification::new()
        .summary("Calibre 60")
        .body(language.tr("notification_done"))
        .timeout(Timeout::Milliseconds(8_000))
        .show();
}
