// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use fw_vsock::vsock::{VsockAddr, VsockStream};

#[allow(dead_code)]
pub fn test_server() {
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
