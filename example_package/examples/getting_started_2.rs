//! This file is part the getting_started.md documentation in the book.
//! It is included here so it can better be automatically tested
//! 
// ANCHOR: preamble
// Bring generated messages into scope:
include!(concat!(env!("OUT_DIR"), "/messages.rs"));

// Bring in traits we need from roslibrust
use roslibrust::traits::{Publish, Subscribe, Ros};

use std::sync::Arc;
use tokio::sync::Mutex;

// Writing a simple publisher behavior using roslibrust's generic traits
async fn pub_counter(ros: impl Ros, state: Arc<Mutex<u32>>) {
    // This will nicely control the rat our code runs at
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    // Create a publisher on our topic
    let publisher = ros
        .advertise::<std_msgs::UInt32>("/example_counter")
        .await
        .expect("Could not create publisher!");

    loop {
        // Wait for next tick of our interval timer
        interval.tick().await;

        // Lock our state and read the current value
        let cur_val = *state.lock().await;

        // Publish the current value
        publisher
            .publish(&std_msgs::UInt32 { data: cur_val })
            .await
            .expect("Failed to publish message!");

        // Increment our state
        *state.lock().await += 1;
    }
}
// ANCHOR_END: preamble
// ANCHOR: subscriber
async fn sub_counter(ros: impl Ros, state: Arc<Mutex<u32>>) {
    // Create a subscriber on our topic
    let mut subscriber = ros
        .subscribe::<std_msgs::UInt32>("/example_counter")
        .await
        .expect("Could not create subscriber!");

    loop {
        // Wait for next message
        let msg = subscriber.next().await.expect("Failed to get message!");

        // Print the message
        println!("Got message: {}", msg.data);
        
        // Decrement our state
        *state.lock().await -= 1;
    }
}
// ANCHOR_END: subscriber

// ANCHOR: main
// This macro sets up a basic tokio runtime for us and lets our main function be `async`
#[tokio::main]
async fn main() {
    // Initialize a logger to help with debugging
    env_logger::init();

    // Create a rosbridge client we can use
    let ros = roslibrust::rosbridge::ClientHandle::new("ws://localhost:9090")
        .await
        .expect("Failed to connect to rosbridge!");

    // Create a shared state we can use to track our counter
    let shared_state = Arc::new(Mutex::new(0));

    // Spawn a new tokio task to run our publisher:
    tokio::spawn(pub_counter(ros.clone(), shared_state.clone()));

    // Spawn a new tokio task to run our subscriber:
    tokio::spawn(sub_counter(ros, shared_state.clone()));

    // Wait for ctrl_c
    tokio::signal::ctrl_c().await.unwrap();
}
// ANCHOR_END: main 

// ANCHOR: test
// cfg(test) here means that this code is only compile when invoking `cargo test` and doesn't get included in normal builds
#[cfg(test)]
mod test {
    // Bring pub_counter and sub_counter into scope
    use super::*;

    // Tokio will automatically set up an individual async runtime for this test
    #[tokio::test]
    async fn test_pub_sub_counter() {
        // To let us see how long the test takes to run record the current time
        let tick = std::time::SystemTime::now();
        // MAGIC HERE:
        tokio::time::pause();

        // Create a mock ros instance we can use
        // This instance of ROS is unique to this test and won't interfere with any other tests running parallel
        let ros = roslibrust::mock::MockRos::new();

        // Creating separate states so we can inspect individually how they change
        let publisher_state = Arc::new(Mutex::new(0));
        let subscriber_state = Arc::new(Mutex::new(10));

        // Spawn a new tokio task to run our publisher:
        tokio::spawn(pub_counter(ros.clone(), publisher_state.clone()));
        // Spawn a new tokio task to run our subscriber:
        tokio::spawn(sub_counter(ros, subscriber_state.clone()));

        // The publisher and subscriber run for a bit in the background
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let published_count = *publisher_state.lock().await;
        let subscribed_count = *subscriber_state.lock().await;
        // Check the exact number of messages our publisher and subscriber got
        assert_eq!(published_count, 10, "Published count should be 10, but was {published_count}");
        assert_eq!(subscribed_count, 0, "Subscribed count should be 0, but was {subscribed_count}");
        // Purely for demonstration purposes, show how long this test takes to run
        let tock = std::time::SystemTime::now();
        println!("Test took in realtime {:?}", tock.duration_since(tick));
    }
}
// ANCHOR_END: test