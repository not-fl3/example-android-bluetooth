package quadbt;

import android.app.Service;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothGattCallback;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattDescriptor;
import android.bluetooth.BluetoothGattService;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothProfile;
import android.bluetooth.le.BluetoothLeScanner;
import android.bluetooth.le.ScanCallback;
import android.bluetooth.le.ScanResult;
import android.content.Context;
import android.content.Intent;
import android.os.Binder;
import android.os.IBinder;
import android.util.Log;

import java.util.List;
import java.util.UUID;

import java.nio.charset.StandardCharsets;


public class BluetoothLeService extends Service {
    private BluetoothManager mBluetoothManager;
    private BluetoothAdapter mBluetoothAdapter;
    private String mBluetoothDeviceAddress;
    private BluetoothGatt mBluetoothGatt;
    private int mConnectionState = STATE_DISCONNECTED;

    private static final int STATE_DISCONNECTED = 0;
    private static final int STATE_CONNECTING = 1;
    private static final int STATE_CONNECTED = 2;

    public final static String ACTION_GATT_CONNECTED =
            "quadbt.ACTION_GATT_CONNECTED";
    public final static String ACTION_GATT_DISCONNECTED =
            "quadbt.ACTION_GATT_DISCONNECTED";
    public final static String ACTION_GATT_SERVICES_DISCOVERED =
            "quadbt.ACTION_GATT_SERVICES_DISCOVERED";
    public final static String ACTION_DATA_AVAILABLE =
            "quadbt.ACTION_DATA_AVAILABLE";
    public final static String EXTRA_DATA =
            "quadbt.EXTRA_DATA";

    final Object mLock = new Object();
    boolean writen = false;
    
    private final BluetoothGattCallback mGattCallback = new BluetoothGattCallback() {
        @Override
        public void onConnectionStateChange(BluetoothGatt gatt, int status, int newState) {
            String intentAction;
            if (newState == BluetoothProfile.STATE_CONNECTED) {
                intentAction = ACTION_GATT_CONNECTED;
                mConnectionState = STATE_CONNECTED;
                broadcastUpdate(intentAction);
                Log.w("SAPP", "Connected to GATT server.");
                // Attempts to discover services after successful connection.
                Log.w("SAPP", "Attempting to start service discovery:" +
                      mBluetoothGatt.discoverServices());
                
            } else if (newState == BluetoothProfile.STATE_DISCONNECTED) {
                mBluetoothGatt.close();
                intentAction = ACTION_GATT_DISCONNECTED;
                mConnectionState = STATE_DISCONNECTED;
                Log.w("SAPP", "Disconnected from GATT server.");
                broadcastUpdate(intentAction);
            }
        }

        @Override
        public void onServicesDiscovered(BluetoothGatt gatt, int status) {
            Log.w("SAPP", "Services discovered");
            if (status == BluetoothGatt.GATT_SUCCESS) {
                broadcastUpdate(ACTION_GATT_SERVICES_DISCOVERED);
            } else {
                Log.w("SAPP", "onServicesDiscovered received: " + status);
            }
        }

        @Override
        public void onCharacteristicRead(BluetoothGatt gatt,
                                         BluetoothGattCharacteristic characteristic,
                                         int status) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                broadcastUpdate(ACTION_DATA_AVAILABLE, characteristic);
            }
        }

        @Override
        public void onCharacteristicWrite(BluetoothGatt gatt,
                                          BluetoothGattCharacteristic characteristic,
                                          int status) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                Log.w("SAPP", "char write OK " + characteristic.getUuid());
            } else {
                Log.w("SAPP", "char write NOT OK: " + status);
            }

            synchronized (mLock) {
                writen = true;
                mLock.notifyAll();
            }
        }

        @Override
        public void onDescriptorWrite(BluetoothGatt gatt,
                                       BluetoothGattDescriptor characteristic,
                                       int status) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                Log.w("SAPP", "Descriptor write success!");
            } else {
                Log.e("SAPP", "Descriptor write error: " + status);
            }

            synchronized (mLock) {
                mLock.notifyAll();
            }
        }
        @Override
        public void onCharacteristicChanged(BluetoothGatt gatt,
                                            BluetoothGattCharacteristic characteristic) {
            broadcastUpdate(ACTION_DATA_AVAILABLE, characteristic);
        }
    };

    private void broadcastUpdate(final String action) {
        final Intent intent = new Intent(action);
        sendBroadcast(intent);
    }

    private void broadcastUpdate(final String action,
                                 final BluetoothGattCharacteristic characteristic) {
        final Intent intent = new Intent(action);

        final byte[] data = characteristic.getValue();

        //final String text = new String(data, StandardCharsets.UTF_8);

        intent.putExtra(EXTRA_DATA, data);

        sendBroadcast(intent);
    }

    public class LocalBinder extends Binder {
        public BluetoothLeService getService() {
            return BluetoothLeService.this;
        }
    }

    @Override
    public IBinder onBind(Intent intent) {
        return binder;
    }

    @Override
    public boolean onUnbind(Intent intent) {
        close();
        return super.onUnbind(intent);
    }

    private final IBinder binder = new LocalBinder();

    public boolean initialize() {
        // For API level 18 and above, get a reference to BluetoothAdapter through
        // BluetoothManager.
        if (mBluetoothManager == null) {
            mBluetoothManager = (BluetoothManager) getSystemService(Context.BLUETOOTH_SERVICE);
            if (mBluetoothManager == null) {
                Log.e("SAPP", "Unable to initialize BluetoothManager.");
                return false;
            }
        }

        mBluetoothAdapter = mBluetoothManager.getAdapter();
        if (mBluetoothAdapter == null) {
            Log.e("SAPP", "Unable to obtain a BluetoothAdapter.");
            return false;
        }

        return true;
    }

    public boolean connect(final String address) {
        if (mBluetoothAdapter == null || address == null) {
            Log.w("SAPP", "BluetoothAdapter not initialized or unspecified address.");
            return false;
        }

        // if (mBluetoothDeviceAddress != null && address.equals(mBluetoothDeviceAddress)
        //         && mBluetoothGatt != null) {
        //     Log.d("SAPP", "Trying to use an existing mBluetoothGatt for connection.");
        //     if (mBluetoothGatt.connect()) {
        //         mConnectionState = STATE_CONNECTING;
        //         return true;
        //     } else {
        //         return false;
        //     }
        // }

        final BluetoothDevice device = mBluetoothAdapter.getRemoteDevice(address);
        if (device == null) {
            Log.w("SAPP", "Device not found.  Unable to connect.");
            return false;
        }

        mBluetoothGatt = device.connectGatt(this, false, mGattCallback);
        Log.d("SAPP", "Trying to create a new connection.");
        mBluetoothDeviceAddress = address;
        mConnectionState = STATE_CONNECTING;
        return true;
    }

    public void disconnect() {
        if (mBluetoothAdapter == null || mBluetoothGatt == null) {
            Log.w("SAPP", "BluetoothAdapter not initialized");
            return;
        }
        mBluetoothGatt.disconnect();
    }

    public void close() {
        if (mBluetoothGatt == null) {
            return;
        }
        mBluetoothGatt.close();
        mBluetoothGatt = null;
    }
    
    public void writeCharacteristic(BluetoothGattCharacteristic characteristic, String data) {
        characteristic.setValue(data);
        mBluetoothGatt.writeCharacteristic(characteristic);

    }

    public void writeCharacteristic(BluetoothGattCharacteristic characteristic, byte[] data, boolean verify) {
        writen = false;
        if (verify) {
            characteristic.setWriteType(BluetoothGattCharacteristic.WRITE_TYPE_DEFAULT);
        } else {
            characteristic.setWriteType(BluetoothGattCharacteristic.WRITE_TYPE_NO_RESPONSE);
        }
        characteristic.setValue(data);
        mBluetoothGatt.writeCharacteristic(characteristic);

            try {
                synchronized (mLock) {
                    while (!writen) {
                        mLock.wait();
                    }
                }
            } catch (final InterruptedException e) {
                Log.e("SAPP", "setCharacterIndication: can't wait for the descriptor valye " + e);
            }

    }

    public void readCharacteristic(BluetoothGattCharacteristic characteristic) {
        if (mBluetoothAdapter == null || mBluetoothGatt == null) {
            Log.w("SAPP", "BluetoothAdapter not initialized");
            return;
        }
        Log.w("SAPP", "read Characteristic " + characteristic.getUuid());

        mBluetoothGatt.readCharacteristic(characteristic);
    }

    public void setCharacteristicNotification(BluetoothGattCharacteristic characteristic, boolean enabled) {
        if (mBluetoothAdapter == null || mBluetoothGatt == null) {
            Log.w("SAPP", "BluetoothAdapter not initialized");
            return;
        }
        mBluetoothGatt.setCharacteristicNotification(characteristic, enabled);

        BluetoothGattDescriptor descriptor = characteristic.getDescriptor(UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"));
        descriptor.setValue(BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE);
        mBluetoothGatt.writeDescriptor(descriptor);

        try {
            synchronized (mLock) {
                while (true) {
                    mLock.wait();
                    byte[] value = descriptor.getValue();
                    if (value != null && value.length == 2 && value[0] == 1 && value[1] == 0) {
                        break;
                    }
                }
            }
        } catch (final InterruptedException e) {
			Log.e("SAPP", "setCharacterIndication: can't wait for the descriptor valye " + e);
		}
    }

    public void setCharacteristicIndication(BluetoothGattCharacteristic characteristic, boolean enabled) {
        if (mBluetoothAdapter == null || mBluetoothGatt == null) {
            Log.w("SAPP", "BluetoothAdapter not initialized");
            return;
        }
        mBluetoothGatt.setCharacteristicNotification(characteristic, enabled);

        BluetoothGattDescriptor descriptor = characteristic.getDescriptor(UUID.fromString("00002902-0000-1000-8000-00805f9b34fb"));
        descriptor.setValue(BluetoothGattDescriptor.ENABLE_INDICATION_VALUE);
        mBluetoothGatt.writeDescriptor(descriptor);

        try {
            synchronized (mLock) {
                while (true) {
                    mLock.wait();
                    byte[] value = descriptor.getValue();
                    if (value != null && value.length == 2 && value[0] == 2 && value[1] == 0) {
                        break;
                    }
                }
            }
        } catch (final InterruptedException e) {
			Log.e("SAPP", "setCharacterIndication: can't wait for the descriptor valye " + e);
		}

        Log.w("SAPP", "setCharacteristicIndication: very well!");
    }

    public List<BluetoothGattService> getSupportedGattServices() {
        if (mBluetoothGatt == null) return null;

        return mBluetoothGatt.getServices();
    }
}
