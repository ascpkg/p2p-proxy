use std::sync::Arc;

use anyhow::Result;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

pub mod data;
pub mod http;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {

    // Create a MediaEngine object to configure the supported codec
    let mut media_engine = MediaEngine::default();

    // Register default codecs
    media_engine.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut media_engine)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let rtc_config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let rtc_peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Create the data channel
    let rtc_data_channel = rtc_peer_connection.create_data_channel("foo", None).await?;

    let rtc_data_channel_clone = Arc::clone(&rtc_data_channel);
    rtc_data_channel.on_open(Box::new(move || {
        println!("Data channel 'foo' open");
        Box::pin(async {})
    }));

    rtc_data_channel.on_close(Box::new(|| {
        println!("Data channel closed");
        Box::pin(async {})
    }));

    rtc_data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
        println!("Message from DataChannel: '{}'", msg_str);
        Box::pin(async {})
    }));

    // Set the handler for ICE connection state
    rtc_peer_connection.on_ice_connection_state_change(Box::new(
        |connection_state: webrtc::ice_transport::ice_connection_state::RTCIceConnectionState| {
            println!("ICE Connection State has changed: {}", connection_state);
            Box::pin(async {})
        },
    ));

    // Create an offer
    let rtc_session_offer = rtc_peer_connection.create_offer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = rtc_peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    rtc_peer_connection
        .set_local_description(rtc_session_offer)
        .await?;

    // Block until ICE Gathering is complete
    let _ = gather_complete.recv().await;

    // Output the offer in base64
    if let Some(local_desc) = rtc_peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        let b64 = utils::b64_encode(&json_str);
        println!("Local Session Description (copy this to another):");
        println!("{}", b64);

        
   
    }

    println!("Please paste the remote Session Description from another:");
    let rtc_remote_session_b64 = utils::must_read_stdin()?;
    let rtc_remote_session_json_data = utils::b64_decode(rtc_remote_session_b64.as_str())?;
    let rtc_remote_session_answer =
        serde_json::from_str::<RTCSessionDescription>(&rtc_remote_session_json_data)?;

    // Set the remote SessionDescription
    rtc_peer_connection
        .set_remote_description(rtc_remote_session_answer)
        .await?;

    println!("Connection established. Type a message to send (or ctrl-c to quit):");
    loop {
        tokio::select! {
            _ = done_rx.recv() => {
                println!("received done signal!");
                break;
            }
            _ = tokio::signal::ctrl_c() => {
                println!();
                break;
            }
            line = tokio::task::spawn_blocking(utils::must_read_stdin) => {
                match line {
                    Ok(Ok(line)) => {
                        if !line.is_empty() {
                            if let Err(err) = rtc_data_channel_clone.send_text(line).await {
                                println!("Error sending message: {}", err);
                            }
                        }
                    }
                    Ok(Err(err)) => {
                        println!("Error reading input: {}", err);
                        break;
                    }
                    Err(err) => {
                        println!("Error in spawn_blocking: {}", err);
                        break;
                    }
                }
            }
        };
    }

    rtc_peer_connection.close().await?;
    Ok(())
}
