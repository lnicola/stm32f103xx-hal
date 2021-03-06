//! Inter-Integrated Circuit (I2C) bus

use stm32f103xx::{I2C1, I2C2};

use gpio::gpioa::{PA10, PA9};
use gpio::gpiob::{PB10, PB11, PB6, PB7, PB8, PB9};
use gpio::{Alternate, PushPull};
use hal::blocking::i2c::{Write, WriteRead};
use rcc::{APB1, Clocks};
use time::Hertz;

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// Bus error
    Bus,
    /// Arbitration loss
    Arbitration,
    // Overrun, // slave mode only
    // Pec, // SMBUS mode only
    // Timeout, // SMBUS mode only
    // Alert, // SMBUS mode only
    #[doc(hidden)] _Extensible,
}

// FIXME these should be "closed" traits
/// SCL pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SclPin<I2C> {}

/// SDA pin -- DO NOT IMPLEMENT THIS TRAIT
pub unsafe trait SdaPin<I2C> {}

unsafe impl SclPin<I2C1> for PB6<Alternate<PushPull>> {}
unsafe impl SclPin<I2C1> for PB8<Alternate<PushPull>> {}

unsafe impl SclPin<I2C2> for PB10<Alternate<PushPull>> {}

unsafe impl SdaPin<I2C1> for PB7<Alternate<PushPull>> {}
unsafe impl SdaPin<I2C1> for PB9<Alternate<PushPull>> {}

unsafe impl SdaPin<I2C2> for PB11<Alternate<PushPull>> {}

/// I2C peripheral operating in master mode
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

macro_rules! busy_wait {
    ($i2c:expr, $flag:ident) => {
        loop {
            let sr1 = $i2c.sr1.read();

            if sr1.berr().bit_is_set() {
                return Err(Error::Bus);
            } else if sr1.arlo().bit_is_set() {
                return Err(Error::Arbitration);
            } else if sr1.$flag().bit_is_set() {
                break;
            } else {
                // try again
            }
        }
    }
}

macro_rules! hal {
    ($($I2CX:ident: ($i2cX:ident, $i2cXen:ident, $i2cXrst:ident),)+) => {
        $(
            impl<SCL, SDA> I2c<$I2CX, (SCL, SDA)> {
                /// Configures the I2C peripheral to work in master mode
                pub fn $i2cX<F>(
                    i2c: $I2CX,
                    pins: (SCL, SDA),
                    freq: F,
                    clocks: Clocks,
                    apb1: &mut APB1,
                ) -> Self where
                    F: Into<Hertz>,
                    SCL: SclPin<$I2CX>,
                    SDA: SdaPin<$I2CX>,
                {
                    apb1.enr().modify(|_, w| w.$i2cXen().enabled());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().set_bit());
                    apb1.rstr().modify(|_, w| w.$i2cXrst().clear_bit());

                    // let freq = freq.into().0;

                    // assert!(freq <= 1_000_000);

                    // // TODO review compliance with the timing requirements of I2C
                    // // t_I2CCLK = 1 / PCLK1
                    // // t_PRESC  = (PRESC + 1) * t_I2CCLK
                    // // t_SCLL   = (SCLL + 1) * t_PRESC
                    // // t_SCLH   = (SCLH + 1) * t_PRESC
                    // //
                    // // t_SYNC1 + t_SYNC2 > 4 * t_I2CCLK
                    // // t_SCL ~= t_SYNC1 + t_SYNC2 + t_SCLL + t_SCLH
                    // let i2cclk = clocks.pclk1().0;
                    // let ratio = i2cclk / freq - 4;
                    // let (presc, scll, sclh, sdadel, scldel) = if freq >= 100_000 {
                    //     // fast-mode or fast-mode plus
                    //     // here we pick SCLL + 1 = 2 * (SCLH + 1)
                    //     let presc = ratio / 387;

                    //     let sclh = ((ratio / (presc + 1)) - 3) / 3;
                    //     let scll = 2 * (sclh + 1) - 1;

                    //     let (sdadel, scldel) = if freq > 400_000 {
                    //         // fast-mode plus
                    //         let sdadel = 0;
                    //         let scldel = i2cclk / 4_000_000 / (presc + 1) - 1;

                    //         (sdadel, scldel)
                    //     } else {
                    //         // fast-mode
                    //         let sdadel = i2cclk / 8_000_000 / (presc + 1);
                    //         let scldel = i2cclk / 2_000_000 / (presc + 1) - 1;

                    //         (sdadel, scldel)
                    //     };

                    //     (presc, scll, sclh, sdadel, scldel)
                    // } else {
                    //     // standard-mode
                    //     // here we pick SCLL = SCLH
                    //     let presc = ratio / 514;

                    //     let sclh = ((ratio / (presc + 1)) - 2) / 2;
                    //     let scll = sclh;

                    //     let sdadel = i2cclk / 2_000_000 / (presc + 1);
                    //     let scldel = i2cclk / 800_000 / (presc + 1) - 1;

                    //     (presc, scll, sclh, sdadel, scldel)
                    // };

                    // let presc = u8(presc).unwrap();
                    // assert!(presc < 16);
                    // let scldel = u8(scldel).unwrap();
                    // assert!(scldel < 16);
                    // let sdadel = u8(sdadel).unwrap();
                    // assert!(sdadel < 16);
                    // let sclh = u8(sclh).unwrap();
                    // let scll = u8(scll).unwrap();

                    // // Configure for "fast mode" (400 KHz)
                    // i2c.timingr.write(|w| unsafe {
                    //     w.presc()
                    //         .bits(presc)
                    //         .scll()
                    //         .bits(scll)
                    //         .sclh()
                    //         .bits(sclh)
                    //         .sdadel()
                    //         .bits(sdadel)
                    //         .scldel()
                    //         .bits(scldel)
                    // });

                    // Enable the peripheral
                    i2c.cr1.write(|w| w.pe().set_bit());

                    I2c { i2c, pins }
                }

                /// Releases the I2C peripheral and associated pins
                pub fn free(self) -> ($I2CX, (SCL, SDA)) {
                    (self.i2c, self.pins)
                }
            }

            impl<PINS> Write for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Error> {
                    // START, wait for SB
                    self.i2c.cr1.write(|w| w.start().set_bit());
                    busy_wait!(self.i2c, sb);

                    // slave address, wait for ADDR
                    self.i2c.dr.write(|w| unsafe { w.dr().bits(addr << 1) });
                    busy_wait!(self.i2c, addr);

                    // clear ADDR
                    self.i2c.sr2.read();

                    for byte in bytes {
                        // Wait until we are allowed to send data (START has been ACKed or last byte
                        // went through)
                        busy_wait!(self.i2c, tx_e);

                        // put byte on the wire
                        self.i2c.dr.write(|w| unsafe { w.dr().bits(*byte) });
                    }

                    // wait for transmission to finish
                    busy_wait!(self.i2c, btf);

                    // STOP
                    self.i2c.cr1.write(|w| w.stop().set_bit());

                    Ok(())
                }
            }

            impl<PINS> WriteRead for I2c<$I2CX, PINS> {
                type Error = Error;

                fn write_read(
                    &mut self,
                    addr: u8,
                    bytes: &[u8],
                    buffer: &mut [u8],
                ) -> Result<(), Error> {
                    // TODO do we have to explicitly wait here if the bus is busy (e.g. another
                    // master is communicating)?

                    // START, wait for SB
                    self.i2c.cr1.write(|w| w.start().set_bit());
                    busy_wait!(self.i2c, sb);

                    // slave address, wait for ADDR
                    self.i2c.dr.write(|w| unsafe { w.dr().bits(addr << 1) });
                    busy_wait!(self.i2c, addr);

                    // clear ADDR
                    self.i2c.sr2.read();

                    for byte in bytes {
                        // Wait until we are allowed to send data (START has been ACKed or last byte
                        // when through)
                        busy_wait!(self.i2c, tx_e);

                        // put byte on the wire
                        self.i2c.dr.write(|w| unsafe { w.dr().bits(*byte) });
                    }

                    // wait for transmission to finish
                    busy_wait!(self.i2c, btf);

                    // STOP
                    self.i2c.cr1.write(|w| w.stop().set_bit());

                    self.i2c.cr1.write(|w| w.ack().set_bit());

                    // START, wait for SB
                    self.i2c.cr1.write(|w| w.start().set_bit());
                    busy_wait!(self.i2c, sb);

                    // slave address, wait for ADDR
                    self.i2c.dr.write(|w| unsafe { w.dr().bits((addr << 1) | 1) });
                    busy_wait!(self.i2c, addr);

                    match buffer.len() {
                        1 => {
                            self.i2c.cr1.write(|w| w.ack().clear_bit());
                        }
                        2 => {
                            self.i2c.cr1.write(|w| w.ack().clear_bit());
                            self.i2c.cr1.write(|w| w.pos().set_bit());
                        }
                        _ => {}
                    }

                    // clear ADDR
                    self.i2c.sr1.read();
                    self.i2c.sr2.read();

                    if buffer.len() > 3 {
                        for byte in &mut buffer[3..] {
                            busy_wait!(self.i2c, rx_ne);

                            *byte = self.i2c.dr.read().dr().bits();
                        }
                    }

                    match buffer.len() {
                        1 => {
                            busy_wait!(self.i2c, rx_ne);

                            // STOP
                            self.i2c.cr1.write(|w| w.stop().set_bit());

                            buffer[0] = self.i2c.dr.read().dr().bits();
                        }
                        2 => {
                            busy_wait!(self.i2c, rx_ne);

                            busy_wait!(self.i2c, btf);

                            // STOP
                            self.i2c.cr1.write(|w| w.stop().set_bit());

                            buffer[0] = self.i2c.dr.read().dr().bits();
                            buffer[1] = self.i2c.dr.read().dr().bits();
                        }
                        3 => {
                            busy_wait!(self.i2c, rx_ne);

                            busy_wait!(self.i2c, btf);

                            self.i2c.cr1.write(|w| w.ack().clear_bit());

                            busy_wait!(self.i2c, rx_ne);

                            busy_wait!(self.i2c, btf);

                            // STOP
                            self.i2c.cr1.write(|w| w.stop().set_bit());

                            buffer[0] = self.i2c.dr.read().dr().bits();
                            buffer[1] = self.i2c.dr.read().dr().bits();
                        }
                        _ => {}
                    }

                    Ok(())
                }
            }
        )+
    }
}

hal! {
    I2C1: (i2c1, i2c1en, i2c1rst),
    I2C2: (i2c2, i2c2en, i2c2rst),
}
