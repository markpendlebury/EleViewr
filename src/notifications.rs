use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum NotificationType {
    Info,
    Success,
    #[allow(dead_code)]
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    #[allow(dead_code)]
    pub fn with_duration(
        message: String,
        notification_type: NotificationType,
        duration: Duration,
    ) -> Self {
        Self {
            message,
            notification_type,
            created_at: Instant::now(),
            duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn opacity(&self) -> f32 {
        let elapsed = self.created_at.elapsed();
        let total_secs = self.duration.as_secs_f32();
        let elapsed_secs = elapsed.as_secs_f32();

        if elapsed_secs < 0.5 {
            // Fade in over first 0.5 seconds
            elapsed_secs / 0.5
        } else if elapsed_secs > total_secs - 0.5 {
            // Fade out over last 0.5 seconds
            (total_secs - elapsed_secs) / 0.5
        } else {
            1.0
        }
    }
}

pub struct NotificationManager {
    notifications: VecDeque<Notification>,
    max_notifications: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: VecDeque::new(),
            max_notifications: 5,
        }
    }

    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_back(notification);

        // Remove old notifications if we exceed the limit
        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
    }

    pub fn add_info(&mut self, message: String) {
        self.add_notification(Notification::new(message, NotificationType::Info));
    }

    pub fn add_success(&mut self, message: String) {
        self.add_notification(Notification::new(message, NotificationType::Success));
    }

    #[allow(dead_code)]
    pub fn add_warning(&mut self, message: String) {
        self.add_notification(Notification::new(message, NotificationType::Warning));
    }

    pub fn add_error(&mut self, message: String) {
        self.add_notification(Notification::new(message, NotificationType::Error));
    }

    pub fn update(&mut self) {
        // Remove expired notifications
        self.notifications.retain(|n| !n.is_expired());
    }

    pub fn get_notifications(&self) -> &VecDeque<Notification> {
        &self.notifications
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.notifications.clear();
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}
