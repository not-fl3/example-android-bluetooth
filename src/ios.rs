#![allow(warnings)]

use miniquad::info;

use once_cell::sync::Lazy;
use std::{collections::HashMap, fmt, sync::Mutex};

use std::sync::mpsc::{self, Receiver, Sender};

//use objc::{msg_send, class, sel, sel_impl};
use miniquad::native::apple::{apple_util::*, frameworks::*};

#[link(name = "CoreBluetooth", kind = "framework")]
extern "C" {}

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
    peripheral: ObjcId,
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub id: String,
    characteristic: ObjcId,
    peripheral: ObjcId,
    // pub write: bool,
    // pub read: bool,
    // pub notify: bool,
    // pub indicate: bool,
    // pub broadcast: bool,
}

impl Device {
    pub fn id(&self) -> DeviceId {
        DeviceId(self.address.clone())
    }
}

impl Characteristic {
    pub fn send_string(&self, data: &str) -> Result<(), BluetoothError> {
        self.send_bytes(data.as_bytes(), false)
    }

    pub fn send_bytes(&self, data: &[u8], verify: bool) -> Result<(), BluetoothError> {
        unsafe {
            let data: ObjcId = msg_send![class!(NSData),
                                         dataWithBytes:data.as_ptr()
                                         length: data.len()];
            let () = msg_send![self.peripheral,
                              writeValue:data
                               forCharacteristic:self.characteristic
                               type:if verify {1} else {0}
            ];
        }

        Ok(())
    }

    pub fn set_notification(&self, notify: bool) -> Result<(), BluetoothError> {
        unsafe {
            let () = msg_send![self.peripheral,
                              setNotifyValue:YES
                              forCharacteristic:self.characteristic
            ];
        }
        Ok(())
    }

    pub fn set_indication(&self, notify: bool) -> Result<(), BluetoothError> {
        unsafe {
            let () = msg_send![self.peripheral,
                              setNotifyValue:YES
                              forCharacteristic:self.characteristic
            ];
        }
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

pub struct Adapter {
    blue_central: ObjcId,
}

#[repr(usize)]
#[derive(Debug, PartialEq)]
pub enum ManagerState {
    Unknown = 0,
    Resetting,
    Unsupported,
    Unauthorized,
    PoweredOff,
    PoweredOn,
}

pub fn define_central_manager_delegate() -> *const Class {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("QuadBTCentralManager", superclass).unwrap();

    decl.add_protocol(Protocol::get("CBCentralManagerDelegate").unwrap());
    decl.add_protocol(Protocol::get("CBPeripheralDelegate").unwrap());

    extern "C" fn central_manager_did_update_state(_: &Object, _: Sel, central: ObjcId) {
        unsafe {
            let state: ManagerState = msg_send![central, state];
            miniquad::warn!("{:?}", state);

            if state == ManagerState::PoweredOn {
                let _: () = msg_send![central,
                                      scanForPeripheralsWithServices:nil
                                      options:nil];
            }
        };
    }

    extern "C" fn did_discover_peripheral(
        this: &Object,
        _: Sel,
        _central: ObjcId,
        peripheral: ObjcId,
        _advertisement_data: ObjcId,
        _rssi: ObjcId,
    ) {
        unsafe {
            let peripheral: ObjcId = msg_send![peripheral, retain];

            let () = msg_send![peripheral, setDelegate: this];

            let name: ObjcId = msg_send![peripheral, name];
            let name = nsstring_to_string(name);

            let uuid: ObjcId = msg_send![peripheral, identifier];
            let uuid: ObjcId = msg_send![uuid, UUIDString];
            let uuid = nsstring_to_string(uuid);

            let mut globals = GLOBALS.lock().unwrap();
            globals.devices.insert(
                uuid.clone(),
                Device {
                    peripheral,
                    name: Some(name),
                    address: uuid,
                },
            );
        }
    }

    extern "C" fn did_connect_peripheral(
        this: &Object,
        _: Sel,
        _central: ObjcId,
        peripheral: ObjcId,
    ) {
        info!("connect peripheral?");

        unsafe {
            let string = str_to_nsstring("6E400001-B5A3-F393-E0A9-E50E24DCCA9E");
            let cbuuid: ObjcId = msg_send![class!(CBUUID), UUIDWithString: string];

            let arr: ObjcId = msg_send![class!(NSArray), arrayWithObject: cbuuid];
            let () = msg_send![peripheral, discoverServices: nil];
        }
    }

    extern "C" fn did_disconnect_peripheral(
        this: &Object,
        _: Sel,
        _central: ObjcId,
        _peripheral: ObjcId,
        _error: ObjcId,
    ) {
        panic!("disconnect?");
    }

    extern "C" fn did_fail_to_connect_peripheral(
        this: &Object,
        _: Sel,
        _central: ObjcId,
        _peripheral: ObjcId,
        error: ObjcId,
    ) {
        panic!("error!")
    }

    extern "C" fn connection_event_did_occur(
        this: &Object,
        _: Sel,
        _central: ObjcId,
        event: ObjcId,
        peripheral: ObjcId,
    ) {
        panic!("event?!")
    }

    extern "C" fn did_discover_services(this: &Object, _: Sel, peripheral: ObjcId, error: ObjcId) {
        unsafe {
            let services: ObjcId = msg_send![peripheral, services];

            let string = str_to_nsstring("6e400002-b5a3-f393-e0a9-e50e24dcca9e");
            let cbuuid: ObjcId = msg_send![class!(CBUUID), UUIDWithString: string];

            let string = str_to_nsstring("6e400003-b5a3-f393-e0a9-e50e24dcca9e");
            let cbuuid1: ObjcId = msg_send![class!(CBUUID), UUIDWithString: string];

            let count: usize = msg_send![services, count];

            let arr = [cbuuid, cbuuid1, nil];
            let chars: ObjcId = msg_send![class!(NSArray),
                                      arrayWithObjects: arr.as_ptr()
                                      count:2];
            for i in 0..count {
                let service: ObjcId = msg_send![services, objectAtIndex: i];
                let () = msg_send![peripheral, discoverCharacteristics:chars forService:service];
            }
        }
    }

    extern "C" fn did_discover_characteristics_for_service(
        this: &Object,
        _: Sel,
        peripheral: ObjcId,
        service: ObjcId,
        error: ObjcId,
    ) {
        unsafe {
            let characteristics: ObjcId = msg_send![service, characteristics];
            let count: usize = msg_send![characteristics, count];

            for i in 0..count {
                let characteristic: ObjcId = msg_send![characteristics, objectAtIndex: i];

                let uuid: ObjcId = msg_send![characteristic, UUID];
                let uuid: ObjcId = msg_send![uuid, UUIDString];
                let uuid = nsstring_to_string(uuid);
                info!("{}", uuid);

                let mut globals = GLOBALS.lock().unwrap();
                if let Some(ref mut tx) = globals.tx {
                    tx.send(Message::CharacteristicDiscovered(Characteristic {
                        id: uuid.to_owned(),
                        characteristic: msg_send![characteristic, retain],
                        peripheral: msg_send![peripheral, retain],
                    }))
                    .unwrap();
                }
            }
        }
    }

    extern "C" fn did_update_notification_state_for_characteristic(
        this: &Object,
        _: Sel,
        peripheral: ObjcId,
        characteristic: ObjcId,
        error: ObjcId,
    ) {
        unsafe {
            let value: ObjcId = msg_send![characteristic, value];
            let length: usize = msg_send![value, length];
            let bytes: *const u8 = msg_send![value, bytes];
            let bytes = std::slice::from_raw_parts(bytes, length);
            let mut globals = GLOBALS.lock().unwrap();
            if let Some(ref mut tx) = globals.tx {
                tx.send(Message::Data(bytes.to_vec())).unwrap();
            }
        }
    }

    unsafe {
        decl.add_method(
            sel!(centralManagerDidUpdateState:),
            central_manager_did_update_state as extern "C" fn(&Object, Sel, ObjcId),
        );
        decl.add_method(
            sel!(centralManager:didDiscoverPeripheral:advertisementData:RSSI:),
            did_discover_peripheral as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(centralManager:didConnectPeripheral:),
            did_connect_peripheral as extern "C" fn(&Object, Sel, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(centralManager:didDisconnectPeripheral:error:),
            did_disconnect_peripheral as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(centralManager:didFailToConnectPeripheral:error:),
            did_fail_to_connect_peripheral as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(centralManager:connectionEventDidOccur:connectionEventDidOccur:),
            connection_event_did_occur as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );

        decl.add_method(
            sel!(peripheral:didDiscoverServices:),
            did_discover_services as extern "C" fn(&Object, Sel, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(peripheral:didDiscoverCharacteristicsForService:error:),
            did_discover_characteristics_for_service
                as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(peripheral:didUpdateNotificationStateForCharacteristic:error:),
            did_update_notification_state_for_characteristic
                as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );
        decl.add_method(
            sel!(peripheral:didUpdateValueForCharacteristic:error:),
            did_update_notification_state_for_characteristic
                as extern "C" fn(&Object, Sel, ObjcId, ObjcId, ObjcId),
        );
    }

    return decl.register();
}

mod dispatch {
    #[repr(C)]
    pub struct dispatch_object_s {
        _private: [u8; 0],
    }
    pub type dispatch_queue_t = *mut dispatch_object_s;

    #[cfg_attr(
        any(target_os = "macos", target_os = "ios"),
        link(name = "System", kind = "dylib")
    )]
    #[cfg_attr(
        not(any(target_os = "macos", target_os = "ios")),
        link(name = "dispatch", kind = "dylib")
    )]
    extern "C" {
        static _dispatch_main_q: dispatch_object_s;
    }
    pub fn dispatch_get_main_queue() -> dispatch_queue_t {
        unsafe { &_dispatch_main_q as *const _ as dispatch_queue_t }
    }
}

impl Adapter {
    pub fn new() -> Result<Adapter, BluetoothError> {
        unsafe {
            let delegate_class: ObjcId = msg_send!(define_central_manager_delegate(), class);
            let delegate: ObjcId = msg_send!(delegate_class, new);

            let blue_central: ObjcId = msg_send![class!(CBCentralManager), alloc];
            //let queue = dispatch::dispatch_get_main_queue();
            let blue_central: ObjcId = msg_send![blue_central, initWithDelegate:delegate
                                                 queue:nil];

            Ok(Adapter { blue_central })
        }
    }

    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn start_scan(&mut self) -> Result<(), BluetoothError> {
        miniquad::warn!("start_scan");
        // unsafe {
        //     let _: () = msg_send![self.blue_central,
        //                           scanForPeripheralsWithServices:nil
        //                           options:nil];
        // };

        Ok(())
    }

    pub fn walk_devices<F: FnMut(&Device)>(&mut self, mut f: F) -> Result<(), BluetoothError> {
        let mut globals = GLOBALS.lock().unwrap();
        globals.devices.values_mut().for_each(|d| f(d));

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
        let mut globals = GLOBALS.lock().unwrap();

        let (tx, client_rx) = mpsc::channel();

        globals.tx = Some(tx);

        let device = &globals.devices.get(&device_id.0).unwrap();

        unsafe {
            let () = msg_send![self.blue_central, stopScan];
            let () = msg_send![self.blue_central,
                               connectPeripheral:device.peripheral
                               options:nil];
        };

        Ok(Connection {
            device_id,
            rx: client_rx,
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
