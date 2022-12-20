# Overview

This app is a proxy that copies everything from one socket to another.
It accepts two input parameters: where to listen, and where to copy to on the other side.

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
