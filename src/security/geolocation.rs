#[derive(Debug, Clone, PartialEq)]
pub struct GeoPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
}

pub struct GeoManager;

impl GeoManager {
    pub fn get_current_position() -> GeoPosition {
        GeoPosition {
            latitude: 23.8103,
            longitude: 90.4125,
            accuracy: 10.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotificationItem {
    pub title: String,
    pub body: String,
}

pub struct NotificationManager;

impl NotificationManager {
    pub fn show_notification(title: &str, body: &str) -> NotificationItem {
        println!("🔔 OS DESKTOP NOTIFICATION: [{}] {}", title, body);
        NotificationItem {
            title: title.to_string(),
            body: body.to_string(),
        }
    }
}
