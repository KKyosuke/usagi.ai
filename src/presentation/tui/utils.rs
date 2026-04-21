use chrono::TimeZone;

pub fn format_modified_at(time: &str) -> String {
    if let Some(base_str) = time.strip_suffix(" UTC") {
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(base_str, "%Y-%m-%d %H:%M") {
            let utc_dt = chrono::Utc.from_utc_datetime(&naive_dt);
            let local_dt: chrono::DateTime<chrono::Local> = utc_dt.with_timezone(&chrono::Local);
            return format!("modified: {}", local_dt.format("%Y/%m/%d %H:%M"));
        }
    }
    format!("modified: {}", time)
}
