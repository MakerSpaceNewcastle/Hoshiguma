use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind, ObservationEvent},
    rpc::{Request, Response},
    types::MonitorKind,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Row, Table, TableState},
};
use std::{fmt::Debug, io, time::Duration};
use strum::{EnumIter, IntoEnumIterator};
use teeny_rpc::{client::Client, transport::serialport::SerialTransport};

#[derive(Debug, Clone, Eq, PartialEq, EnumIter)]
enum Parameter {
    PeripheralControllerGitRevision,
    PeripheralControllerBootReason,

    CoolerGitRevision,
    CoolerBootReason,

    MonitorMachinePowerOff,
    MonitorChassisIntrusion,
    MonitorCoolerCommunicationFault,
    MonitorCoolantHeaderTankLevelSensorFault,
    MonitorCoolantHeaderTankEmpty,
    MonitorCoolantHeaderTankOverfilled,
    MonitorHeatExchangerFluidLow,
    MonitorCoolantFlowInsufficient,
    MonitorTemperatureSensorFaultA,
    MonitorTemperatureSensorFaultB,
    MonitorCoolantFlowOvertemperatureA,
    MonitorCoolantFlowOvertemperatureB,
    MonitorHeatExchangerOvertemperature,

    MachineOperationLockout,

    CoolingEnabled,
    CoolingDemand,

    ObservationMachinePower,
    ObservationMachineRun,
    ObservationChassisIntrusion,
    ObservationFumeExtractionMode,
    ObservationAirAssistDemand,
    ObservationCoolantFlow,
    ObservationHeatExchangerFluidLevel,
    ObservationCoolantHeaderTankLevel,
    ObservationTemperatureAOnboard,
    ObservationTemperatureAElectronicsBay,
    ObservationTemperatureALaserChamber,
    ObservationTemperatureACoolantFlow,
    ObservationTemperatureACoolantReturn,
    ObservationTemperatureBOnboard,
    ObservationTemperatureBCoolantFlow,
    ObservationTemperatureBCoolantMid,
    ObservationTemperatureBCoolantReturn,
    ObservationTemperatureBHeatExchangeFluid,
    ObservationTemperatureBHeatExchangerLoop,

    ControlAirAssistPump,
    ControlFumeExtractionFan,
    ControlLaserEnable,
    ControlMachineEnable,
    ControlStatusLamp,
    ControlCoolerStirrer,
    ControlCoolerCompressor,
    ControlCoolerRadiatorFan,
    ControlCoolantPump,
}

#[derive(Debug)]
struct TableParameter {
    name: Parameter,
    value: Option<String>,
    updated: DateTime<Utc>,
}

impl TableParameter {
    fn new(name: Parameter) -> Self {
        Self {
            name,
            value: None,
            updated: Utc::now(),
        }
    }
}

struct App {
    items: Vec<TableParameter>,
    table_state: TableState,
}

impl Default for App {
    fn default() -> App {
        let items = Parameter::iter()
            .map(|p| TableParameter::new(p))
            .collect::<Vec<_>>();

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        App { items, table_state }
    }
}

impl App {
    fn next(&mut self) {
        self.table_state.scroll_down_by(1);
    }

    fn previous(&mut self) {
        self.table_state.scroll_up_by(1);
    }

    fn update_parameter(&mut self, parameter: Parameter, value: String) {
        self.items
            .iter_mut()
            .find(|item| item.name == parameter)
            .map(|item| {
                item.value = Some(value);
                item.updated = Utc::now();
            });
    }
}

fn event_to_values(event: EventKind) -> Vec<(Parameter, String)> {
    match event {
        EventKind::Boot(system_information) => vec![
            (
                Parameter::PeripheralControllerGitRevision,
                format!("{}", system_information.git_revision),
            ),
            (
                Parameter::PeripheralControllerBootReason,
                format!("{:?}", system_information.last_boot_reason),
            ),
        ],
        EventKind::CoolerBoot(system_information) => vec![
            (
                Parameter::CoolerGitRevision,
                format!("{}", system_information.git_revision),
            ),
            (
                Parameter::CoolerBootReason,
                format!("{:?}", system_information.last_boot_reason),
            ),
        ],
        EventKind::MonitorsChanged(monitors) => [
            (
                Parameter::MonitorMachinePowerOff,
                MonitorKind::MachinePowerOff,
            ),
            (
                Parameter::MonitorChassisIntrusion,
                MonitorKind::ChassisIntrusion,
            ),
            (
                Parameter::MonitorCoolerCommunicationFault,
                MonitorKind::CoolerCommunicationFault,
            ),
            (
                Parameter::MonitorCoolantHeaderTankLevelSensorFault,
                MonitorKind::CoolantHeaderTankLevelSensorFault,
            ),
            (
                Parameter::MonitorCoolantHeaderTankEmpty,
                MonitorKind::CoolantHeaderTankEmpty,
            ),
            (
                Parameter::MonitorCoolantHeaderTankOverfilled,
                MonitorKind::CoolantHeaderTankOverfilled,
            ),
            (
                Parameter::MonitorHeatExchangerFluidLow,
                MonitorKind::HeatExchangerFluidLow,
            ),
            (
                Parameter::MonitorCoolantFlowInsufficient,
                MonitorKind::CoolantFlowInsufficient,
            ),
            (
                Parameter::MonitorTemperatureSensorFaultA,
                MonitorKind::TemperatureSensorFaultA,
            ),
            (
                Parameter::MonitorTemperatureSensorFaultB,
                MonitorKind::TemperatureSensorFaultB,
            ),
            (
                Parameter::MonitorCoolantFlowOvertemperatureA,
                MonitorKind::CoolantFlowOvertemperatureA,
            ),
            (
                Parameter::MonitorCoolantFlowOvertemperatureB,
                MonitorKind::CoolantFlowOvertemperatureB,
            ),
            (
                Parameter::MonitorHeatExchangerOvertemperature,
                MonitorKind::HeatExchangerOvertemperature,
            ),
        ]
        .into_iter()
        .map(|(p, k)| (p, format!("{:?}", monitors.get(k))))
        .collect(),
        EventKind::LockoutChanged(machine_operation_lockout) => vec![(
            Parameter::MachineOperationLockout,
            format!("{:?}", machine_operation_lockout),
        )],
        EventKind::CoolingEnableChanged(cooling_enabled) => {
            vec![(Parameter::CoolingEnabled, format!("{:?}", cooling_enabled))]
        }
        EventKind::CoolingDemandChanged(cooling_demand) => {
            vec![(Parameter::CoolingDemand, format!("{:?}", cooling_demand))]
        }
        EventKind::Observation(ObservationEvent::MachinePower(value)) => {
            vec![(Parameter::ObservationMachinePower, format!("{:?}", value))]
        }
        EventKind::Observation(ObservationEvent::MachineRun(value)) => {
            vec![(Parameter::ObservationMachineRun, format!("{:?}", value))]
        }
        EventKind::Observation(ObservationEvent::ChassisIntrusion(value)) => vec![(
            Parameter::ObservationChassisIntrusion,
            format!("{:?}", value),
        )],
        EventKind::Observation(ObservationEvent::FumeExtractionMode(value)) => vec![(
            Parameter::ObservationFumeExtractionMode,
            format!("{:?}", value),
        )],
        EventKind::Observation(ObservationEvent::AirAssistDemand(value)) => vec![(
            Parameter::ObservationAirAssistDemand,
            format!("{:?}", value),
        )],
        EventKind::Observation(ObservationEvent::CoolantFlow(value)) => {
            vec![(Parameter::ObservationCoolantFlow, format!("{:?}", value))]
        }
        EventKind::Observation(ObservationEvent::HeatExchangerFluidLevel(value)) => vec![(
            Parameter::ObservationHeatExchangerFluidLevel,
            format!("{:?}", value),
        )],
        EventKind::Observation(ObservationEvent::CoolantHeaderTankLevel(value)) => vec![(
            Parameter::ObservationCoolantHeaderTankLevel,
            format!("{:?}", value),
        )],
        EventKind::Observation(ObservationEvent::TemperaturesA(value)) => vec![
            (
                Parameter::ObservationTemperatureAOnboard,
                format!("{:?}", value.onboard),
            ),
            (
                Parameter::ObservationTemperatureAElectronicsBay,
                format!("{:?}", value.electronics_bay_top),
            ),
            (
                Parameter::ObservationTemperatureALaserChamber,
                format!("{:?}", value.laser_chamber),
            ),
            (
                Parameter::ObservationTemperatureACoolantFlow,
                format!("{:?}", value.coolant_flow),
            ),
            (
                Parameter::ObservationTemperatureACoolantReturn,
                format!("{:?}", value.coolant_return),
            ),
        ],
        EventKind::Observation(ObservationEvent::TemperaturesB(value)) => vec![
            (
                Parameter::ObservationTemperatureBOnboard,
                format!("{:?}", value.onboard),
            ),
            (
                Parameter::ObservationTemperatureBCoolantFlow,
                format!("{:?}", value.coolant_flow),
            ),
            (
                Parameter::ObservationTemperatureBCoolantMid,
                format!("{:?}", value.coolant_mid),
            ),
            (
                Parameter::ObservationTemperatureBCoolantReturn,
                format!("{:?}", value.coolant_return),
            ),
            (
                Parameter::ObservationTemperatureBHeatExchangeFluid,
                format!("{:?}", value.heat_exchange_fluid),
            ),
            (
                Parameter::ObservationTemperatureBHeatExchangerLoop,
                format!("{:?}", value.heat_exchanger_loop),
            ),
        ],
        EventKind::Control(ControlEvent::AirAssistPump(value)) => {
            vec![(Parameter::ControlAirAssistPump, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::FumeExtractionFan(value)) => {
            vec![(Parameter::ControlFumeExtractionFan, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::LaserEnable(value)) => {
            vec![(Parameter::ControlLaserEnable, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::MachineEnable(value)) => {
            vec![(Parameter::ControlMachineEnable, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::StatusLamp(value)) => {
            vec![(Parameter::ControlStatusLamp, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::CoolerStirrer(value)) => {
            vec![(Parameter::ControlCoolerStirrer, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::CoolerCompressor(value)) => {
            vec![(Parameter::ControlCoolerCompressor, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::CoolerRadiatorFan(value)) => {
            vec![(Parameter::ControlCoolerRadiatorFan, format!("{:?}", value))]
        }
        EventKind::Control(ControlEvent::CoolantPump(value)) => {
            vec![(Parameter::ControlCoolantPump, format!("{:?}", value))]
        }
    }
}

pub(super) async fn run(
    mut client: Client<'static, SerialTransport, Request, Response>,
) -> Result<(), io::Error> {
    std::env::set_var("RUST_LOG", "nope");

    let (tx, mut rx) = tokio::sync::mpsc::channel(64);
    let (exit_tx, mut exit_rx) = tokio::sync::watch::channel(false);

    // Get new values
    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(50));

        loop {
            tokio::select! {
                    _ = ticker.tick() => {
                        if let Ok(Response::GetOldestEvent(Some(event))) = client
                            .call(
                                Request::GetOldestEvent,
                                core::time::Duration::from_millis(50),
                            )
                            .await
                        {
                            for (p, v) in event_to_values(event.kind){
                                let _ = tx.send((p, v)).await;
                            }
                        }
                    }
                    Ok(_) = exit_rx.changed() => {
                        return;
                    }
            }
        }
    });

    let mut app = App::default();

    let mut terminal = ratatui::init();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.area());

            let header = Row::new(vec!["Parameter", "Value", "Age", "Last Updated"]).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            );

            let now = Utc::now();

            let rows: Vec<Row> = app
                .items
                .iter()
                .map(|item| {
                    Row::new(vec![
                        format!("{:?}", item.name),
                        item.value
                            .clone()
                            .unwrap_or_else(|| "<unknown>".to_string()),
                        (now - item.updated).num_seconds().to_string(),
                        item.updated.to_string(),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(10),
                    Constraint::Percentage(20),
                ],
            )
            .header(header)
            .row_highlight_style(Style::default().bg(Color::Blue).fg(Color::Black));

            f.render_stateful_widget(table, chunks[0], &mut app.table_state);
        })?;

        // Handle keyboard input
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    _ => {}
                }
            }
        }

        // Handle new parameter values
        while let Ok((parameter, value)) = rx.try_recv() {
            app.update_parameter(parameter, value);
        }
    }

    let _ = exit_tx.send(true);
    let _ = handle.await;

    ratatui::restore();

    Ok(())
}
