use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::data::{Configurations, Sdp};
use crate::http;

const MESSAGE_SIZE: usize = 1500;

pub async fn start_rtc_agent(
    config: &Configurations,
    client_sdp: Sdp,
    passive_done_rx: &mut tokio::sync::mpsc::Receiver<()>,
) -> Result<()> {
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

    // Create a SettingEngine and enable Detach
    let mut s = SettingEngine::default();
    s.detach_data_channels();

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .with_setting_engine(s)
        .build();

    // Prepare the configuration
    let rtc_config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: config.stun_server_urls.clone(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let (active_done_tx, mut active_done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Create a new RTCPeerConnection
    let rtc_peer_conn = Arc::new(api.new_peer_connection(rtc_config).await?);

    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    rtc_peer_conn.on_peer_connection_state_change(Box::new(
        move |state: RTCPeerConnectionState| {
            if state != RTCPeerConnectionState::Failed {
                tracing::info!("rtc_peer_conn.on_peer_connection_state_change({state})");
            } else {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                tracing::error!(
                    "rtc_peer_conn.on_peer_connection_state_change(RTCPeerConnectionState::Failed)"
                );
                let _ = active_done_tx.try_send(());
            }

            Box::pin(async {})
        },
    ));

    // Register data channel creation handling
    rtc_peer_conn.on_data_channel(Box::new(move |rtc_data_ch: Arc<RTCDataChannel>| {
        let rtc_data_ch_id = rtc_data_ch.id();
        let rtc_data_ch_label = rtc_data_ch.label().to_owned();
        tracing::info!("on_data_channel(id: {rtc_data_ch_id}), label: {rtc_data_ch_label})");

        // Register channel opening handling
        Box::pin(async move {
            let rtc_data_ch_clone = Arc::clone(&rtc_data_ch);
            let rtc_data_ch_clone_id = rtc_data_ch_id;
            let rtc_data_ch_clone_label = rtc_data_ch_label.clone();
            let label_1 = rtc_data_ch_clone_label.clone();
            let label_2 = rtc_data_ch_clone_label.clone();
            rtc_data_ch.on_close(Box::new(move || {
                tracing::info!(
                    "rtc_data_ch.on_close(id: {rtc_data_ch_clone_id}), label: {label_1})"
                );
                Box::pin(async {})
            }));

            rtc_data_ch.on_open(Box::new(move || {
                tracing::info!(
                    "rtc_data_ch.on_open(id: {rtc_data_ch_clone_id}), label: {label_2})"
                );

                Box::pin(async move {
                    let raw_data_ch = match rtc_data_ch_clone.detach().await {
                        Ok(raw) => raw,
                        Err(err) => {
                            tracing::error!("rtc_data_ch_clone.detach() error, err: {err}");
                            return;
                        }
                    };

                    let socket = tokio::net::TcpStream::connect(SocketAddr::new(
                        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                        client_sdp.port,
                    ))
                    .await
                    .unwrap();
                    let (mut socket_r, mut socket_w) = tokio::io::split(socket);

                    let raw_data_ch_r = Arc::clone(&raw_data_ch);
                    tokio::spawn(async move {
                        let _ = read_loop(raw_data_ch_r, &mut socket_w).await;
                    });

                    tokio::spawn(async move {
                        let _ = write_loop(raw_data_ch, &mut socket_r).await;
                    });
                })
            }));

            // Register message handling
            rtc_data_ch.on_message(Box::new(move |_msg: DataChannelMessage| {
                // forward
                Box::pin(async move {})
            }));
        })
    }));

    // Set the remote SessionDescription
    let client_session_desc = serde_json::from_str::<RTCSessionDescription>(&client_sdp.sdp)?;
    rtc_peer_conn
        .set_remote_description(client_session_desc)
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

    if let Some(session_desc) = rtc_peer_conn.local_description().await {
        let agent_session_desc = serde_json::to_string(&session_desc)?;
        let agent_sdp = Sdp {
            sdp: agent_session_desc,
            ..client_sdp
        };
        http::publish_agent_sdp(config, &config.uuid, &agent_sdp);
    } else {
        tracing::warn!("rtc_peer_conn.local_description error");
    }

    tokio::select! {
        _ = active_done_rx.recv() => {
            tracing::warn!("received active done signal");
        }
        _ = passive_done_rx.recv() => {
            tracing::warn!("received passive done signal");
        }
    };

    rtc_peer_conn.close().await?;

    Ok(())
}

async fn read_loop(
    data_ch_r: Arc<webrtc::data::data_channel::DataChannel>,
    socket_w: &mut tokio::io::WriteHalf<tokio::net::TcpStream>,
) -> Result<()> {
    let mut buffer = vec![0u8; MESSAGE_SIZE];
    loop {
        let n = match data_ch_r.read(&mut buffer).await {
            Ok(n) => {
                socket_w.write(&buffer[..n]).await?;
                n
            }
            Err(err) => {
                tracing::error!("data_ch.read error, err: {err}");
                return Ok(());
            }
        };
    }
}

async fn write_loop(
    data_ch_w: Arc<webrtc::data::data_channel::DataChannel>,
    socket_r: &mut tokio::io::ReadHalf<tokio::net::TcpStream>,
) -> Result<()> {
    let mut result = Result::<usize>::Ok(0);
    while result.is_ok() {
        let mut buf = vec![0u8; MESSAGE_SIZE];
        socket_r.read(&mut buf).await?;
        result = data_ch_w.write(&Bytes::from(buf)).await.map_err(Into::into);
    }

    Ok(())
}
