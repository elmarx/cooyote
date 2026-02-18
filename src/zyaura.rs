use hidapi::{HidDevice, HidError};

const VENDOR_ID: u16 = 0x04d9;
const PRODUCT_ID: u16 = 0xa052;

pub struct Sensor {
    device: HidDevice,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    HidError(#[from] HidError),
    #[error("Encrypted data")]
    Encrypted,
    #[error("Checksum mismatch")]
    ChecksumMismatch,
}

#[derive(PartialEq, Debug)]
pub struct Measurement {
    pub temperature: f32,
    pub co2: u16,
}

impl Sensor {
    pub fn new() -> Result<Self, Error> {
        let api: hidapi::HidApi = hidapi::HidApi::new()?;

        let device = api.open(VENDOR_ID, PRODUCT_ID).expect("Device not found!");

        // buffer 0 is the report id and must be set to 0x00. The remaining bytes are the key, but unused… so 0
        let buffer = vec![0; 9];
        device.send_feature_report(&buffer)?;

        Ok(Self { device })
    }

    pub fn read_item(&self) -> Result<Item, Error> {
        let mut data = [0u8; 8];
        assert_eq!(self.device.read(&mut data)?, 8, "Expected to read 8 bytes");

        Item::try_from(data)
    }

    pub fn read(&self) -> Result<Measurement, Error> {
        let mut co2: Option<u16> = None;
        let mut temp: Option<f32> = None;

        loop {
            let item = self.read_item()?;
            match item {
                Item::CO2(c) => co2 = Some(c),
                Item::Temperature(t) => temp = Some(t),
                _ => {}
            }

            if let (Some(c), Some(t)) = (co2, temp) {
                return Ok(Measurement {
                    temperature: t,
                    co2: c,
                });
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Item {
    /// a temperature reading in degrees of celcius
    Temperature(f32),
    /// a co2 concentration in parts-per-million (PPM)
    CO2(u16),
    /// relative humidity (not supported in Dorstman mini)
    RelativeHumidity(f32),
    // unknown value, with item code
    Unknown(u8),
}

impl TryFrom<[u8; 8]> for Item {
    type Error = Error;

    fn try_from(data: [u8; 8]) -> Result<Self, Self::Error> {
        if is_encrypted(data) {
            return Err(Error::Encrypted);
        }
        if !is_checksum_valid(data) {
            return Err(Error::ChecksumMismatch);
        }

        Ok(decode(data))
    }
}

/// Check if data are encrypted. Earlier versions of the TFA Dorstman mini had encryption
fn is_encrypted(data: [u8; 8]) -> bool {
    // basically check the "End of frame" is the expected value
    data[4] != 0x0d
}

/// Validate the checksum of a packet
///
fn is_checksum_valid(data_frame: [u8; 8]) -> bool {
    let sum = (data_frame[0..3].iter().map(|x| u16::from(*x)).sum::<u16>() & 0xff) as u8;
    sum == data_frame[3]
}

/// Decode reading from the sensor.
///
/// Documented here: <http://co2meters.com/Documentation/AppNotes/AN146-RAD-0401-serial-communication.pdf>
fn decode(data: [u8; 8]) -> Item {
    let op = data[0];
    let val = u16::from(data[1]) << 8 | u16::from(data[2]);

    match op {
        b'P' => Item::CO2(val),
        b'B' => Item::Temperature(f32::from(val) / 16.0 - 273.15),
        b'A' => Item::RelativeHumidity(f32::from(val) * 0.01),
        item_code => Item::Unknown(item_code),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_checksum() {
        assert!(is_checksum_valid([
            0x50, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00
        ]));
        assert!(!is_checksum_valid([
            0x51, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00
        ]));
    }

    #[test]
    fn decodes_co2() {
        assert_eq!(
            Item::CO2(1013),
            decode([0x50, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00])
        );
        assert_eq!(
            Item::Unknown(0x51),
            decode([0x51, 0x03, 0xF5, 0x48, 0x0D, 0x00, 0x00, 0x00])
        );
    }
}
