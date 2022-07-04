//% IMPORTS
    
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.BroadcastReceiver;
import android.content.ComponentName;
import android.content.ServiceConnection;
import android.bluetooth.BluetoothAdapter;
import android.bluetooth.BluetoothGattCharacteristic;
import android.os.IBinder;
import android.widget.TextView;
import android.util.Log;
import android.Manifest;
import android.content.pm.PackageManager;

import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import java.util.List;
import java.util.Timer;
import java.util.TimerTask;

import quadbt.BluetoothLeService;
import quadbt.QuadBT;

//% END

//% MAIN_ACTIVITY_BODY

public static BluetoothLeService bluetoothService;

// service can be binded only after receiving permission
// BUT pause can arrive before that
// this flag is a workaround to not unregister some part of the service(or whatever should be unregistered)
// in onPause
private boolean serviceBinded;

private static IntentFilter makeGattUpdateIntentFilter() {
    final IntentFilter intentFilter = new IntentFilter();
    intentFilter.addAction(BluetoothLeService.ACTION_GATT_CONNECTED);
    intentFilter.addAction(BluetoothLeService.ACTION_GATT_DISCONNECTED);
    intentFilter.addAction(BluetoothLeService.ACTION_GATT_SERVICES_DISCOVERED);
    intentFilter.addAction(BluetoothLeService.ACTION_DATA_AVAILABLE);
    return intentFilter;
}

private boolean checkPermissions() {
    return ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED;
}

private void bindBluetoothService() {
    BluetoothAdapter bluetoothAdapter = BluetoothAdapter.getDefaultAdapter();
    if (!bluetoothAdapter.isEnabled()) {
        Intent enableBtIntent = new Intent(BluetoothAdapter.ACTION_REQUEST_ENABLE);
        startActivityForResult(enableBtIntent, 2);
    } 
    registerReceiver(gattUpdateReceiver, makeGattUpdateIntentFilter());
    
    Intent gattServiceIntent = new Intent(this, BluetoothLeService.class);
    bindService(gattServiceIntent, serviceConnection, Context.BIND_AUTO_CREATE);
}

@Override
public void onRequestPermissionsResult(
        int requestCode,
        String permissions[],
        int[] grantResults) {
    if (requestCode == 1) {
        if (grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
            bindBluetoothService();
        }
    }
}

// @Override
// protected void onActivityResult(int requestCode, int resultCode, Intent data) {
//      if (requestCode == 2) {
//          // bluetooth enabled
//      }
//  }

private void requestPermissions() {
    ActivityCompat.requestPermissions(this, new String[]{
            Manifest.permission.ACCESS_FINE_LOCATION}, 1);
}

private ServiceConnection serviceConnection = new ServiceConnection() {
        @Override
        public void onServiceConnected(ComponentName name, IBinder service) {
            Log.w("SAPP", "Bluetooth service connected");
            
            bluetoothService = ((BluetoothLeService.LocalBinder)service).getService();
            if (bluetoothService == null) {
                Log.e("SAPP", "bluetoothService == null");
            }
            if (!bluetoothService.initialize()) {
                Log.e("SAPP", "Bluetooth service cant be initialized");
            }
            
            if (bluetoothService != null) {
                QuadBT.connectService(bluetoothService);
            }
        }
        
        @Override
        public void onServiceDisconnected(ComponentName name) {
            bluetoothService = null;
        }
    };

private final BroadcastReceiver gattUpdateReceiver = new BroadcastReceiver() {
        @Override
        public void onReceive(Context context, Intent intent) {
            final String action = intent.getAction();
            if (BluetoothLeService.ACTION_GATT_CONNECTED.equals(action)) {
                QuadBT.onGattConnected();
            } else if (BluetoothLeService.ACTION_GATT_DISCONNECTED.equals(action)) {
                QuadBT.onGattDisconnected();
            } else if (BluetoothLeService.ACTION_GATT_SERVICES_DISCOVERED.equals(action)) {
                QuadBT.servicesDiscovered(bluetoothService.getSupportedGattServices());
            } else if (BluetoothLeService.ACTION_DATA_AVAILABLE.equals(action)) {
                byte[] data = intent.getByteArrayExtra(BluetoothLeService.EXTRA_DATA);
                QuadBT.onDataAvailable(data);
            }
        }
    };

//% END


//% MAIN_ACTIVITY_ON_RESUME

if (!checkPermissions()) {
    requestPermissions();
} else {
    bindBluetoothService();
}

//% END

//% MAIN_ACTIVITY_ON_PAUSE

if (serviceBinded) {
    unregisterReceiver(gattUpdateReceiver);
}

//% END

//% MAIN_ACTIVITY_ON_CREATE

//% END
