#![allow(warnings)]

use miniquad::{
    info,
    native::android::{self, ndk_sys, ndk_utils},
};

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
    object: ndk_sys::jobject,
    pub address: String,
    // same string as an address, but java
    // to avoid jni string creation all the time
    address_j: ndk_sys::jobject,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Characteristic {
    characteristic: ndk_sys::jobject,
    pub id: String,
    /// PROPERTY_WRITE
    pub write: bool,
    ///  PROPERTY_READ
    pub read: bool,
    /// PROPERTY_NOTIFY
    pub notify: bool,
    /// PROPERTY_INDICATE
    pub indicate: bool,
    /// PROPERTY_BROADCAST
    pub broadcast: bool,
}

impl Device {
    pub fn id(&self) -> DeviceId {
        DeviceId(self.address.clone())
    }

    fn update_name(&mut self, env: *mut ndk_sys::JNIEnv) {
        if self.name.is_some() {
            return;
        }

        unsafe {
            let name =
                ndk_utils::call_object_method!(env, self.object, "getName", "()Ljava/lang/String;");
            if !name.is_null() {
                self.name = Some(ndk_utils::get_utf_str!(env, name).to_string());
            }
        }
    }
}

impl Characteristic {
    pub fn send_string(&self, data: &str) -> Result<(), BluetoothError> {
        let env = unsafe { android::attach_jni_env() };
        let mut globals = GLOBALS.lock().unwrap();

        let data = std::ffi::CString::new(data).unwrap();
        let string = unsafe { ((**env).NewStringUTF.unwrap())(env, data.as_ptr()) };
        unsafe {
            ndk_utils::call_void_method!(
                env,
                globals.quad_bt,
                "writeCharacteristicString",
                "(Landroid/bluetooth/BluetoothGattCharacteristic;Ljava/lang/String;)V",
                self.characteristic,
                string
            );
        }

        Ok(())
    }

    pub fn send_bytes(&self, data: &[u8], verify: bool) -> Result<(), BluetoothError> {
        info!("send_bytes: {:?} {:?} {:?}", self.id, data, verify);
        let env = unsafe { android::attach_jni_env() };
        let mut globals = GLOBALS.lock().unwrap();

        unsafe {
            let array = (**env).NewByteArray.unwrap()(env, data.len() as _);
            assert!(!array.is_null());
            assert!((**env).GetArrayLength.unwrap()(env, array) == data.len() as i32);
            let temp = (**env).GetPrimitiveArrayCritical.unwrap()(env, array, std::ptr::null_mut());
            std::ptr::copy_nonoverlapping(data.as_ptr(), temp as _, data.len());
            (**env).ReleasePrimitiveArrayCritical.unwrap()(env, array, temp, 0);

            ndk_utils::call_void_method!(
                env,
                globals.quad_bt,
                "writeCharacteristicBytes",
                "(Landroid/bluetooth/BluetoothGattCharacteristic;[BZ)V",
                self.characteristic,
                ndk_utils::new_local_ref!(env, array),
                verify as i32
            );
        }

        Ok(())
    }

    pub fn set_notification(&self, notify: bool) -> Result<(), BluetoothError> {
        let globals = GLOBALS.lock().unwrap();
        let env = unsafe { android::attach_jni_env() };

        unsafe {
            ndk_utils::call_void_method!(
                env,
                globals.quad_bt,
                "setCharacteristicNotification",
                "(Landroid/bluetooth/BluetoothGattCharacteristic;Z)V",
                self.characteristic,
                notify as i32
            );
        }

        Ok(())
    }

    pub fn set_indication(&self, notify: bool) -> Result<(), BluetoothError> {
        let globals = GLOBALS.lock().unwrap();
        let env = unsafe { android::attach_jni_env() };

        unsafe {
            ndk_utils::call_void_method!(
                env,
                globals.quad_bt,
                "setCharacteristicIndication",
                "(Landroid/bluetooth/BluetoothGattCharacteristic;Z)V",
                self.characteristic,
                notify as i32
            );
        }

        Ok(())
    }
}
struct GlobalData {
    quad_bt: ndk_sys::jobject,
    devices: HashMap<String, Device>,
    tx: Option<Sender<Message>>,
    rx: Option<Receiver<Vec<u8>>>,
}

unsafe impl Send for GlobalData {}
unsafe impl Sync for GlobalData {}

static GLOBALS: Lazy<Mutex<GlobalData>> = Lazy::new(|| {
    let data = GlobalData {
        quad_bt: std::ptr::null_mut(),
        devices: HashMap::new(),
        tx: None,
        rx: None,
    };
    Mutex::new(data)
});

const PROPERTY_BROADCAST: i32 = 0x00000001;
const PROPERTY_READ: i32 = 0x00000002;
const PROPERTY_WRITE: i32 = 0x00000008;
const PROPERTY_NOTIFY: i32 = 0x00000010;
const PROPERTY_INDICATE: i32 = 0x00000020;

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onDeviceFound(
    env: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    device: ndk_sys::jobject,
) {
    let mut globals = GLOBALS.lock().unwrap();

    let device_addr_j =
        ndk_utils::call_object_method!(env, device, "getAddress", "()Ljava/lang/String;");

    let device_addr = ndk_utils::get_utf_str!(env, device_addr_j);

    globals.devices.insert(
        device_addr.to_string(),
        Device {
            address: device_addr.to_string(),
            address_j: ndk_utils::new_global_ref!(env, device_addr_j),
            name: None,
            object: ndk_utils::new_global_ref!(env, device),
        },
    );
}

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onDataAvailable(
    env: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    array: ndk_sys::jobject,
) {
    let len = ((**env).GetArrayLength.unwrap())(env, array);
    let elements = ((**env).GetByteArrayElements.unwrap())(env, array, std::ptr::null_mut());
    let data = std::slice::from_raw_parts(elements as *mut u8, len as usize);

    let mut globals = GLOBALS.lock().unwrap();

    if let Some(ref mut tx) = globals.tx {
        tx.send(Message::Data(data.to_vec())).unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onCharacteristicDiscovered(
    env: *mut ndk_sys::JNIEnv,
    _: ndk_sys::jobject,
    characteristic: ndk_sys::jobject,
) {
    let uuid = ndk_utils::call_object_method!(env, characteristic, "getUuid", "()Ljava/util/UUID;");
    let uuid = ndk_utils::call_object_method!(env, uuid, "toString", "()Ljava/lang/String;");
    let uuid = ndk_utils::get_utf_str!(env, uuid);

    let properties: i32 = ndk_utils::call_int_method!(env, characteristic, "getProperties", "()I");

    let mut globals = GLOBALS.lock().unwrap();

    if let Some(ref mut tx) = globals.tx {
        tx.send(Message::CharacteristicDiscovered(Characteristic {
            id: uuid.to_owned(),
            characteristic: ndk_utils::new_global_ref!(env, characteristic),
            write: (properties & PROPERTY_WRITE) != 0,
            broadcast: (properties & PROPERTY_BROADCAST) != 0,
            read: (properties & PROPERTY_READ) != 0,
            notify: (properties & PROPERTY_NOTIFY) != 0,
            indicate: (properties & PROPERTY_INDICATE) != 0,
        }))
        .unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onGattConnected() {
    let mut globals = GLOBALS.lock().unwrap();
    if let Some(ref mut tx) = globals.tx {
        tx.send(Message::Connected).unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onGattDisconnected() {
    let mut globals = GLOBALS.lock().unwrap();
    if let Some(ref mut tx) = globals.tx {
        tx.send(Message::Disconnected).unwrap();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Java_quadbt_QuadBT_onServiceConnected() {
    let env = android::attach_jni_env();

    let quad_bt = ndk_utils::new_object!(env, "quadbt/QuadBT", "()V");
    assert!(!quad_bt.is_null());

    let quad_bt = ndk_utils::new_global_ref!(env, quad_bt);
    GLOBALS.lock().unwrap().quad_bt = quad_bt;
}

pub struct Adapter;

impl Adapter {
    pub fn new() -> Result<Adapter, BluetoothError> {
        Ok(Adapter)
    }

    pub fn is_ready(&self) -> bool {
        let quad_bt = GLOBALS.lock().unwrap().quad_bt;
        if quad_bt.is_null() {
            return false;
        }

        unsafe {
            let env = android::attach_jni_env();

            ndk_utils::call_bool_method!(env, quad_bt, "isEnabled", "()Z") != 0
        }
    }

    pub fn start_scan(&mut self) -> Result<(), BluetoothError> {
        let quad_bt = GLOBALS.lock().unwrap().quad_bt;
        if quad_bt.is_null() {
            return Err(BluetoothError::AdapterNotReady);
        }

        unsafe {
            let env = android::attach_jni_env();

            ndk_utils::call_void_method!(env, quad_bt, "startScan", "()V");
        }

        Ok(())
    }

    pub fn walk_devices<F: FnMut(&Device)>(&mut self, mut f: F) -> Result<(), BluetoothError> {
        let env = unsafe { android::attach_jni_env() };

        let mut globals = GLOBALS.lock().unwrap();

        globals.devices.values_mut().for_each(|d| {
            d.update_name(env);
            f(d)
        });

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
        let env = unsafe { android::attach_jni_env() };

        let mut globals = GLOBALS.lock().unwrap();

        if globals.quad_bt.is_null() {
            return Err(BluetoothError::AdapterNotReady);
        }

        if !globals.devices.contains_key(&device_id.0) {
            return Err(BluetoothError::DeviceUnavailable);
        }

        let device = &globals.devices[&device_id.0];
        unsafe {
            ndk_utils::call_void_method!(
                env,
                globals.quad_bt,
                "connect",
                "(Ljava/lang/String;)V",
                device.address_j
            );
        }

        let (_, rx) = mpsc::channel();
        let (tx, client_rx) = mpsc::channel();

        globals.tx = Some(tx);
        globals.rx = Some(rx);

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
        let env = unsafe { android::attach_jni_env() };
        let mut globals = GLOBALS.lock().unwrap();

        unsafe {
            ndk_utils::call_void_method!(env, globals.quad_bt, "disconnect", "()V");
        }

        Ok(())
    }
}
