use std::sync::Arc;

use anyhow::Result;
use tokio::time::Duration;
use webrtc::api::APIBuilder;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::math_rand_alpha;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
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
    let rtc_peer_conn = Arc::new(api.new_peer_connection(rtc_config).await?);

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    rtc_peer_conn.on_peer_connection_state_change(Box::new(
        move |state: RTCPeerConnectionState| {
            println!("Peer Connection State has changed: {state}");

            if state == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                println!("Peer Connection has gone to failed exiting");
                let _ = done_tx.try_send(());
            }

            Box::pin(async {})
        },
    ));

    // Register data channel creation handling
    rtc_peer_conn
        .on_data_channel(Box::new(move |rtc_data_ch: Arc<RTCDataChannel>| {
            let rtc_data_ch_label = rtc_data_ch.label().to_owned();
            let rtc_data_ch_id = rtc_data_ch.id();
            println!("New DataChannel {rtc_data_ch_label} {rtc_data_ch_id}");

            // Register channel opening handling
            Box::pin(async move {
                let rtc_data_ch2 = Arc::clone(&rtc_data_ch);
                let rtc_data_ch_label2 = rtc_data_ch_label.clone();
                let rtc_data_ch_id2 = rtc_data_ch_id;
                rtc_data_ch.on_close(Box::new(move || {
                    println!("Data channel closed");
                    Box::pin(async {})
                }));

                rtc_data_ch.on_open(Box::new(move || {
                    println!("Data channel '{rtc_data_ch_label2}'-'{rtc_data_ch_id2}' open. Random messages will now be sent to any connected DataChannels every 5 seconds");

                    Box::pin(async move {
                        let mut result = Result::<usize>::Ok(0);
                        while result.is_ok() {
                            let timeout = tokio::time::sleep(Duration::from_secs(5));
                            tokio::pin!(timeout);

                            tokio::select! {
                                _ = timeout.as_mut() =>{
                                    let message = math_rand_alpha(15);
                                    println!("Sending '{message}'");
                                    result = rtc_data_ch2.send_text(message).await.map_err(Into::into);
                                }
                            };
                        }
                    })
                }));

                // Register text message handling
                rtc_data_ch.on_message(Box::new(move |msg: DataChannelMessage| {
                    let msg_str = String::from_utf8(msg.data.to_vec()).unwrap();
                    println!("Message from DataChannel '{rtc_data_ch_label}': '{msg_str}'");
                    Box::pin(async {})
                }));
            })
        }));

    // Wait for the offer to be pasted
    let rtc_remote_desc_b64 = utils::must_read_stdin()?;
    let rtc_remote_desc_json_data = utils::b64_decode(rtc_remote_desc_b64.as_str())?;
    let rtc_remote_session_offer =
        serde_json::from_str::<RTCSessionDescription>(&rtc_remote_desc_json_data)?;

    // Set the remote SessionDescription
    rtc_peer_conn
        .set_remote_description(rtc_remote_session_offer)
        .await?;

    // Create an answer
    let rtc_session_answer = rtc_peer_conn.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = rtc_peer_conn.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    rtc_peer_conn
        .set_local_description(rtc_session_answer)
        .await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    if let Some(local_desc) = rtc_peer_conn.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        let b64 = utils::b64_encode(&json_str);
        println!("{b64}");
    } else {
        println!("generate local_description failed!");
    }

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    rtc_peer_conn.close().await?;

    Ok(())
}
