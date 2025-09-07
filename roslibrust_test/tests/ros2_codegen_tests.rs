use roslibrust_test::ros2::*;

#[test]
fn test_defaults() {
    let x: test_msgs::Defaults = Default::default();
    assert_eq!(x.x, 42);
    assert_eq!(x.y, -2000);
    assert_eq!(x.full_name, "John Doe");
    assert_eq!(x.samples, vec![-200, -100, 0, 100, 200]);
    assert_eq!(x.s_vec, vec!["hello", "world"]);
    assert_eq!(x.f_samples, vec![-200.0, -1.0, 0.0]);
}

#[test]
fn fixed_sized_arrays() {
    // Prove the default works, compiler failure here is the test
    let x: sensor_msgs::NavSatFix = Default::default();
    // Prove the types here match, compiler failure here is the test
    let _y: [f64; 9] = x.position_covariance;
    // NavSatFix is the easy case because it is <32 items long

    // Harder case that deals with arrays >32
    let x: geometry_msgs::TwistWithCovariance = Default::default();
    let _y: [f64; 36] = x.covariance;
}

/// Confirms that ros2 hashes are being populated with correct expected values
#[test]
fn ros2_hash_checks() {
    use roslibrust::{RosMessageType, RosServiceType};
    use roslibrust_test::ros2::*;

    // Basic string message
    assert_eq!(
        std_msgs::String::ROS2_HASH,
        "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
    );

    // MultiArrayLayout has a dependency on MultiArrayDimension
    // Proving we can handle chained dependencies
    assert_eq!(
        std_msgs::MultiArrayLayout::ROS2_HASH,
        "RIHS01_4c66e6f78e740ac103a94cf63259f968e48c617e7699e829b63c21a5cb50dac6"
    );

    // Empty is an edge case so worth checking
    // Note: this is a weird one... JSON:
    // Need to implement a case to match this:
    /*
    "type_description_msg": {
      "type_description": {
        "type_name": "std_msgs/msg/Empty",
        "fields": [
          {
            "name": "structure_needs_at_least_one_member",
            "type": {
              "type_id": 3,
              "capacity": 0,
              "string_capacity": 0,
              "nested_type_name": ""
            },
            "default_value": ""
          }
        ]
      },
      "referenced_type_descriptions": []
    }
    */
    // assert_eq!(
    //     std_msgs::Empty::ROS2_HASH,
    //     "RIHS01_20b625256f32d5dbc0d04fee44f43c41e51c70d3502f84b4a08e7a9c26a96312"
    // );

    // TODO not implemented yet
    // Test a service
    // assert_eq!(
    //     std_srvs::SetBool::ROS2_HASH,
    //     "RIHS01_abe9e4bb6b41b40e6789712c00ec8871923e089af3f667a79992a428cff2da0a"
    // );

    // TODO a special case will be required here
    // Test empty service
    // assert_eq!(
    //     std_srvs::Empty::ROS2_HASH,
    //     "RIHS01_5888399dedec5ccc85ea6451949fd2c9f97bfdf963f9a588821639fcd31b5d19"
    // )
}
