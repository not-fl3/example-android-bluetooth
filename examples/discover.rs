//! List bluetooth devices available, connect and get a GATT data.

use macroquad::{
    prelude::*,
    ui::{root_ui, widgets},
};
use quad_bt::{self as bt, Message};
use std::collections::VecDeque;

enum State {
    BluetoothNotReady,
    Scan,
    Connected(bt::Connection),
}

#[macroquad::main("List BT Devices")]
async fn main() {
    let mut state = State::BluetoothNotReady;
    let mut received_data = VecDeque::new();
    let mut adapter = bt::Adapter::new().unwrap();

    let mut characteristics = vec![];
    loop {
        match state {
            State::BluetoothNotReady => {
                if adapter.is_ready() {
                    adapter.start_scan().unwrap();
                    state = State::Scan;
                }
            }
            State::Connected(ref mut connection) => {
                let mut done = false;
                while let Ok(Some(msg)) = connection.try_recv() {
                    match msg {
                        Message::CharacteristicDiscovered(characteristic) => {
                            info!("Got characteristic: {:?}", characteristic);
                            characteristics.push(characteristic);
                        }
                        Message::Data(data) => {
                            info!("Received data: {:?}", data);
                            received_data.push_front(data);
                            if received_data.len() > 20 {
                                received_data.pop_back();
                            }
                        }
                        Message::Disconnected => {
                            info!("Disconnected!");
                            done = true;
                        }
                        _ => {}
                    }
                }
                if done {
                    received_data.clear();
                    characteristics.clear();
                    adapter.start_scan().unwrap();
                    state = State::Scan;
                }
            }
            _ => {}
        }

        clear_background(WHITE);

        match state {
            State::BluetoothNotReady => root_ui().label(None, "Bluetooth initializing"),
            State::Scan => {
                root_ui().label(None, "Devices:");

                let mut device_id = None;
                adapter
                    .walk_devices(|device| {
                        if widgets::Button::new(format!("{:?} {:?}", device.address, device.name))
                            .size(vec2(400., 50.))
                            .ui(&mut *root_ui())
                        {
                            device_id = Some(device.id());
                        }
                    })
                    .unwrap();

                if let Some(device_id) = device_id {
                    let connection = adapter.connect(device_id.clone()).unwrap();
                    state = State::Connected(connection);
                }
            }
            State::Connected(ref mut connection) => {
                for characteristic in &characteristics {
                    widgets::Label::new(format!("{:?}", &characteristic.id)).ui(&mut *root_ui());

                    widgets::Label::new(format!(
                        "write: {:?}, read: {:?}, notify: {:?}, indicate: {:?}",
                        characteristic.write,
                        characteristic.read,
                        characteristic.notify,
                        characteristic.indicate
                    ))
                    .ui(&mut *root_ui());

                    if widgets::Button::new("write 1")
                        .size(vec2(100., 50.))
                        .ui(&mut *root_ui())
                    {
                        characteristic.send_bytes(&[0x01], true).unwrap();
                    }
                    root_ui().same_line(110.);
                    if widgets::Button::new("write Uxx")
                        .size(vec2(100., 50.))
                        .ui(&mut *root_ui())
                    {
                        characteristic.send_string("Uxx").unwrap();
                    }
                    root_ui().same_line(220.);

                    if widgets::Button::new("notify")
                        .size(vec2(100., 50.))
                        .ui(&mut *root_ui())
                    {
                        characteristic.set_notification(true).unwrap();
                    }
                    root_ui().same_line(330.);
                    if widgets::Button::new("indicate")
                        .size(vec2(100., 50.))
                        .ui(&mut *root_ui())
                    {
                        characteristic.set_indication(true).unwrap();
                    }
                }
                if widgets::Button::new("disconnect")
                    .position(vec2(screen_width() - 200., screen_height() - 50.))
                    .size(vec2(200., 50.))
                    .ui(&mut *root_ui())
                {
                    connection.disconnect().unwrap();
                }
            }
        }

        for (n, data) in received_data.iter().enumerate() {
            widgets::Label::new(format!("{:?}", data))
                .position(vec2(450., n as f32 * 20.))
                .ui(&mut *root_ui());
        }
        next_frame().await;
    }
}
