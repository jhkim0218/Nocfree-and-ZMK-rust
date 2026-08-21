use embedded_hal_async::i2c::I2c;

use crate::scanner::EXPANDER_ADDRESSES;

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
        for address in EXPANDER_ADDRESSES {
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
        }
        Ok(())
    }

    pub async fn read_inputs(&mut self) -> Result<[u16; 3], Error<I::Error>> {
        let mut words = [0; 3];
        for (index, address) in EXPANDER_ADDRESSES.into_iter().enumerate() {
            words[index] = self.read_pair(address, INPUT_PORT0).await?;
        }
        Ok(words)
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
        registers: [[u16; 4]; 3],
        ignore_polarity_write: bool,
    }

    impl MockI2c {
        fn new() -> Self {
            Self {
                registers: [[0xffff, 0, 0xffff, 0]; 3],
                ignore_polarity_write: false,
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
                        bytes.copy_from_slice(
                            &self.registers[device][register as usize / 2].to_le_bytes(),
                        );
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
        i2c.registers[0][0] = 0x1111;
        i2c.registers[1][0] = 0x2222;
        i2c.registers[2][0] = 0x3333;
        let mut bus = Pca9555Bus::new(i2c);
        assert_eq!(
            block_on(bus.read_inputs()).unwrap(),
            [0x1111, 0x2222, 0x3333]
        );
    }
}
