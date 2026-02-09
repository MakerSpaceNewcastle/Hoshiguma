#![no_std]

pub mod air_assist;
pub mod coolant_rate;
pub mod cooling;
pub mod extraction_airflow;
pub mod fume_extraction;
pub mod hmi_status_screen;
pub mod interlock;
pub mod machine_power;
pub mod status_light;
pub mod temperatures;

pub struct StateMachineRunner<InputChannel, OutputChannel, State: Default> {
    input_channel: InputChannel,
    output_channel: OutputChannel,
    state: State,
}

#[allow(async_fn_in_trait)]
pub trait StateMachineRun {
    async fn run(&mut self) -> !;
}

#[macro_export]
macro_rules! state_machine {
    ($input_msg: ty, $output_msg: ty, $state: ty, $channel_size: expr) => {
        pub type InputChannel = embassy_sync::channel::Channel<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            $input_msg,
            $channel_size,
        >;
        pub type OutputChannel = embassy_sync::channel::Channel<
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            $output_msg,
            $channel_size,
        >;

        pub type OutputChannelSender<'a> = embassy_sync::channel::Sender<
            'a,
            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
            $output_msg,
            $channel_size,
        >;

        pub type StateMachineRunner<'a> = $crate::StateMachineRunner<
            embassy_sync::channel::Receiver<
                'a,
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $input_msg,
                $channel_size,
            >,
            OutputChannelSender<'a>,
            $state,
        >;

        pub struct StateMachineCommunicator<'a> {
            input_channel: embassy_sync::channel::Sender<
                'a,
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $input_msg,
                $channel_size,
            >,
            output_channel: embassy_sync::channel::Receiver<
                'a,
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                $output_msg,
                $channel_size,
            >,
        }

        impl<'a> StateMachineCommunicator<'a> {
            pub async fn send_input(&self, message: $input_msg) {
                self.input_channel.send(message).await;
            }

            pub fn receive_channel_len(&self) -> usize {
                self.output_channel.len()
            }

            pub async fn receive_output(&mut self) -> $output_msg {
                self.output_channel.receive().await
            }
        }

        pub fn new<'a>(
            input_channel: &'a InputChannel,
            output_channel: &'a OutputChannel,
        ) -> (StateMachineRunner<'a>, StateMachineCommunicator<'a>) {
            let runner = StateMachineRunner {
                input_channel: input_channel.receiver(),
                output_channel: output_channel.sender(),
                state: Default::default(),
            };

            let communicator = StateMachineCommunicator {
                input_channel: input_channel.sender(),
                output_channel: output_channel.receiver(),
            };

            (runner, communicator)
        }
    };
}
