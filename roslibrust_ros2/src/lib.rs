#[cfg(test)]
mod tests {
    // Basic test showing serialization and deserialization of a simple string message with Serde and cdr
    #[test]
    fn test_deserialize_hello_world() {
        // Test data was taken from a manual zenoh subscription against `ros2 topic pub /chatter std_msgs/msg/String 'data: Hello World'`
        let zenoh_bytes: [u8; _] = [
            0x00, 0x01, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20,
            0x57, 0x6f, 0x72, 0x6c, 0x64, 0x00,
        ];

        // Example struct we expect to convert to/from std_msgs::String in ros
        #[derive(serde::Deserialize, serde::Serialize)]
        struct StdMsgString {
            data: String,
        }

        // Serialize
        let serialized = cdr::serialize::<_, _, cdr::CdrLe>(
            &StdMsgString {
                data: "Hello World".to_string(),
            },
            cdr::Infinite,
        )
        .unwrap();
        assert_eq!(serialized, zenoh_bytes);

        // Deserialize
        let deserialized = cdr::deserialize::<StdMsgString>(&zenoh_bytes).unwrap();
        assert_eq!(deserialized.data, "Hello World");
    }

}
