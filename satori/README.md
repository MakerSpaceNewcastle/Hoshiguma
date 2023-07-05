# Satori

This coprocessor is responsible for slow, low priority or unreliable machine safety signalling and networked reporting.

## Tasks

- Read cooling system temperature sensors
- Read cooling system flow sensor
- Receive machine status from Koishi via UART
- Report complete machine status and sensor values via MQTT
