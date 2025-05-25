use core::fmt::Arguments;
use log::{Level, Record, kv::*};
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri_plugin_log::fern::FormatCallback;

struct KeyValueVisitor<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

impl<'kvs> VisitSource<'kvs> for KeyValueVisitor<'kvs> {
    #[inline]
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        self.0.insert(key, value);
        Ok(())
    }
}

/// This function is used to format log in a JSON format.
pub fn formatter(out: FormatCallback, _msg: &Arguments, record: &Record) {
    let kvs = record.key_values();
    let mut visitor = KeyValueVisitor(BTreeMap::new());
    let _ = kvs.visit(&mut visitor);
    visitor
        .0
        .insert(Key::from("target"), Value::from(record.target()));

    let args = record.args();
    let msg: String;
    if let Some(msg) = args.as_str() {
        visitor.0.insert(Key::from("message"), Value::from(msg));
    } else {
        msg = args.to_string();
        visitor.0.insert(Key::from("message"), Value::from(&msg));
    }

    let level = record.level();
    visitor
        .0
        .insert(Key::from("level"), Value::from(level.as_str()));

    if level <= Level::Warn {
        if let Some(val) = record.module_path() {
            visitor.0.insert(Key::from("module"), Value::from(val));
        }
        if let Some(val) = record.file() {
            visitor.0.insert(Key::from("file"), Value::from(val));
        }
        if let Some(val) = record.line() {
            visitor.0.insert(Key::from("line"), Value::from(val));
        }
    }

    visitor
        .0
        .insert(Key::from("timestamp"), Value::from(unix_ms()));
    match serde_json::to_string(&visitor.0) {
        Ok(log) => out.finish(format_args!("{}", log)),
        Err(_) => out.finish(format_args!("{:?}", visitor.0)),
    }
}

#[inline]
fn unix_ms() -> u64 {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before Unix epoch");
    ts.as_millis() as u64
}
