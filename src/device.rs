use std::time::Duration;

pub fn open_device(path: &str) -> Result<Box<dyn serialport::SerialPort>, String> {
    let builder = serialport::new(path, 115_200)
        .timeout(Duration::from_secs(1))
        .open();

    match builder {
        Ok(device) => Ok(device),
        Err(e) => Err(format!("Cannot open serial port: {}", e))
    }
}

pub fn upload_frame(device: &mut Box<dyn serialport::SerialPort>, frame: &[u8]) -> Result<(), String> {
    if let Err(e) = device.write(&frame) {
        return Err(e.to_string());
    }

    if let Err(e) = device.flush() {
        return Err(e.to_string());
    }

    Ok(())
}
