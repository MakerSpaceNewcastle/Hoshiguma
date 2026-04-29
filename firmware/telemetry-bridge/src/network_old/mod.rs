#[embassy_executor::task]
async fn connection_task(stack: Stack<'static>) -> ! {
    let mut rng = RoscRng;

    'connection: loop {
        // Get configuration via DHCP
        {
            info!("Waiting for DHCP");
            while !stack.is_config_up() {
                Timer::after_millis(100).await;
            }
            info!("DHCP is now up");

            let config = stack.config_v4().unwrap();
            LINK_STATE.lock(|v| {
                let mut state = v.borrow_mut();
                state.last_changed.replace(Instant::now());
                state.dhcp4_config.replace(config);
            });
        }

        let mut rx_buffer = [0; 8192];
        let mut tls_read_buffer = [0; 16640];
        let mut tls_write_buffer = [0; 16640];

        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);
        let tls_config = TlsConfig::new(
            rng.next_u64(),
            &mut tls_read_buffer,
            &mut tls_write_buffer,
            TlsVerify::None,
        );

        let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

        let mut data_point_line_rx = TELEMETRY_TX.subscriber().unwrap();

        let mut telegraf_buffer = TelegrafBuffer::default();

        loop {
            match select(data_point_line_rx.next_message(), Timer::after_millis(800)).await {
                Either::First(WaitResult::Message(metric)) => {
                    // Add the metric to the buffer
                    match telegraf_buffer.push(metric) {
                        Ok(_) => {
                            DATA_POINTS_ACCEPTED.add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            warn!("Failed to push metric to buffer");
                            DATA_POINTS_DISCARDED.add(1, Ordering::Relaxed);
                        }
                    }

                    // If the buffer is nearing capacity, then send now
                    if telegraf_buffer.send_required() {
                        info!("Tx reason: buffer nearly full");
                        telegraf_buffer.tx(&mut http_client, &mut rx_buffer).await;
                    }
                }
                Either::First(WaitResult::Lagged(n)) => {
                    warn!("Subscriber lagged, lost {} messages", n);
                }
                Either::Second(_) => {
                    info!("Tx reason: periodic purge");
                    telegraf_buffer.tx(&mut http_client, &mut rx_buffer).await;

                    if !stack.is_config_up() {
                        warn!("Network down");

                        LINK_STATE.lock(|v| {
                            let mut state = v.borrow_mut();
                            state.last_changed.replace(Instant::now());
                            state.dhcp4_config.take();
                        });

                        continue 'connection;
                    }
                }
            }
        }
    }
}
