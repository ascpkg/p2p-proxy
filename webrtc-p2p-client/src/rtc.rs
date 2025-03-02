use std::{sync::Arc, time::Duration};

use anyhow::Result;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::api::{
    interceptor_registry::register_default_interceptors, setting_engine::SettingEngine,
};
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use crate::{
    data::{Agent, Configurations, Sdp},
    http,
};

const MESSAGE_SIZE: usize = 1500;

pub async fn start_rtc_client(
    config: &Configurations,
    agent: Agent,
    is_udp: bool,
    local_port: u16,
    remote_port: u16,
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

    // Create a new RTCPeerConnection
    let rtc_peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Create the data channel
    let label = format!(
        "{}-{}-{}-{}-{}",
        agent.name,
        agent.os,
        agent.uuid,
        if is_udp { "udp" } else { "tcp" },
        remote_port
    );
    let label_1 = label.clone();
    let label_2 = label_1.clone();
    let rtc_data_channel = rtc_peer_connection
        .create_data_channel(&label, None)
        .await?;

    let _rtc_data_channel_clone = Arc::clone(&rtc_data_channel);
    rtc_data_channel.on_open(Box::new(move || {
        tracing::info!("rtc_data_channel.on_open(label: {label_1})");
        Box::pin(async {})
    }));

    rtc_data_channel.on_close(Box::new(move || {
        tracing::info!("rtc_data_channel.on_close(label: {label_2})");
        Box::pin(async {})
    }));

    rtc_data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        // forward
        Box::pin(async {})
    }));

    // Set the handler for ICE connection state
    rtc_peer_connection.on_ice_connection_state_change(Box::new(
        |connection_state: webrtc::ice_transport::ice_connection_state::RTCIceConnectionState| {
            tracing::info!(
                "rtc_peer_connection.on_ice_connection_state_change(state: {connection_state})"
            );
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

    // Output the offer
    if let Some(local_desc) = rtc_peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        http::publish_client_sdp(
            config,
            &agent.uuid,
            &Sdp {
                sdp: json_str,
                is_udp,
                port: remote_port,
            },
        );
    }

    let mut agent_sdp = Sdp::default();
    while agent_sdp.sdp.is_empty() {
        let sdps = http::query_agent_sdp(config, &agent.uuid);
        for s in &sdps {
            if s.is_udp == is_udp && s.port == remote_port {
                agent_sdp = s.to_owned();
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let rtc_remote_session_answer = serde_json::from_str::<RTCSessionDescription>(&agent_sdp.sdp)?;

    // Set the remote SessionDescription
    rtc_peer_connection
        .set_remote_description(rtc_remote_session_answer)
        .await?;

    let rtc_data_ch_clone = Arc::clone(&rtc_data_channel);
    let rtc_data_ch_clone_id = rtc_data_ch_clone.id();
    let rtc_data_ch_clone_label = rtc_data_ch_clone.label().to_string();
    rtc_data_channel.on_open(Box::new(move || {
        tracing::info!(
            "rtc_data_ch.on_open(id: {rtc_data_ch_clone_id}), label: {rtc_data_ch_clone_label})"
        );

        Box::pin(async move {
            // tokio start tcp server
            let tcp_server = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", local_port))
                .await
                .unwrap();

            // accept tcp connection
            let (tcp_stream, _) = tcp_server.accept().await.unwrap();
            tracing::info!("tcp connection accepted");

            let raw_data_ch = match rtc_data_ch_clone.detach().await {
                Ok(raw) => raw,
                Err(err) => {
                    tracing::error!("rtc_data_ch_clone.detach() error, err: {err}");
                    return;
                }
            };

            let (mut r, mut w) = tokio::io::split(tcp_stream);
            let raw_data_ch_r = Arc::clone(&raw_data_ch);
            tokio::spawn(async move {
                let _ = read_loop(&mut r, raw_data_ch).await;
            });

            tokio::spawn(async move {
                let _ = write_loop(&mut w, raw_data_ch_r).await;
            });
        })
    }));

    tokio::select! {
        _ = done_rx.recv() => {
            tracing::info!("received done signal");
        }
    };

    rtc_peer_connection.close().await?;

    Ok(())
}

async fn read_loop(
    socket_r: &mut tokio::io::ReadHalf<tokio::net::TcpStream>,
    data_ch_w: Arc<webrtc::data::data_channel::DataChannel>,
) -> Result<()> {
    let mut buffer = vec![0u8; MESSAGE_SIZE];
    let mut result = Result::<usize>::Ok(0);
    while result.is_ok() {
        let n = match socket_r.read(&mut buffer).await {
            Ok(n) => {
                result = data_ch_w
                    .write(&Bytes::from(buffer[..n].to_vec()))
                    .await
                    .map_err(Into::into);
                n
            }
            Err(err) => {
                tracing::error!("data_ch_w.write error, err: {err}");
                return Ok(());
            }
        };
    }

    Ok(())
}

async fn write_loop(
    socket_w: &mut tokio::io::WriteHalf<tokio::net::TcpStream>,
    data_ch_r: Arc<webrtc::data::data_channel::DataChannel>,
) -> Result<()> {
    let mut buffer = vec![0u8; MESSAGE_SIZE];
    loop {
        let n = match data_ch_r.read(&mut buffer).await {
            Ok(n) => {
                socket_w.write(&buffer[..n]).await?;
                n
            }
            Err(err) => {
                tracing::error!("socket_w.write error, err: {err}");
                return Ok(());
            }
        };
    }
}
