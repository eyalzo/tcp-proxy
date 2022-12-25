See also [NFC README](nfc/README.md)

# Overview

This app is a proxy that copies everything from one socket to another.
It accepts two input parameters: where to listen, and where to copy to on the other side.

# Simple run

Pushing bytes with `dd` and `nc`.

First, run the proxy:
```bash
cargo run -- --client 127.0.0.1:6000 --server 127.0.0.1:7000
```

Now the server:
```bash
nc -kvl 0.0.0.0 7000 > /dev/null
```

And finally the client, through the proxy:
```bash
dd if=/dev/zero bs=10240000 count=10 | nc -v 127.0.0.1 6000
```

The client can be stopped with ^c and run over and over again.
To compare the performance with and without the proxy, it is recommended to also connect the client directly to the server:
```bash
dd if=/dev/zero bs=10240000 count=10 | nc -v 127.0.0.1 7000
```

# Network setup

## Gateway

Preparing the internal NIC that is connected to HostA.

```bash
# List all NICS
sudo ip link show
# Set the input NIC (an example)
NICI=enp3s0f1
sudo ip addr flush dev $NICI 
sudo ip addr add 10.0.1.1/24 dev $NICI
# Verify that the input device was defined properly
sudo ip -4 addr show $NICI
```

## Define NAT

```bash
# Set packet forward that is needed for the rest of the traffic through the gateway
sudo sysctl -w net.ipv4.ip_forward=1

# Flush former rules (be careful here)
sudo iptables -F
sudo iptables -t nat -F

# Define NAT
export NICO=wlp4s0
export NICI=enp3s0f1
sudo iptables -t nat -A POSTROUTING -o $NICO -j MASQUERADE
sudo iptables -A FORWARD -i $NICI -j ACCEPT

# Accept traffic to this machine on port 6000 of the input NIC (the proxy)
sudo iptables -I INPUT 1 -i $NICI -p tcp --dport 6000 -j ACCEPT
# Additional rule to see counters of other packets to this machine
sudo iptables -A INPUT -i $NICI -j ACCEPT

# To verify and watch counters
sudo iptables -nvL
sudo iptables -t nat -nvL

# To watch counters live:
sudo watch -dc -n 1 iptables -nvL
```

### Tips

To reset counters:

```bash
iptables -Z
iptables -t nat -Z
```

To test NAT and iptables, run netcat on the gateway itself or on another machine behind it.
For example, to listen on all IPS forever:

```bash
nc -kvl 0.0.0.0 6000 > /dev/null
```

# Transparent proxy

To set a TPROXY rule in Ubuntu, you will need to use the iptables utility. 
TPROXY is a feature that allows you to redirect traffic to a local proxy without the need for NAT.

Note: The TPROXY is allowed in PREROUTING only, meaning that only forwarded traffic can be captured and not INPUT nor OUTPUT (as DIVERT does).
To simulate forwarding, one can use VirtualBox with "Bridged Adapter" (the default NAT does not suffice).

Here's an example of how to set a TPROXY rule that redirects all HTTP traffic from a specific source IP address to a local HTTP proxy listening on port 8080:

```bash
iptables -t mangle -A PREROUTING -i $NICI -p tcp --dport 443 -j TPROXY --tproxy-mark 0x1/0x1 --on-port 6000
```

This rule will mark all HTTPS traffic from the source IP with the mark 0x1/0x1, and redirect it to the local proxy listening on port 6000.

You can then use the ip rule command to specify that traffic marked with 0x1/0x1 should be redirected to the proxy:

```bash
ip rule add fwmark 1 lookup 100
ip route add local 0.0.0.0/0 dev lo table 100
```

This will redirect all traffic marked with 0x1/0x1 to the local loopback interface, which is where the proxy is listening.

Keep in mind that these rules are not persistent across reboots. 
To make them persistent, you will need to save them to a script and configure the system to run the script at startup.

In the proxy's code, the setsockopt() operation requires Linux with CAP_NET_ADMIN privileges.

# Examples

## TPROXY

```bash
$ sudo cargo run -- --client 0.0.0.0:6000 --server 185.125.190.48:80
```
```text
Bind successfully on 0.0.0.0:6000
Succeeded to setsockopt(IpTransparent).
Waiting for a client to connect 0.0.0.0:6000 ...
Copied client -> server: 87 bytes
   Accepted from 10.0.1.2:47702
   Client connected from 10.0.1.2:47702, trying to reach 91.189.91.48:80 (TPROXY)
Trying to connect server on 91.189.91.48:80 ...
   Server 91.189.91.48:80 connected
Copied server -> client: 147 bytes
```