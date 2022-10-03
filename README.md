# Overview

This app is a proxy that copies everything from one socket to another.
It accepts two input parameters: where to listen, and where to copy to on the other side.

# Network setup

## Gateway

Preparing the internal NIC that is connected to HostA.

```bash
sudo su
ip link show
NICI=enp3s0f1
ip addr flush dev $NICI 
ip addr add 10.0.1.1/24 dev $NICI
ip -4 addr show
```

## Define NAT

```bash
sudo su
# Set packet forward that is needed for the rest of the traffic through the gateway
sudo sysctl -w net.ipv4.ip_forward=1
# Define NAT
export NICO=wlp4s0
export NICI=enp3s0f1
iptables -t nat -A POSTROUTING -o $NICO -j MASQUERADE
iptables -A FORWARD -i $NICI -j ACCEPT

# To verify and watch counters
iptables -nvL
iptables -t nat -nvL
```

### Tips

To flush iptables before start

```bash
iptables -F
iptables -t nat -F
```
To reset counters

```bash
iptables -Z
iptables -t nat -Z
```

To watch counters live

```bash
watch -dc -n 1 iptables -nvL
```