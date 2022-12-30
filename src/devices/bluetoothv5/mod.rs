use std::{time::Duration, cell::RefCell, rc::Rc};
use bluest::{Adapter, AdvertisingDevice, Uuid, Characteristic, Service};
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio_stream::StreamExt;

/// The BLE GATT Service that V5 Brains provide
const GATT_SERVICE: &str = "08590f7e-db05-467e-8757-72f6faeb13d5";

/// The unknown GATT characteristic
const GATT_UNKNOWN: &str = "08590f7e-db05-467e-8757-72f6faeb1306";

/// The user port GATT characteristic
const GATT_USER: &str = "08590f7e-db05-467e-8757-72f6faeb1316";

/// The system port GATT characteristic
const GATT_SYSTEM: &str = "08590f7e-db05-467e-8757-72f6faeb13e5";

/// A serial port implemented over a GATT characteristic
pub struct BluetoothSerial {
    inner: Rc<RefCell<BluetoothInner>>,
    characteristic: Characteristic
}

impl AsyncRead for BluetoothSerial {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let fut = self.characteristic.read();
        tokio::pin!(fut);
        match std::future::Future::poll(fut, cx) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(v) => {
                match v {
                    Ok(d) => {
                        buf.put_slice(&d);
                        std::task::Poll::Ready(Ok(()))
                    },
                    Err(e) => {
                        std::task::Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, crate::errors::DeviceError::InvalidDevice)))
                    } 
                }
            }
        }
    }
}

/// Contains all data that needs to be shared between bluetooth serial devices.
#[derive(Clone, Debug)]
pub struct BluetoothInner {
    pub adapter: Option<Adapter>,
    pub advertising: AdvertisingDevice,
}

/// Represents a brain connected over bluetooth
#[derive(Clone, Debug)]
pub struct BluetoothBrain{
    inner: Rc<RefCell<BluetoothInner>>
}

impl BluetoothBrain {
    pub async fn new(advertising: AdvertisingDevice) -> Result<Self, crate::errors::DeviceError> {
        
        Ok(Self {
            inner: Rc::new(RefCell::new(BluetoothInner {
                adapter: None,
                advertising,
            }))
        })
    }

    /// Connects self to the brain
    pub async fn connect(&mut self) -> Result<(), crate::errors::DeviceError> {

        // Get the adapter and wait for it to be available
        let adapter = Adapter::default().await.ok_or(crate::errors::DeviceError::NoBluetoothAdapter)?;
        adapter.wait_available().await?;

        let mut inner = self.inner.borrow_mut();

        inner.adapter = Some(adapter);

        inner.adapter.as_ref().ok_or(crate::errors::DeviceError::NotConnected)?.connect_device(&inner.advertising.device).await?;

        
        Ok(())
    }

    /// Handshakes to the brain, telling it we have connected
    pub async fn handshake(&self) -> Result<(), crate::errors::DeviceError> {

        let mut system = self.get_system().await?;

        // The system characteristic should return 0xDEADFACE. If not, the device is bad
        let mut data = [0u8; 4];
        system.read(&mut data).await?;

        println!("{data:?}");

        Ok(())
    }

    pub async fn get_system(&self) -> Result<BluetoothSerial, crate::errors::DeviceError> {
        let inner = self.inner.borrow();

        let service = inner.advertising.device.discover_services_with_uuid(GATT_SERVICE.try_into().unwrap()).await?.get(0).ok_or(crate::errors::DeviceError::InvalidDevice)?.clone();

        Ok(BluetoothSerial {
            inner: self.inner.clone(),
            characteristic: service
                .discover_characteristics_with_uuid(GATT_SYSTEM.try_into().unwrap()).await?
                .get(0).ok_or(crate::errors::DeviceError::InvalidDevice)?.clone()
        })
    }

    pub async fn get_user(&self) -> Result<BluetoothSerial, crate::errors::DeviceError> {
        let inner = self.inner.borrow();

        let service = inner.advertising.device.discover_services_with_uuid(GATT_SERVICE.try_into().unwrap()).await?.get(0).ok_or(crate::errors::DeviceError::InvalidDevice)?.clone();

        Ok(BluetoothSerial {
            inner: self.inner.clone(),
            characteristic: service
                .discover_characteristics_with_uuid(GATT_USER.try_into().unwrap()).await?
                .get(0).ok_or(crate::errors::DeviceError::InvalidDevice)?.clone()
        })
    }

    /// Disconnects self from the brain
    pub async fn disconnect(&self) -> Result<(), crate::errors::DeviceError> {

        let inner = self.inner.borrow();

        inner.adapter.as_ref().ok_or(crate::errors::DeviceError::NotConnected)?.disconnect_device(&inner.advertising.device).await?;

        Ok(())
    }
}

/// Discovers all V5 devices that are advertising over bluetooth.
/// By default it scans for 5 seconds, but this can be configured
pub async fn scan_for_v5_devices(timeout: Option<Duration>) -> Result<Vec<BluetoothBrain>, crate::errors::DeviceError> {

    // If timeout is None, then default to five seconds
    let timeout = timeout.unwrap_or_else(|| Duration::new(5, 0));

    // Get the adapter and wait for it to be available
    let adapter = Adapter::default().await.ok_or(crate::errors::DeviceError::NoBluetoothAdapter)?;
    adapter.wait_available().await?;

    // Create the GATT UUID
    let service: bluest::Uuid = GATT_SERVICE.try_into().unwrap();
    let service = &[service];

    // Start scanning
    let scan_stream = adapter.scan(service).await?;

    // Set a timeout
    let timeout_stream = scan_stream.timeout(timeout);
    tokio::pin!(timeout_stream);

    // Find the current time
    let time = std::time::SystemTime::now();

    let mut devices = Vec::<BluetoothBrain>::new();

    // Find each device
    while let Ok(Some(discovered_device)) = timeout_stream.try_next().await {
        devices.push(BluetoothBrain::new(discovered_device).await?);
        // If over timeout has passed, then break
        if time.elapsed().unwrap() >= timeout {
            break;
        }
    }

    // These are our brains
    Ok(devices)
}