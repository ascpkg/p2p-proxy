use std::future::Future;
use std::net::{Ipv4Addr, SocketAddr};
use std::{io, net::IpAddr};
use tokio::net::{TcpStream, UdpSocket};

pub trait ProxySocket: Send {
    fn teardown<'a>(&'a mut self) -> Box<dyn Future<Output = io::Result<()>> + Send + 'a>;
    fn send<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Box<dyn Future<Output = io::Result<usize>> + Send + 'a>;
    fn read<'a>(
        &'a mut self,
        data: &'a mut [u8],
    ) -> Box<dyn Future<Output = io::Result<(usize, SocketAddr)>> + Send + 'a>;
}

pub struct ProxyTcpSocket {
    addr: SocketAddr,
    tcp_stream: TcpStream,
}

pub struct ProxyUdpSocket {
    addr: SocketAddr,
    udp_socket: UdpSocket,
}

pub struct X {
    proxy_socket: Box<dyn ProxySocket>,
}

impl X {
    pub fn new(proxy_socket: Box<dyn ProxySocket>) -> Self {
        Self { proxy_socket }
    }
}

impl ProxyTcpSocket {
    pub async fn setup(port: u16) -> io::Result<ProxyTcpSocket> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let tcp_stream = TcpStream::connect(addr).await?;
        Ok(Self { addr, tcp_stream })
    }
}

impl ProxySocket for ProxyTcpSocket {
    fn teardown<'a>(&'a mut self) -> Box<dyn Future<Output = io::Result<()>> + Send + 'a> {
        Box::new(async move { Ok(()) })
    }

    fn send<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Box<dyn Future<Output = io::Result<usize>> + Send + 'a> {
        Box::new(async move { self.tcp_stream.try_write(data) })
    }

    fn read<'a>(
        &'a mut self,
        data: &'a mut [u8],
    ) -> Box<dyn Future<Output = io::Result<(usize, SocketAddr)>> + Send + 'a> {
        Box::new(async move { Ok((self.tcp_stream.try_read(data)?, self.addr)) })
    }
}

impl ProxyUdpSocket {
    pub async fn setup(port: u16) -> io::Result<ProxyUdpSocket> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
        let udp_socket = UdpSocket::bind(addr).await?;
        Ok(Self { addr, udp_socket })
    }
}

impl ProxySocket for ProxyUdpSocket {
    fn teardown<'a>(&'a mut self) -> Box<dyn Future<Output = io::Result<()>> + Send + 'a> {
        Box::new(async move { Ok(()) })
    }

    fn send<'a>(
        &'a mut self,
        data: &'a [u8],
    ) -> Box<dyn Future<Output = io::Result<usize>> + Send + 'a> {
        Box::new(async move { self.udp_socket.send_to(data, self.addr).await })
    }

    fn read<'a>(
        &'a mut self,
        data: &'a mut [u8],
    ) -> Box<dyn Future<Output = io::Result<(usize, SocketAddr)>> + Send + 'a> {
        Box::new(async move { self.udp_socket.recv_from(data).await })
    }
}
