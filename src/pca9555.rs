use embedded_hal_async::i2c::I2c;

use crate::keymap::{EXPANDER_ADDRESSES, EXPANDER_COUNT};

const INPUT_PORT0: u8 = 0x00;
const POLARITY_PORT0: u8 = 0x04;
const CONFIGURATION_PORT0: u8 = 0x06;
const POLARITY_NONE: u16 = 0x0000;
const CONFIGURATION_ALL_INPUTS: u16 = 0xffff;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error<E> {
    Bus { address: u8, source: E },
    PolarityReadback { address: u8, actual: u16 },
    ConfigurationReadback { address: u8, actual: u16 },
}

pub struct Pca9555Bus<I> {
    i2c: I,
}

impl<I> Pca9555Bus<I>
where
    I: I2c,
{
    pub const fn new(i2c: I) -> Self {
        Self { i2c }
    }

    pub async fn configure_and_verify(&mut self) -> Result<(), Error<I::Error>> {
        let mut configured = false;
        let mut first_error = None;
        for address in EXPANDER_ADDRESSES {
            match self.configure_and_verify_address(address).await {
                Ok(()) => configured = true,
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }
        if configured {
            Ok(())
        } else {
            Err(first_error.expect("at least one expander address"))
        }
    }

    pub async fn read_inputs(&mut self) -> Result<[u16; EXPANDER_COUNT], Error<I::Error>> {
        let mut words = [u16::MAX; EXPANDER_COUNT];
        let mut any_read = false;
        let mut first_error = None;
        for (index, address) in EXPANDER_ADDRESSES.into_iter().enumerate() {
            let port_count = if address == 0x21 { 1 } else { 2 };
            for port in 0..port_count {
                match self.read_byte(address, INPUT_PORT0 + port).await {
                    Ok(byte) => {
                        words[index] &= !(0xff_u16 << (port * 8));
                        words[index] |= u16::from(byte) << (port * 8);
                        any_read = true;
                    }
                    Err(error) if first_error.is_none() => first_error = Some(error),
                    Err(_) => {}
                }
            }
        }
        if any_read {
            Ok(words)
        } else {
            Err(first_error.expect("at least one expander port"))
        }
    }

    async fn configure_and_verify_address(&mut self, address: u8) -> Result<(), Error<I::Error>> {
        self.write_pair(address, POLARITY_PORT0, POLARITY_NONE)
            .await?;
        self.write_pair(address, CONFIGURATION_PORT0, CONFIGURATION_ALL_INPUTS)
            .await?;

        let polarity = self.read_pair(address, POLARITY_PORT0).await?;
        if polarity != POLARITY_NONE {
            return Err(Error::PolarityReadback {
                address,
                actual: polarity,
            });
        }
        let configuration = self.read_pair(address, CONFIGURATION_PORT0).await?;
        if configuration != CONFIGURATION_ALL_INPUTS {
            return Err(Error::ConfigurationReadback {
                address,
                actual: configuration,
            });
        }
        Ok(())
    }

    async fn write_pair(
        &mut self,
        address: u8,
        register: u8,
        value: u16,
    ) -> Result<(), Error<I::Error>> {
        self.i2c
            .write(address, &[register, value as u8, (value >> 8) as u8])
            .await
            .map_err(|source| Error::Bus { address, source })
    }

    async fn read_pair(&mut self, address: u8, register: u8) -> Result<u16, Error<I::Error>> {
        let mut bytes = [0; 2];
        self.i2c
            .write_read(address, &[register], &mut bytes)
            .await
            .map_err(|source| Error::Bus { address, source })?;
        Ok(u16::from_le_bytes(bytes))
    }

    async fn read_byte(&mut self, address: u8, register: u8) -> Result<u8, Error<I::Error>> {
        let mut byte = [0];
        self.i2c
            .write_read(address, &[register], &mut byte)
            .await
            .map_err(|source| Error::Bus { address, source })?;
        Ok(byte[0])
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    use embedded_hal_async::i2c::{ErrorKind, ErrorType, Operation};

    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct MockError;

    impl embedded_hal_async::i2c::Error for MockError {
        fn kind(&self) -> ErrorKind {
            ErrorKind::Other
        }
    }

    struct MockI2c {
        registers: [[u16; 4]; EXPANDER_COUNT],
        ignore_polarity_write: bool,
        failed_read: Option<(u8, u8)>,
    }

    impl MockI2c {
        fn new() -> Self {
            Self {
                registers: [[0xffff, 0, 0xffff, 0]; EXPANDER_COUNT],
                ignore_polarity_write: false,
                failed_read: None,
            }
        }

        fn index(address: u8) -> usize {
            EXPANDER_ADDRESSES
                .iter()
                .position(|candidate| *candidate == address)
                .unwrap()
        }
    }

    impl ErrorType for MockI2c {
        type Error = MockError;
    }

    impl I2c for MockI2c {
        async fn transaction(
            &mut self,
            address: u8,
            operations: &mut [Operation<'_>],
        ) -> Result<(), Self::Error> {
            let device = Self::index(address);
            let mut register = 0;
            for operation in operations {
                match operation {
                    Operation::Write(bytes) if bytes.len() == 1 => register = bytes[0],
                    Operation::Write(bytes) if bytes.len() == 3 => {
                        register = bytes[0];
                        if !(self.ignore_polarity_write && register == POLARITY_PORT0) {
                            self.registers[device][register as usize / 2] =
                                u16::from_le_bytes([bytes[1], bytes[2]]);
                        }
                    }
                    Operation::Read(bytes) if bytes.len() == 2 => {
                        if self.failed_read == Some((address, register)) {
                            return Err(MockError);
                        }
                        bytes.copy_from_slice(
                            &self.registers[device][register as usize / 2].to_le_bytes(),
                        );
                    }
                    Operation::Read(bytes) if bytes.len() == 1 => {
                        if self.failed_read == Some((address, register)) {
                            return Err(MockError);
                        }
                        let word = self.registers[device][register as usize / 2].to_le_bytes();
                        bytes[0] = word[register as usize & 1];
                    }
                    _ => return Err(MockError),
                }
            }
            Ok(())
        }
    }

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn block_on<F: Future>(future: F) -> F::Output {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        let mut future = std::pin::pin!(future);
        loop {
            if let Poll::Ready(output) = future.as_mut().poll(&mut context) {
                return output;
            }
        }
    }

    #[test]
    fn configuration_is_written_and_read_back_on_all_expanders() {
        let mut bus = Pca9555Bus::new(MockI2c::new());
        block_on(bus.configure_and_verify()).unwrap();
        for registers in bus.i2c.registers {
            assert_eq!(registers[POLARITY_PORT0 as usize / 2], POLARITY_NONE);
            assert_eq!(
                registers[CONFIGURATION_PORT0 as usize / 2],
                CONFIGURATION_ALL_INPUTS
            );
        }
    }

    #[test]
    fn configuration_fails_closed_when_readback_disagrees() {
        let mut i2c = MockI2c::new();
        i2c.ignore_polarity_write = true;
        i2c.registers[0][POLARITY_PORT0 as usize / 2] = 0xffff;
        let mut bus = Pca9555Bus::new(i2c);
        assert_eq!(
            block_on(bus.configure_and_verify()),
            Err(Error::PolarityReadback {
                address: 0x20,
                actual: 0xffff
            })
        );
    }

    #[test]
    fn input_reads_keep_expander_order() {
        let mut i2c = MockI2c::new();
        for (index, registers) in i2c.registers.iter_mut().enumerate() {
            registers[0] = 0x1111 * (index as u16 + 1);
        }
        let mut bus = Pca9555Bus::new(i2c);
        let words = block_on(bus.read_inputs()).unwrap();
        for (index, word) in words.into_iter().enumerate() {
            let mut expected = 0x1111 * (index as u16 + 1);
            if EXPANDER_ADDRESSES[index] == 0x21 {
                expected |= 0xff00;
            }
            assert_eq!(word, expected);
        }
    }

    #[cfg(feature = "layout-kr")]
    #[test]
    fn input_reads_isolate_failed_ports() {
        let mut partial_config = MockI2c::new();
        partial_config.failed_read = Some((EXPANDER_ADDRESSES[3], POLARITY_PORT0));
        let mut bus = Pca9555Bus::new(partial_config);
        block_on(bus.configure_and_verify()).unwrap();

        let mut i2c = MockI2c::new();
        for (index, registers) in i2c.registers.iter_mut().enumerate() {
            registers[0] = 0x1111 * (index as u16 + 1);
        }
        i2c.failed_read = Some((EXPANDER_ADDRESSES[1], INPUT_PORT0 + 1));
        let mut bus = Pca9555Bus::new(i2c);

        let words = block_on(bus.read_inputs()).unwrap();

        assert_eq!(words[0], 0x1111);
        assert_eq!(words[1], 0xff22);
        assert_eq!(words[2], 0x3333);
        assert_eq!(words[3] & 0x00ff, 0x44);
        assert_eq!(words[3] & 0xff00, 0xff00);
    }
}
