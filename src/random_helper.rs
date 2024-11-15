use chrono::{self, DateTime, SecondsFormat, Utc};
use serde_json::Value;

fn create_entity(id: &str, r#type: &str, datetime: &str, value: &str) -> Value {
    serde_json::json!({
        "id": id,
        "type": r#type,
        "dateObserved": {
            "type": "DateTime",
            "value": datetime,
            "metadata": {},
        },
        "value": {"type": "Number", "value": value, "metadata": {}},
    })
}

fn generate_random_id(size: u16) -> String {
    const HEX_CHARS: [char; 22] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B',
        'C', 'D', 'E', 'F',
    ];
    let mut random_id = String::with_capacity(size as usize + 3);
    random_id.push_str("id-");
    for _ in 0..size {
        random_id.push(HEX_CHARS[rand::random::<usize>() % HEX_CHARS.len()]);
    }
    random_id
}

pub fn generate_simple_time_series(
    min: u16,
    max: u16,
    size: u16,
    measuremnt_interval: Option<u16>,
    type_name: Option<String>,
    metadata: Option<Value>,
) -> Value {
    let mut time_series = Vec::with_capacity(size as usize);
    let time_now: DateTime<Utc> = chrono::Utc::now();
    let r#type = type_name.unwrap_or(String::from("SimpleTimeSeries"));

    for i in 0..size {
        let datetime = time_now
            .checked_add_signed(chrono::Duration::minutes(
                i as i64 * measuremnt_interval.unwrap_or(1) as i64,
            ))
            .unwrap();
        let value = rand::random::<u16>() % (max - min) + min;
        let id = generate_random_id(16);
        let mut entity = create_entity(
            &id,
            &r#type,
            &datetime.to_rfc3339_opts(SecondsFormat::Millis, true),
            &value.to_string(),
        );
        if metadata.is_some() {
            for (key, value) in metadata.as_ref().unwrap().as_object().unwrap() {
                entity[key] = value.clone();
            }
        }
        time_series.push(entity);
    }

    serde_json::json!(time_series)
}
