

## VirtIO VSock

###  Enable Vsock module

```insmod vhost_vsock```
```insmod vsock```

```
# set vsock permission
sudo setfacl -m u:${USER}:rw /dev/vhost-vsock
```

result:
```
luxy@luxy-clr-linux ~/run/vsock $ ls -l /dev/vhost-vsock
crw-rw----+ 1 root root 10, 241 Jul 30 15:33 /dev/vhost-vsock
luxy@luxy-clr-linux ~/run/vsock $ lsmod | grep vsock
vsockmon               16384  1
vhost_vsock            24576  0
vmw_vsock_virtio_transport_common    36864  1 vhost_vsock
vsock                  45056  3 vmw_vsock_virtio_transport_common,vhost_vsock,vsockmon
luxy@luxy-clr-linux ~/run/vsock $
```


### Test with Nmap (test vsock)

**Version** Use 7.91

NMap
https://nmap.org/download.html

NMap source
https://nmap.org/dist/

Install
https://nmap.org/book/inst-source.html

```
tar xf nmap-7.91.tar.bz2
pushd nmap-7.91/
./configure --prefix /home/luxy/local
make install
popd
export PATH=/home/luxy/local/bin:$PATH

libtool --finish /home/luxy/local/lib
```


```
# listening in host and guest connecting, in this case, they can communicate each other.

# listening in guest and host connecting, in this case, they communicate failed.
# assume guest CID is 33
nc -v --vsock -l 1234
nc -v --vsock 33 1234
```

### Test with Socat

SOCAT
http://www.dest-unreach.org/socat/
**version** 1.7.4.1 on Jul  6 2021 07:45:25
Download: http://www.dest-unreach.org/socat/download/socat-1.7.4.1.tar.gz

```
wget http://www.dest-unreach.org/socat/download/socat-1.7.4.1.tar.gz
e
tar xf socat-1.7.4.1.tar.gz
cd socat-1.7.4.1
./configure
make
make install
```

Socat Example
http://www.dest-unreach.org/socat/doc/socat.html#EXAMPLES

Vsock Test
```
socat - VSOCK-LISTEN:1234
socat - VSOCK-CONNECT:33:1234
```


### Qemu param

CLEARLINUX ref: https://docs.01.org/clearlinux/latest/

```
qemu-system-x86_64 \
-enable-kvm \
-bios $OVMF_FD \
-smp sockets=1,cpus=1,cores=1 -cpu host \
-m 1024 \
-vga none -nographic \
-drive file="$CLEAR_LINUX",if=virtio,aio=threads,format=raw \
-device vhost-vsock-pci,id=vhost-vsock-pci1,guest-cid=${VMN} \
-debugcon file:debug.log -global isa-debugcon.iobase=0x402
 ```


### Vsock spec

https://github.com/stefanha/virtio

vsock qemu driver
https://github.com/stefanha/qemu/tree/vsock

vsock linux driver
https://github.com/stefanha/linux/tree/vsock


### Appendix A: Reference
kvm conference
https://www.youtube.com/watch?v=_bYSQ68JPwE

use qemu with vsock
https://gist.github.com/mcastelino/9a57d00ccf245b98de2129f0efe39857

VSOCK: VM ↔host socket with minimal configuration - DevConfCZ 2020
https://www.youtube.com/watch?v=R5DQWdPUQSY
https://static.sched.com/hosted_files/devconfcz2020a/b1/DevConf.CZ_2020_vsock_v1.1.pdf

Clear Linux
https://docs.01.org/clearlinux/latest/get-started/virtual-machine-install/kvm.html
https://cdn.download.clearlinux.org/image/

VirtioVsock:
https://static.sched.com/hosted_files/kvmforum2019/50/KVMForum_2019_virtio_vsock_Andra_Paraschiv_Stefano_Garzarella_v1.3.pdf

VSOCK linux upstream
https://lwn.net/Articles/520000/
