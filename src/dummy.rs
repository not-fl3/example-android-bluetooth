#![allow(warnings)]

use miniquad::info;

use once_cell::sync::Lazy;
use std::{collections::HashMap, fmt, sync::Mutex};

use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub enum BluetoothError {
    AdapterNotReady,
    DeviceUnavailable,
    DeviceDisconnected,
}

impl fmt::Display for BluetoothError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for BluetoothError {}

#[derive(Clone, Debug)]
pub struct DeviceId(String);

#[derive(Clone)]
pub struct Device {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub id: String,
    pub write: bool,
    pub read: bool,
    pub notify: bool,
    pub indicate: bool,
    pub broadcast: bool,
}

impl Device {
    pub fn id(&self) -> DeviceId {
        DeviceId(self.address.clone())
    }

    // fn update_name(&mut self, env: *mut ndk_sys::JNIEnv) {
    //     if self.name.is_some() {
    //         return;
    //     }
    // }
}

impl Characteristic {
    pub fn send_string(&self, data: &str) -> Result<(), BluetoothError> {
        Ok(())
    }

    pub fn send_bytes(&self, data: &[u8], verify: bool) -> Result<(), BluetoothError> {
        Ok(())
    }

    pub fn set_notification(&self, notify: bool) -> Result<(), BluetoothError> {
        Ok(())
    }

    pub fn set_indication(&self, notify: bool) -> Result<(), BluetoothError> {
        Ok(())
    }
}
struct GlobalData {
    devices: HashMap<String, Device>,
    tx: Option<Sender<Message>>,
    rx: Option<Receiver<Vec<u8>>>,
}

unsafe impl Send for GlobalData {}
unsafe impl Sync for GlobalData {}

static GLOBALS: Lazy<Mutex<GlobalData>> = Lazy::new(|| {
    let data = GlobalData {
        devices: HashMap::new(),
        tx: None,
        rx: None,
    };
    Mutex::new(data)
});

pub struct Adapter;

impl Adapter {
    pub fn new() -> Result<Adapter, BluetoothError> {
        Ok(Adapter)
    }

    pub fn is_ready(&self) -> bool {
        false
    }

    pub fn start_scan(&mut self) -> Result<(), BluetoothError> {
        Ok(())
    }

    pub fn walk_devices<F: FnMut(&Device)>(&mut self, mut f: F) -> Result<(), BluetoothError> {
        Ok(())
    }

    pub fn get_device_name(&self, device_id: &DeviceId) -> Option<String> {
        let globals = GLOBALS.lock().unwrap();

        globals
            .devices
            .get(&device_id.0)
            .and_then(|d| d.name.clone())
    }

    pub fn connect(&mut self, device_id: DeviceId) -> Result<Connection, BluetoothError> {
        let (tx, client_rx) = mpsc::channel();

        Ok(Connection {
            device_id: DeviceId("".to_string()), 
            rx: client_rx
        })
    }
}

pub enum Message {
    Connected,
    Disconnected,
    Data(Vec<u8>),
    CharacteristicDiscovered(Characteristic),
}

pub struct Connection {
    device_id: DeviceId,
    rx: Receiver<Message>,
}

impl Connection {
    pub fn device_id(&self) -> DeviceId {
        self.device_id.clone()
    }

    pub fn try_recv(&mut self) -> Result<Option<Message>, BluetoothError> {
        Ok(self.rx.try_recv().ok())
    }

    pub fn disconnect(&mut self) -> Result<(), BluetoothError> {
        Ok(())
    }
}
