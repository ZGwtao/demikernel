# Catlions on Microkit

`sddf` owns the sDDF client queues, timer calls, and C bridge without depending on Demikernel. `std_sel4` supplies the target-only compatibility types and facade. `src/catlions` implements the LibOS network adapter and Microkit event loop using those layers. The echo and probe applications live in `examples/catlions-pd`; `examples/catlions` in the outer repository contains the Microkit system description and build parameters.

The `catlions-libos` feature builds Demikernel's original `SharedInetStack`, UDP and TCP socket paths, and `LibOS` API for the `aarch64-sel4` target using `core` and `alloc`. The PD in `dep/demikernel/examples/catlions-pd` runs a UDP echo server on `10.0.2.15:9999` and a concurrent TCP echo server on `10.0.2.15:9998`; its receive and send operations use `LibOS::pop`, `LibOS::pushto`, and `LibOS::wait`. Ethernet frames pass through the SDDF network queues. QEMU host forwarding has returned UDP and TCP payloads; TCP was checked with two waves of eight simultaneous connections, twenty echo messages per connection, and reconnects. An eight-client, 25-second run echoed 36,907 messages without a PD fault; UDP still worked afterward. A separate run completed 150 connect/echo/close cycles without a panic or PD fault.

## Target adaptations

- Platform code is split between `src/catlions`, `src/std_sel4`, `sddf`, and `std_sel4`. The remaining edits in shared Demikernel code are feature gates and call sites for the catlions LibOS, `core`/`alloc` imports, buffer selection, timer deadline access, and retaining a listener after an accept timeout. The `std` compatibility facade, MAC address, and PAL definitions live in `src/std_sel4`; libc and socket constants live in `std_sel4`. The single-PD timer storage, static random seed, target configuration, initial task scheduling policy, and notification loop live in `src/catlions`.
- `dep/demikernel/sddf` wraps the SDDF client queues and reads and arms the SDDF timer through its Microkit protected call channel. Each notification advances Demikernel's runtime clock before polling the scheduler. The PD arms the SDDF timer for the earliest pending coroutine timeout; it does not use a periodic 10 ms tick.
- `dep/demikernel/src/catlions/runtime.rs` initializes the SDDF network and timer and owns the Microkit notification loop. Applications implement `Application::poll`; `EventLoop` advances Demikernel's clock, drains RX notifications, and arms the next coroutine deadline. The echo server and the separate UDP probe both use this entry point.
- `dep/demikernel/std_sel4/libc` supplies the errno and socket address definitions needed by Demikernel on seL4. `dep/demikernel/std_sel4/socket_types` supplies the `socket2` domain, type, and protocol values used by the socket API. These do not implement host OS sockets.
- The Makefile supplies the guest IP, application ports, and host connection endpoint at build time. Defaults match QEMU user networking: guest IP `10.0.2.15`, host gateway `10.0.2.2`, UDP echo port `9999`, and TCP echo port `9998`. The MAC comes from SDDF. The PD does not parse YAML. Host builds retain the normal configuration path.
- The target-only `std_sel4` facade maps required collection, time, and synchronization interfaces to `core`, `alloc`, and `hashbrown`. The target timer storage assumes one thread in the PD.
- `dep/demikernel/sddf/buffer` can borrow a mapped SDDF RX buffer until the final `DemiBuffer` view is dropped, when its descriptor returns to the RX free queue. TX-backed buffers are allocated from the SDDF TX free queue with protocol headroom. Unique packets transfer their descriptor directly to the TX active queue; shared packets, including TCP data retained for retransmission, use the copy path. TX descriptors acquired but not submitted are held in a PD-private recycle stack, preserving the SDDF free queue's single-producer ownership. The SDDF copy component still copies driver RX data into the client RX region.

## Build and run

Set `MICROKIT_SDK` to the Microkit SDK directory and build with:

```sh
make -C examples/catlions all
```

The resulting image is `examples/catlions/build/catlions.img`. Run it on QEMU virt AArch64 with a virtio network device and user networking:

```sh
qemu-system-aarch64 -machine virt,virtualization=on -cpu cortex-a53 -m size=2G -nographic \
  -netdev user,id=netdev0,hostfwd=udp:127.0.0.1:19999-:9999,hostfwd=tcp:127.0.0.1:19998-:9998 \
  -device virtio-net-device,netdev=netdev0,bus=virtio-mmio-bus.0 \
  -global virtio-mmio.force-legacy=false \
  -device loader,file=examples/catlions/build/catlions.img,addr=0x70000000,cpu-num=0
```

Send a UDP packet to `127.0.0.1:19999` or connect by TCP to `127.0.0.1:19998`; both echo payloads. The TCP demo keeps an accept operation pending and uses `wait_any` to advance active connections.

The deployment values live in `examples/catlions/Makefile`: `CATLIONS_GUEST_IP`, `CATLIONS_UDP_PORT`, `CATLIONS_TCP_PORT`, `CATLIONS_HOST_IP`, `CATLIONS_HOST_TCP_PORT`, `CATLIONS_HOST_REFUSED_PORT`, and `CATLIONS_PROBE_PORT`. The `QEMU_GUEST_PORT` and `QEMU_TCP_GUEST_PORT` defaults feed the echo ports. For example, `make -C examples/catlions all QEMU_GUEST_PORT=10005 QEMU_TCP_GUEST_PORT=10006` builds an image that was verified through QEMU forwards to those ports.

For the active-open example, run a TCP echo server on the host at port `10000`, then build with `make -C examples/catlions all CATLIONS_CONNECT_TEST=1`. The PD connects to QEMU's host gateway `10.0.2.2:10000`, checks the echoed bytes, closes, and reconnects once. Build with `CATLIONS_CONNECT_TEST=1 CATLIONS_CONNECT_REFUSED=1` to exercise a connection refusal on host port `10001`; the PD reports the failed connect and closes the socket. Rebuild without these options for the normal server image.

To build the separate UDP probe application, use `make -C examples/catlions all CATLIONS_PROBE_APP=1` and forward `udp:127.0.0.1:20001-:10001` in QEMU. A datagram sent to host port `20001` returns with ASCII letters uppercased. This application is implemented in `dep/demikernel/examples/catlions-pd/src/probe.rs` and uses the same catlions platform event loop as echo.

## Scope

This verifies the original Demikernel UDP and TCP socket paths and SDDF timer/network integration. Eight concurrent TCP connections, close and reconnect, a 25-second sustained run, 150 sequential connection cycles, and UDP coexistence have been exercised. After the RX borrowing change, four concurrent TCP clients sent ten messages each, followed by 8,000 TCP echoes across eight clients and a UDP echo; no panic or PD fault occurred. With deadline-driven scheduling, four concurrent TCP clients and UDP echo passed again. A three-second idle run produced zero timer notifications before a UDP packet arrived. With one outgoing TCP data segment deliberately dropped, the echo arrived after about one second via retransmission. With SYN-ACK packets dropped until handshake timeout, `accept` returned a timeout while retaining the listener; after loss was disabled, a new TCP client and UDP both echoed successfully. The TX-backed UDP echo path passed 5, 256, and 1200-byte payloads. With deferred catlions scheduling and direct TX submission, eight TCP clients completed 8,000 echoes, UDP remained available, and a dropped TCP data response was retransmitted after about one second. Larger connection counts, other LibOS operations, and longer runs have not been exercised on Microkit. The PD uses a 16 MiB heap because each TCP connection allocates substantial receive and send state. The single-thread timer storage needs generalization for multiple PD threads.

## Fault injection

For QEMU timer and loss tests, build with `make -C examples/catlions all CATLIONS_DROP_TCP=1` to drop one outgoing TCP SYN-ACK, `CATLIONS_DROP_TCP=2` to drop one outgoing TCP data segment, or `CATLIONS_DROP_TCP=3` to drop outgoing SYN-ACK packets until the first accept timeout, then allow traffic again for recovery testing. The default is `0` (no loss). Rebuild with `CATLIONS_DROP_TCP=0` before normal use.
