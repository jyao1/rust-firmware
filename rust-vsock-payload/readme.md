## Purpose

POC rust vsock over virtio vsock

## How to Build

Note: Assume you can build `rust-firmware` with `rust-uefi-payload` and run it in qemu.

```bash
export RESET_VECTOR_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/ResetVector.bin
export RUST_IPL_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust_ipl.efi
export RUST_PAYLOAD_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/rust-vsock-payload.efi
export RUST_FIRMWARE_BIN=$BASE_DIR/target/x86_64-unknown-uefi/release/final_vsock.bin

cargo mbuild -p rust-vsock-payload --release
cargo run -p rust-firmware-tool -- $RESET_VECTOR_BIN $RUST_IPL_BIN $RUST_PAYLOAD_BIN $RUST_FIRMWARE_BIN

```

## How to Run

Qemu version is *QEMU emulator version 5.2.0*

Linux kernel version is *5.12.16*

```
## RUST_FIRMWARE_BIN is final_vsock.bin
## VMN is GUEST_CID
export VMN=33
/home/luxy/local/bin/qemu-system-x86_64 \
 -m 4G -machine q35 \
 -drive if=pflash,format=raw,unit=0,file=$RUST_FIRMWARE_BIN \
 -device vhost-vsock-pci,id=vhost-vsock-pci1,guest-cid=${VMN} \
 -nic none -vnc 0.0.0.0:1 -serial mon:stdio \
 -debugcon file:debug.log -global isa-debugcon.iobase=0x402
```

## Rust Socket API Design


| Socket Impl | C                                                                                                     | My-Rust                                                                  | Rust-Vsock                                          |
| ----------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------- |
| new_socket  | int socket(family, type, protocol)                                                                    | pub fn new() -> Self                                                     | bind(addr: &SockAddr) -> Result                     |
| bind        | int bind(int sockfd, const struct sockaddr *addr,socklen_t addrlen)                                   | pub  fn  bind(&mut  self, addr: &VsockAddr) ->  Result                   | bind(addr: &SockAddr) -> Result                     |
| listen      | int listen(int sockfd, int backlog)                                                                   | pub  fn  listen(&mut  self, backlog: u32) ->  Result                     | bind(addr: &SockAddr) -> Result<VsockListener>      |
| accept      | int new_socket= accept(int sockfd, struct sockaddr *addr, socklen_t *addrlen);                        | pub  fn  accept(&self) ->  Result<(VsockStream, VsockAddr)>              | fn accept(&self) -> Result<(VsockStream, SockAddr)> |
| connect     | int connect(int sockfd, const struct sockaddr *addr,socklen_t addrlen);                               | pub  fn  connect(&mut  self, addr: &VsockAddr) ->  Result                | pub fn connect(addr: &SockAddr) -> Result<Self>     |
| shutdown    | int shutdown (int fd, int how)                                                                        | pub  fn  shutdown(&mut  self) ->  Result                                 | shutdown(&self, how: Shutdown) -> Result<()>        |
| send        | ssize_t send(int sockfd, const void *buf, size_t len, int flags);                                     | pub  fn  send(&mut  self, buf: &[u8], _flags: u32) ->  Result<usize>     | impl Write for VsockStream {                        |
| recv        | ssize_t recv(int sockfd, void *buf, size_t len, int flags);                                           | pub  fn  recv(&mut  self, buf: &mut [u8], _flags: u32) ->  Result<usize> | impl Read for VsockStream                           |
| getsockopt  | int getsockopt(int sockfd, int level, int optname,void *restrict optval, socklen_t *restrict optlen); | NA                                                                       | NA                                                  |
| setsockopt  | int setsockopt(int sockfd, int level, int optname,void *restrict optval, socklen_t *restrict optlen); | NA                                                                       | NA                                                  |

