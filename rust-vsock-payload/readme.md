## Purpose

POC rust vsock over virtio vsock

## How to Build

Note: Assume you can build `rust-firmware` with `rust-uefi-payload` and run it in qemu.

```bash
cargo xbuild --target x86_64-unknown-uefi --release
export RESET_VECTOR_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/ResetVector.bin
export RUST_IPL_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust_ipl.efi
export RUST_FIRMWARE_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/final_vsock.bin
```

To link a static C library in Rust, set the folder and name of static lib file to the environment variable:

```bash
export RUST_LINK_C_LIB_DIR=$BASE_DIR/rust-vsock-payload/
export RUST_LINK_C_LIB_NAME=main
```

To build as default PE format OBJ and MS x64 calling convention using clang and cargo mbuild:

```bash
## build static C library
clang rust-vsock-payload/vsock_c_lib/main.c -c --target=x86_64-unknown-windows -o rust-vsock-payload/vsock_c_lib/main.o
llvm-ar r rust-vsock-payload/main.lib rust-vsock-payload/vsock_c_lib/main.o
## build rust-vsock-payload
cargo mbuild -p rust-vsock-payload --release
export RUST_PAYLOAD_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust-vsock-payload.efi
```

To build as ELF format and Linux GCC x86-64 calling convention using GCC and cargo elfbuild:

```bash
## build static C library
gcc rust-vsock-payload/vsock_c_lib/main.c -c -fno-stack-protector -o rust-vsock-payload/vsock_c_lib/main.o
ar r rust-vsock-payload/libmain.a rust-vsock-payload/vsock_c_lib/main.o
## build rust-vsock-payload
cargo elfbuild -p rust-vsock-payload --release
export RUST_PAYLOAD_BIN=$BASE_DIR/target/target/release/rust-vsock-payload
```

Then:

```bash
cargo run -p rust-firmware-tool -- $RESET_VECTOR_BIN $RUST_IPL_BIN $RUST_PAYLOAD_BIN $RUST_FIRMWARE_BIN
```


## How to Run

Qemu version is *QEMU emulator version 5.2.0*

Linux kernel version is *5.12.16*

```
## enable vsock-vhost module
sudo insmod vhost-vsock
# set vhost-vsock permission
sudo setfacl -m u:${USER}:rw /dev/vhost-vsock

## RUST_FIRMWARE_BIN is final_vsock.bin
## VMN is GUEST_CID
export VMN=33
/home/luxy/local/bin/qemu-system-x86_64 \
 -m 2G -machine q35 \
 -drive if=pflash,format=raw,unit=0,file=$RUST_FIRMWARE_BIN \
 -device vhost-vsock-pci,id=vhost-vsock-pci1,guest-cid=${VMN} \
 -nic none -vnc 0.0.0.0:1 -serial mon:stdio \
 -debugcon file:debug.log -global isa-debugcon.iobase=0x402
```

## Rust Socket API Design

| Socket Impl | My-Rust                                                              | C                                                                                                     | Rust-Vsock                                          | Python                                               |
| ----------- | -------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------------- |
| new_socket  | pub fn new() -> Self                                                 | int socket(family, type, protocol)                                                                    | bind(addr: &SockAddr) -> Result                     | socket.socket(familiy, type, proto, fileno)          |
| bind        | pub fn bind(&mut self, addr: &VsockAddr) -> Result                   | int bind(int sockfd, const struct sockaddr *addr,socklen_t addrlen)                                   | bind(addr: &SockAddr) -> Result                     | socket.bind(address)                                 |
| listen      | pub fn listen(&mut self, backlog: u32) -> Result                     | int listen(int sockfd, int backlog)                                                                   | bind(addr: &SockAddr) -> Result<VsockListener>      | socket.listen([backlog])                             |
| accept      | pub fn accept(&self) -> Result<(VsockStream, VsockAddr)>             | int new_socket= accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);                        | fn accept(&self) -> Result<(VsockStream, SockAddr)> | socket.accept()                                      |
| connect     | pub fn connect(&mut self, addr: &VsockAddr) -> Result                | int connect(int sockfd, const struct sockaddr *addr,socklen_t addrlen);                               | pub fn connect(addr: &SockAddr) -> Result<Self>     | socket.connect(address)                              |
| shutdown    | pub fn shutdown(&mut self) -> Result                                 | int shutdown (int fd, int how)                                                                        | shutdown(&self, how: Shutdown) -> Result<()>        | socket.shutdown(how)                                 |
| send        | pub fn send(&mut self, buf: &[u8], _flags: u32) -> Result<usize>     | ssize_t send(int sockfd, const void *buf, size_t len, int flags);                                     | impl Write for VsockStream {                        | socket.send(bytes[, flags])                          |
| recv        | pub fn recv(&mut self, buf: &mut [u8], _flags: u32) -> Result<usize> | ssize_t recv(int sockfd, void *buf, size_t len, int flags);                                           | impl Read for VsockStream                           | socket.recv(bufsize[, flags])                        |
| getsockopt  | NA                                                                   | int getsockopt(int sockfd, int level, int optname,void *restrict optval, socklen_t *restrict optlen); | NA                                                  | socket.getsockopt(level, optname[, buflen])          |
| setsockopt  | NA                                                                   | int setsockopt(int sockfd, int level, int optname,void *restrict optval, socklen_t *restrict optlen); | NA                                                  | socket.setsockopt(level, optname, None, optlen: int) |
| settimeout  | NA                                                                   | NA                                                                                                    | NA                                                  | s.settimeout(timeout)                                |
