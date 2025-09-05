//! Eventually will need to be moved to roslibrust_codegen

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
    // Value is not used in hashing
    #[serde(skip)]
    default_value: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ros_json_formatter;


    #[test]
    fn basic_ros2_hashing() {
        let test_file = include_str!("../assets/String.json");

        let parsed: TypeDescriptionFile = serde_json::from_str(test_file).unwrap();

        let hash = parsed.type_hashes[0].hash_string.clone();

        assert_eq!(
            hash,
            "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
        );

        let type_description_string =
            ros_json_formatter::to_ros2_json(&parsed.type_description_msg);

        println!("Type description string: {type_description_string}");

        // Calculate the sha256 hash of the type description string
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(type_description_string);
        let result = hasher.finalize();
        let hash = format!("RIHS01_{result:x}");

        assert_eq!(
            hash,
            "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
        );
    }
}
