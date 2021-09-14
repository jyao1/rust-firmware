use core::slice;

use fw_vsock::vsock::{VsockAddr, VsockStream};
use alloc::collections::btree_map::BTreeMap;

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SOCKET_COUNT: Mutex<i32> = Mutex::new(2);
    static ref SOCKET_COLLECT: Mutex<BTreeMap<i32, Socket>> = Mutex::new(BTreeMap::new());
}

#[no_mangle]
pub extern "C" fn socket_client () {
    let mut s = VsockStream::new();
    s.connect(&VsockAddr::new(2, 1234)).expect("error");
    let _nsend = s.send(b"hello", 0).unwrap();
}

#[no_mangle]
//to do: 1. object format (clang --target=x86_64-unknown-windows) 2. calling convention, substitute extern "C"
pub extern "C" fn socket_server () {
    let mut server_socket = VsockStream::new();
    let listen_addrss = VsockAddr::new(33, 1234);
    server_socket.bind(&listen_addrss).expect("bind error\n");
    log::info!("listen on: {}\n", listen_addrss);
    server_socket.listen(1).expect("listen error\n");
    // can accept
    let (mut client_socket, client_addr) = server_socket.accept().expect("accept failed\n");
    log::info!("client accept: {}\n", client_addr);

    loop {
        let mut recv_buf = [0u8; 1024];
        let recvn = client_socket
            .recv(&mut recv_buf[..], 0)
            .expect("recv error\n");
        if recvn == 0 {
            break;
        }
        log::info!("recv: {:?}\n", &recv_buf[..recvn]);
    }
}

pub struct Socket {
    vsock_stream: VsockStream,
    remote: Option<VsockStream>
}

#[repr(C)]
pub struct SockAddr {
    svm_family: u16,
    svm_reserved: u16,
    svm_port: u32,
    svm_cid: u32,
    sa_data:[u8;4],
}

#[no_mangle]
pub extern "C" fn socket (_domain: i32, _socket_type: i32, _protocol: i32) -> i32 {
    let socket_stream = VsockStream::new();
    *SOCKET_COUNT.lock() += 1;
    let sockfd = *SOCKET_COUNT.lock();
    let socket = Socket {vsock_stream: socket_stream, remote: None};
    SOCKET_COLLECT.lock().insert(sockfd, socket);
    sockfd
}

#[no_mangle]
pub extern "C" fn bind (sockfd: i32, socket_addr: *mut SockAddr, _addrlen: u32) -> i32 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            unsafe {
                log::info! ("bind sockaddr: cid: {}, port: {}\n", (*socket_addr).svm_cid, (*socket_addr).svm_port);
                let listen_addrss = VsockAddr::new((*socket_addr).svm_cid, (*socket_addr).svm_port);
                match socket.vsock_stream.bind(&listen_addrss) {
                    Ok(()) => 0,
                    Err(_e) => {
                        log::info!("bind error\n");
                        -1
                    }
                }
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn listen (sockfd: i32, backlog: i32) -> i32 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            match socket.vsock_stream.listen(backlog as u32) {
                Ok(()) => 0,
                Err(_e) => {
                    log::info!("listen error\n");
                    -1
                }
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn accept (sockfd: i32, socket_addr: *mut SockAddr, addrlen: *mut u32) -> i32 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            match socket.vsock_stream.accept() {
                Ok((remote, vsockaddr)) => {
                    socket.remote = Some(remote);
                    unsafe {
                        (*socket_addr).svm_cid = vsockaddr.cid();
                        (*socket_addr).svm_port = vsockaddr.port();
                        *addrlen = 14;
                    };
                    0
                },
                Err(_e) => {
                    log::info!("accept error\n");
                    -1
                }
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn connect (sockfd: i32, socket_addr: *mut SockAddr, _addrlen: u32) -> i32 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            // if socket.connected.
            unsafe {
                match socket.vsock_stream.connect(&VsockAddr::new((*socket_addr).svm_cid, (*socket_addr).svm_port)) {
                    Ok(()) => 0,
                    Err(_e) => {
                        log::info!("connect error\n");
                        -1
                    }
                }
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn recv (sockfd: i32, buf: *mut u8,  len: u64, flags: i32) -> i64 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            match &mut socket.remote {
                Some(remote) => {
                    unsafe {
                        let recv_buf = slice::from_raw_parts_mut(buf, len as usize);
                        let recvn = remote
                            .recv(&mut recv_buf[..], flags as u32)
                            .expect("recv error\n");
                        if recvn == 0 {
                            return 0;
                        }
                        log::info!("recv: {:?}\n", &recv_buf[..recvn]);
                        recvn as i64
                    }
                }
                None => {
                    log::info!("client not found\n");
                    return -1
                }
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn shutdown (sockfd: i32, _how: i32) -> i32 {
    let mut map = SOCKET_COLLECT.lock();
    match map.remove(&sockfd) {
        Some(socket) => {
            let _sock_stream = socket;
            0
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn send (sockfd: i32, buf: *mut u8,  len: u64, flags: i32) -> i64 {
    match SOCKET_COLLECT.lock().get_mut(&sockfd) {
        Some(socket) => {
            unsafe {
                let send_buf = slice::from_raw_parts(buf, len as usize);
                log::info!("send len: {:}\n", len);
                let sendn = socket.vsock_stream
                    .send(&send_buf[..], flags as u32)
                    .expect("send error\n");
                if sendn == 0 {
                    return 0;
                }
                log::info!("send: {:?}\n", &send_buf[..len as usize]);
                sendn as i64
            }
        },
        None => {
            log::info!("sockfd: {} not found\n", sockfd);
            -1
        },
    }
}
