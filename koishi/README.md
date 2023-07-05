# Koishi

This coprocessor is responsible for fast safety and machine control signalling only.

All inputs are via isolated digital pins. Serial input should not be used.

Outputs are via relays and status reporting via UART.

## Tasks

- Providing Ruida generic system protection signal
- Providing Ruida cooling protection signal
- Providing laser PSU protection signal
- Start/stop air assist compressor
- Start/stop fume extraction fan
- Illuminate machine status lights
