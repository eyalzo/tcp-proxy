# Overview

This folder holds a runable version of NFC testing tool.
It runs in Linux only (Ubuntu, for example).

# Run without the proxy

You need to activate two copies - a server and a client.
Note that both are pretty fragile and intended for testing only.
The server must run before the client's `Connect` command, and it terminates after the client terminates.

## Preparing file to send

```bash
rm /tmp/test_*
head -c 10M /dev/urandom > /tmp/test_random_10m
```

## Server

In a separate tab, to remain open while running the client on another tab.

```bash
cd ~/git/tcp-proxy/nfc/
./nfcTest -p 14465
```

As a response, you should see tons of error messages, ending with `Server bound to port 14465`.

## Client

The client can copy a local file to the remote NFC server by using the PUT command with full path.
The destination folder needs to be ready before copy, and the file must not exist.
Here, too, you should see tons of error messages, ending with `Starting Interactive Shell...`.

```bash
cd ~/git/tcp-proxy/nfc/
./nfcTest
```

The following commands should be typed in the interactive shell.

```text
SetCnxType direct
Connect 127.0.0.1 14465
Put /tmp/test_random_10m /tmp/test_random_10m_out_01
Quit
```

# Run with the proxy

The difference here is that the client needs to connect the proxy instead of the NFC server.

The proxy:

```bash
cargo run -- --client 127.0.0.1:6000 --server 127.0.0.1:14465
```

The client's commands (only the `Connect` port was changed here):

```text
SetCnxType direct
Connect 127.0.0.1 6000
Put /tmp/test_random_10m /tmp/test_random_10m_out_01
Quit
```

The proxy's output would be similar to this:

```text
     Running `target/debug/tcp-proxy --client '127.0.0.1:6000' --server '127.0.0.1:14465'`
Bind successfully on 127.0.0.1:6000
Waiting for a client to connect 127.0.0.1:6000 ...
   Client connected from 127.0.0.1:6000
Trying to connect server on 127.0.0.1:14465 ...
   Server 127.0.0.1:14465 connected
Copied client -> server: 10498811 bytes
c2s done
Waiting for a client to connect 127.0.0.1:6000 ...
Copied server -> client: 1584 bytes
```

# Hijacking to the proxy

We start by doing everything locally, so the hijacking is performed in OUTPUT because localhost does not go through PREROUTING.

```bash
# Cleanup previous NAT rules (be careful)
sudo iptables -t nat -F
# Add rule to capture traffic intended for the imaginary port 7000 where nothing actually runs
sudo iptables -t nat -A OUTPUT -o lo -d 127.0.0.1 -p tcp --dport 7000 -j DNAT --to-destination 127.0.0.1:6000
# To watch counters live:
sudo watch -dc -n 1 iptables -t nat -nvL
```

This time, connect the NFC client to port 7000:

```text
SetCnxType direct
Connect 127.0.0.1 7000
Put /tmp/test_random_10m /tmp/test_random_10m_out_01
Quit
```

Wireshark can capture this session, but it will look a bit weired. 
See below the connection phase, where port 6000 is seen on one way, but port 7000 answers instead.

```text
No. Time         Info
1   0.000000000  39360 → 6000 [SYN] Seq=0 Win=65495 Len=0 MSS=65495 SACK_PERM=1 TSval=186089247 TSecr=0 WS=128
2   0.000014465  7000 → 39360 [SYN, ACK] Seq=0 Ack=1 Win=65483 Len=0 MSS=65495 SACK_PERM=1 TSval=186089247 TSecr=186089247 WS=128
3   0.000026275  39360 → 6000 [ACK] Seq=1 Ack=1 Win=65536 Len=0 TSval=186089247 TSecr=186089247
```