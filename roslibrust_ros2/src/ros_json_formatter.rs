/// ROS2 requires a very specific JSON format for hashing:
/// - Single line
/// - Spaces after every control : and ,

use std::io::Write;
use serde::Serialize;
use serde_json::{ser::Formatter, Value};

pub fn to_ros2_json<T: Serialize>(v: T) -> String {
    let mut buf = Vec::new();
    {
        let mut ser = serde_json::Serializer::with_formatter(
            &mut buf,
            Ros2Formatter::default(),
        );
        v.serialize(&mut ser).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

#[derive(Default)]
struct Ros2Formatter;

impl Formatter for Ros2Formatter {
    fn begin_object_value<W: ?Sized + Write>(&mut self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(b": ")
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> std::io::Result<()>
    where
        W: ?Sized + Write,
    {
        if first {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }
}
