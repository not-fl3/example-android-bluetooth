package quadbt;

import android.os.Handler;
import android.bluetooth.BluetoothManager;
import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothProfile;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattCallback;
import android.bluetooth.BluetoothGattService;
import android.bluetooth.BluetoothDevice;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.le.BluetoothLeScanner;
import android.bluetooth.le.ScanCallback;
import android.bluetooth.le.ScanResult;
import android.util.Log;
import java.util.List;

import TARGET_PACKAGE_NAME.MainActivity;

public class QuadBT  {
    public static BluetoothAdapter bluetoothAdapter;
    public static BluetoothLeScanner bluetoothLeScanner;
    private static BluetoothLeService bluetoothService;

    native static void onServiceConnected();
    public native static void onGattConnected();
    public native static void onGattDisconnected();
    native void onDeviceFound(BluetoothDevice device);
    native static void onCharacteristicDiscovered(BluetoothGattCharacteristic characteristic);
    public native static void onDataAvailable(byte[] data);

    private ScanCallback leScanCallback =
        new ScanCallback() {
            @Override
            public void onScanResult(int callbackType, ScanResult result) {
                super.onScanResult(callbackType, result);

                BluetoothDevice device = result.getDevice();
                onDeviceFound(device);
            }
        };

    public static void connectService(BluetoothLeService service) {
        bluetoothService = service;
        onServiceConnected();
    }

    public static void servicesDiscovered(List<BluetoothGattService> services) {
        for (BluetoothGattService gattService : services) {
            List<BluetoothGattCharacteristic> characteristics = gattService.getCharacteristics();
            for (BluetoothGattCharacteristic characteristic : characteristics) {
                onCharacteristicDiscovered(characteristic);
            }
        }
    }

    public QuadBT() {
    }

    public boolean isEnabled() {
        return this.bluetoothAdapter.isEnabled();
    }

    public void startScan() {
        bluetoothLeScanner = this.bluetoothAdapter.getBluetoothLeScanner();
        bluetoothLeScanner.startScan(leScanCallback);
    }

    public void connect(String address) {
        bluetoothService.connect(address);
    }

    public void disconnect() {
        bluetoothService.disconnect();
    }

    public void setCharacteristicNotification(BluetoothGattCharacteristic characteristic, boolean enabled) {
        bluetoothService.setCharacteristicNotification(characteristic, enabled);
    }

    public void setCharacteristicIndication(BluetoothGattCharacteristic characteristic, boolean enabled) {
        bluetoothService.setCharacteristicIndication(characteristic, enabled);
    }

    public void writeCharacteristicString(BluetoothGattCharacteristic characteristic, String data) {
        assert characteristic != null;
        assert data != null;

        bluetoothService.writeCharacteristic(characteristic, data);
    }

    public void writeCharacteristicBytes(BluetoothGattCharacteristic characteristic, byte[] data, boolean verify) {
        assert characteristic != null;
        assert data != null;

        bluetoothService.writeCharacteristic(characteristic, data, verify);
    }
}
