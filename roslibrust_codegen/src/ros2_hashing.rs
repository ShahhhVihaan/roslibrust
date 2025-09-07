//! Module for calculating the ROS2 hash of a message definition via https://github.com/ros-infrastructure/rep/pull/381/files

/// The following structs define the format of the JSON file used for hashing in ROS2
/// A quick description of the process to calculate a ROS2 hash:
///  - ROS2 parses the .msg / .srv files
///  - Covert them to the below struct formats
///  - Generate a specifically formatted JSON string of the below structs
///  - Calculate the sha256 hash of the JSON string (utf-8)
///  - Generate a string of the format RIHS01_<hex hash>
#[derive(serde::Deserialize, serde::Serialize)]
pub struct TypeDescriptionFile {
    type_description_msg: TypeDescriptionMsg,
    type_hashes: Vec<TypeHash>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TypeDescriptionMsg {
    type_description: TypeDescription,
    referenced_type_descriptions: Vec<TypeDescription>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TypeDescription {
    type_name: String,
    fields: Vec<Field>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Field {
    name: String,
    #[serde(rename = "type")]
    field_type: FieldType,
    // Value is not used in hashing, included for completeness
    #[serde(skip)]
    _default_value: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FieldType {
    type_id: u8,
    capacity: u32,
    string_capacity: u32,
    nested_type_name: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TypeHash {
    type_name: String,
    hash_string: String,
}

/// Calculates the ROS2 hash of a message definition, if possible
/// Returns None if the message definition is not hashable (e.g. contains nested types not present in the graph)
pub fn calculate_ros2_hash(
    parsed: &ParsedMessageFile,
    graph: &BTreeMap<String, MessageFile>,
) -> Option<String> {
    let type_description_msg = convert_to_type_description(parsed, graph)?;
    Some(calculate_hash(&type_description_msg))
}

/// Calculates the hash from a TypeDescriptionMsg struct
fn calculate_hash(type_description_msg: &TypeDescriptionMsg) -> String {
    let type_description_string = to_ros2_json(type_description_msg);
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(type_description_string);
    let result = hasher.finalize();
    format!("RIHS01_{result:x}")
}

/// Taking in a parsed representation of a message convert it to the ROS2 specific type representation
fn convert_to_type_description(
    parsed: &ParsedMessageFile,
    graph: &BTreeMap<String, MessageFile>,
) -> Option<TypeDescriptionMsg> {
    let mut fields = vec![];
    let mut referenced_type_descriptions = vec![];

    for field in &parsed.fields {
        let nested_type_name = if field.field_type.is_primitive() {
            "".to_string()
        } else {
            field.get_ros2_full_type_name()
        };

        fields.push(Field {
            name: field.field_name.clone(),
            field_type: FieldType {
                type_id: get_field_type_id(&field.field_type),
                // TODO ignoring capacity and string_capacity for now
                capacity: 0,
                string_capacity: 0,
                nested_type_name,
            },
            // Default value is not used in hashing, but included for completeness
            _default_value: "".to_string(),
        });

        if !field.field_type.is_primitive() {
            let sub_message = graph.get(field.get_full_type_name().as_str())?;
            let sub_type_description = convert_to_type_description(&sub_message.parsed, graph)?;
            referenced_type_descriptions.push(sub_type_description.type_description);
        }
    }

    Some(TypeDescriptionMsg {
        type_description: TypeDescription {
            type_name: parsed.get_ros2_full_name(),
            fields,
        },
        referenced_type_descriptions,
    })
}

use serde::Serialize;
use serde_json::{ser::Formatter, Value};
use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

use crate::{parse::ParsedMessageFile, FieldInfo, MessageFile};

/// ROS2 requires a very specific JSON format for hashing:
/// - Single line
/// - Spaces after every control : and ,
/// - No other whitespace
pub fn to_ros2_json<T: Serialize>(v: T) -> String {
    let mut buf = Vec::new();
    {
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, Ros2Formatter::default());
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

// Directly taken from:
// https://github.com/ros2/rosidl/blob/rolling/rosidl_generator_type_description/rosidl_generator_type_description/__init__.py
lazy_static::lazy_static! {
    static ref FIELD_TYPE_NAME_TO_ID: HashMap<String, u8> = {
        let mut map = HashMap::new();
        map.insert("FIELD_TYPE_NOT_SET".to_string(), 0);

        // Nested type defined in other .msg/.idl files.
        map.insert("FIELD_TYPE_NESTED_TYPE".to_string(), 1);

        // Basic Types
        // Integer Types
        map.insert("FIELD_TYPE_INT8".to_string(), 2);
        map.insert("FIELD_TYPE_UINT8".to_string(), 3);
        map.insert("FIELD_TYPE_INT16".to_string(), 4);
        map.insert("FIELD_TYPE_UINT16".to_string(), 5);
        map.insert("FIELD_TYPE_INT32".to_string(), 6);
        map.insert("FIELD_TYPE_UINT32".to_string(), 7);
        map.insert("FIELD_TYPE_INT64".to_string(), 8);
        map.insert("FIELD_TYPE_UINT64".to_string(), 9);

        // Floating-Point Types
        map.insert("FIELD_TYPE_FLOAT".to_string(), 10);
        map.insert("FIELD_TYPE_DOUBLE".to_string(), 11);
        map.insert("FIELD_TYPE_LONG_DOUBLE".to_string(), 12);

        // Char and WChar Types
        map.insert("FIELD_TYPE_CHAR".to_string(), 13);
        map.insert("FIELD_TYPE_WCHAR".to_string(), 14);

        // Boolean Type
        map.insert("FIELD_TYPE_BOOLEAN".to_string(), 15);

        // Byte/Octet Type
        map.insert("FIELD_TYPE_BYTE".to_string(), 16);

        // String Types
        map.insert("FIELD_TYPE_STRING".to_string(), 17);
        map.insert("FIELD_TYPE_WSTRING".to_string(), 18);

        // Fixed String Types
        map.insert("FIELD_TYPE_FIXED_STRING".to_string(), 19);
        map.insert("FIELD_TYPE_FIXED_WSTRING".to_string(), 20);

        // Bounded String Types
        map.insert("FIELD_TYPE_BOUNDED_STRING".to_string(), 21);
        map.insert("FIELD_TYPE_BOUNDED_WSTRING".to_string(), 22);

        // Fixed Sized Array Types
        map.insert("FIELD_TYPE_NESTED_TYPE_ARRAY".to_string(), 49);
        map.insert("FIELD_TYPE_INT8_ARRAY".to_string(), 50);
        map.insert("FIELD_TYPE_UINT8_ARRAY".to_string(), 51);
        map.insert("FIELD_TYPE_INT16_ARRAY".to_string(), 52);
        map.insert("FIELD_TYPE_UINT16_ARRAY".to_string(), 53);
        map.insert("FIELD_TYPE_INT32_ARRAY".to_string(), 54);
        map.insert("FIELD_TYPE_UINT32_ARRAY".to_string(), 55);
        map.insert("FIELD_TYPE_INT64_ARRAY".to_string(), 56);
        map.insert("FIELD_TYPE_UINT64_ARRAY".to_string(), 57);
        map.insert("FIELD_TYPE_FLOAT_ARRAY".to_string(), 58);
        map.insert("FIELD_TYPE_DOUBLE_ARRAY".to_string(), 59);
        map.insert("FIELD_TYPE_LONG_DOUBLE_ARRAY".to_string(), 60);
        map.insert("FIELD_TYPE_CHAR_ARRAY".to_string(), 61);
        map.insert("FIELD_TYPE_WCHAR_ARRAY".to_string(), 62);
        map.insert("FIELD_TYPE_BOOLEAN_ARRAY".to_string(), 63);
        map.insert("FIELD_TYPE_BYTE_ARRAY".to_string(), 64);
        map.insert("FIELD_TYPE_STRING_ARRAY".to_string(), 65);
        map.insert("FIELD_TYPE_WSTRING_ARRAY".to_string(), 66);
        map.insert("FIELD_TYPE_FIXED_STRING_ARRAY".to_string(), 67);
        map.insert("FIELD_TYPE_FIXED_WSTRING_ARRAY".to_string(), 68);
        map.insert("FIELD_TYPE_BOUNDED_STRING_ARRAY".to_string(), 69);
        map.insert("FIELD_TYPE_BOUNDED_WSTRING_ARRAY".to_string(), 70);

        // Bounded Sequence Types
        map.insert("FIELD_TYPE_NESTED_TYPE_BOUNDED_SEQUENCE".to_string(), 97);
        map.insert("FIELD_TYPE_INT8_BOUNDED_SEQUENCE".to_string(), 98);
        map.insert("FIELD_TYPE_UINT8_BOUNDED_SEQUENCE".to_string(), 99);
        map.insert("FIELD_TYPE_INT16_BOUNDED_SEQUENCE".to_string(), 100);
        map.insert("FIELD_TYPE_UINT16_BOUNDED_SEQUENCE".to_string(), 101);
        map.insert("FIELD_TYPE_INT32_BOUNDED_SEQUENCE".to_string(), 102);
        map.insert("FIELD_TYPE_UINT32_BOUNDED_SEQUENCE".to_string(), 103);
        map.insert("FIELD_TYPE_INT64_BOUNDED_SEQUENCE".to_string(), 104);
        map.insert("FIELD_TYPE_UINT64_BOUNDED_SEQUENCE".to_string(), 105);
        map.insert("FIELD_TYPE_FLOAT_BOUNDED_SEQUENCE".to_string(), 106);
        map.insert("FIELD_TYPE_DOUBLE_BOUNDED_SEQUENCE".to_string(), 107);
        map.insert("FIELD_TYPE_LONG_DOUBLE_BOUNDED_SEQUENCE".to_string(), 108);
        map.insert("FIELD_TYPE_CHAR_BOUNDED_SEQUENCE".to_string(), 109);
        map.insert("FIELD_TYPE_WCHAR_BOUNDED_SEQUENCE".to_string(), 110);
        map.insert("FIELD_TYPE_BOOLEAN_BOUNDED_SEQUENCE".to_string(), 111);
        map.insert("FIELD_TYPE_BYTE_BOUNDED_SEQUENCE".to_string(), 112);
        map.insert("FIELD_TYPE_STRING_BOUNDED_SEQUENCE".to_string(), 113);
        map.insert("FIELD_TYPE_WSTRING_BOUNDED_SEQUENCE".to_string(), 114);
        map.insert("FIELD_TYPE_FIXED_STRING_BOUNDED_SEQUENCE".to_string(), 115);
        map.insert("FIELD_TYPE_FIXED_WSTRING_BOUNDED_SEQUENCE".to_string(), 116);
        map.insert("FIELD_TYPE_BOUNDED_STRING_BOUNDED_SEQUENCE".to_string(), 117);
        map.insert("FIELD_TYPE_BOUNDED_WSTRING_BOUNDED_SEQUENCE".to_string(), 118);

        // Unbounded Sequence Types
        map.insert("FIELD_TYPE_NESTED_TYPE_UNBOUNDED_SEQUENCE".to_string(), 145);
        map.insert("FIELD_TYPE_INT8_UNBOUNDED_SEQUENCE".to_string(), 146);
        map.insert("FIELD_TYPE_UINT8_UNBOUNDED_SEQUENCE".to_string(), 147);
        map.insert("FIELD_TYPE_INT16_UNBOUNDED_SEQUENCE".to_string(), 148);
        map.insert("FIELD_TYPE_UINT16_UNBOUNDED_SEQUENCE".to_string(), 149);
        map.insert("FIELD_TYPE_INT32_UNBOUNDED_SEQUENCE".to_string(), 150);
        map.insert("FIELD_TYPE_UINT32_UNBOUNDED_SEQUENCE".to_string(), 151);
        map.insert("FIELD_TYPE_INT64_UNBOUNDED_SEQUENCE".to_string(), 152);
        map.insert("FIELD_TYPE_UINT64_UNBOUNDED_SEQUENCE".to_string(), 153);
        map.insert("FIELD_TYPE_FLOAT_UNBOUNDED_SEQUENCE".to_string(), 154);
        map.insert("FIELD_TYPE_DOUBLE_UNBOUNDED_SEQUENCE".to_string(), 155);
        map.insert("FIELD_TYPE_LONG_DOUBLE_UNBOUNDED_SEQUENCE".to_string(), 156);
        map.insert("FIELD_TYPE_CHAR_UNBOUNDED_SEQUENCE".to_string(), 157);
        map.insert("FIELD_TYPE_WCHAR_UNBOUNDED_SEQUENCE".to_string(), 158);
        map.insert("FIELD_TYPE_BOOLEAN_UNBOUNDED_SEQUENCE".to_string(), 159);
        map.insert("FIELD_TYPE_BYTE_UNBOUNDED_SEQUENCE".to_string(), 160);
        map.insert("FIELD_TYPE_STRING_UNBOUNDED_SEQUENCE".to_string(), 161);
        map.insert("FIELD_TYPE_WSTRING_UNBOUNDED_SEQUENCE".to_string(), 162);
        map.insert("FIELD_TYPE_FIXED_STRING_UNBOUNDED_SEQUENCE".to_string(), 163);
        map.insert("FIELD_TYPE_FIXED_WSTRING_UNBOUNDED_SEQUENCE".to_string(), 164);
        map.insert("FIELD_TYPE_BOUNDED_STRING_UNBOUNDED_SEQUENCE".to_string(), 165);
        map.insert("FIELD_TYPE_BOUNDED_WSTRING_UNBOUNDED_SEQUENCE".to_string(), 166);

        map
    };

        static ref FIELD_VALUE_TYPE_NAMES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("nested_type", "FIELD_TYPE_NESTED_TYPE");
        map.insert("int8", "FIELD_TYPE_INT8");
        map.insert("uint8", "FIELD_TYPE_UINT8");
        map.insert("int16", "FIELD_TYPE_INT16");
        map.insert("uint16", "FIELD_TYPE_UINT16");
        map.insert("int32", "FIELD_TYPE_INT32");
        map.insert("uint32", "FIELD_TYPE_UINT32");
        map.insert("int64", "FIELD_TYPE_INT64");
        map.insert("uint64", "FIELD_TYPE_UINT64");
        map.insert("float", "FIELD_TYPE_FLOAT");
        map.insert("double", "FIELD_TYPE_DOUBLE");
        map.insert("long", "FIELD_TYPE_LONG_DOUBLE");
        map.insert("char", "FIELD_TYPE_CHAR");
        map.insert("wchar", "FIELD_TYPE_WCHAR");
        map.insert("boolean", "FIELD_TYPE_BOOLEAN");
        map.insert("octet", "FIELD_TYPE_BYTE");
        map.insert("string", "FIELD_TYPE_STRING");
        // TODO following likely don't work yet
        map.insert("wstring", "FIELD_TYPE_WSTRING");
        map.insert("bounded_string", "FIELD_TYPE_BOUNDED_STRING");
        map.insert("bounded_wstring", "FIELD_TYPE_BOUNDED_WSTRING");
        map
    };
}

/// Converts our internal representation of a field type to the numeric ID used in the ROS2 hash format
// TODO MAJOR: We don't support ROS2 capacity limits yet
fn get_field_type_id(field_type: &crate::FieldType) -> u8 {
    let core_name = match FIELD_VALUE_TYPE_NAMES.get(field_type.field_type.as_str()) {
        Some(name) => name,
        None => "FIELD_TYPE_NESTED_TYPE",
    };

    match field_type.array_info {
        Some(Some(_)) => {
            // Fixed length array
            let combined_name = core_name.to_string() + "_ARRAY";
            match FIELD_TYPE_NAME_TO_ID.get(&combined_name) {
                Some(id) => *id,
                None => {
                    panic!("Failed to find field type ID for {}", combined_name);
                }
            }
        }
        Some(None) => {
            // Unbounded array
            let combined_name = core_name.to_string() + "_UNBOUNDED_SEQUENCE";
            match FIELD_TYPE_NAME_TO_ID.get(&combined_name) {
                Some(id) => *id,
                None => {
                    panic!("Failed to find field type ID for {}", combined_name);
                }
            }
        }
        None => {
            // Not an array
            match FIELD_TYPE_NAME_TO_ID.get(core_name) {
                Some(id) => *id,
                None => {
                    panic!("Failed to find field type ID for {}", core_name);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::utils::{Package, RosVersion};

    /// Basic demonstration of the hashing process, and that our struct formatting matches ros2
    /// Reads in a pre-generated JSON file for std_msgs/String and confirms the hash matches
    /// File was generated by running `rosidl generate std_msgs ./String.msg -I .`
    /// Turns out these json files can just be found in the share/ folder of installed ros2 packages!
    /// This test is independent of our message file parsing logic
    #[test]
    fn ros2_hashing_against_message_files() {
        let test_data = [
            (
                include_str!("../assets/String.json"),
                "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
            ),
            (
                include_str!("../assets/MultiArrayLayout.json"),
                "RIHS01_4c66e6f78e740ac103a94cf63259f968e48c617e7699e829b63c21a5cb50dac6",
            ),
            (
                include_str!("../assets/MultiArrayDimension.json"),
                "RIHS01_5e773a60a4c7fc8a54985f307c7837aa2994252a126c301957a24e31282c9cbe",
            ),
            (
                include_str!("../assets/ColorRGBA.json"),
                "RIHS01_77a7a5b9ae477306097665106e0413ba74440245b1f3d0c6d6405fe5c7813fe8",
            ),
        ];
        for (test_file, expected_hash) in test_data {
            let parsed: super::TypeDescriptionFile = serde_json::from_str(test_file)
                .expect(format!("Failed to parse test file {test_file}").as_str());
            let hash = parsed.type_hashes[0].hash_string.clone();

            assert_eq!(hash, expected_hash,);

            let calculated_hash = super::calculate_hash(&parsed.type_description_msg);

            assert_eq!(calculated_hash, expected_hash);
        }
    }

    /// Double checking our conversion to the ROS2 integer type ids are correct
    #[test]
    fn spot_check_field_type_id() {
        let mut field = crate::FieldType {
            package_name: Some("std_msgs".to_string()),
            source_package: "std_msgs".to_string(),
            field_type: "string".to_string(),
            array_info: Some(Some(10)),
        };
        assert_eq!(super::get_field_type_id(&field), 65);

        field.array_info = Some(None);
        assert_eq!(super::get_field_type_id(&field), 161);

        field.field_type = "int32".to_string();
        field.array_info = Some(Some(10));
        assert_eq!(super::get_field_type_id(&field), 54);

        field.array_info = Some(None);
        assert_eq!(super::get_field_type_id(&field), 150);

        field.array_info = None;
        assert_eq!(super::get_field_type_id(&field), 6);
    }

    /// End-To-End test from parse -> hash
    /// Note: this test isn't super needed, and we test this more clearly in roslibrust_test package.
    /// Leaving this here, as it's slightly more specific than the end-to-end test in roslibrust_test
    #[test]
    fn full_hash_tests() {
        // We can at least do basic hashing for a String!
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let package = Package {
            name: "std_msgs".to_string(),
            path: root.join("../assets/ros2_common_interfaces/std_msgs"),
            version: Some(RosVersion::ROS2),
        };

        let (msg, _, _) = crate::parse_ros_files(vec![(
            package,
            root.join("../assets/ros2_common_interfaces/std_msgs/msg/String.msg"),
        )])
        .expect("Failed to parse test file");

        let hash = super::calculate_ros2_hash(&msg[0], &std::collections::BTreeMap::new())
            .expect("Failed to calculate hash");

        assert_eq!(
            hash,
            "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
        );
    }
}
