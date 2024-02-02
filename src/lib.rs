#![no_std]

use embedded_hal::digital::{InputPin, PinState, ErrorType};
use embedded_hal_async::digital::Wait;
use embassy_time::{Duration, Timer};

pub struct Debouncer<T: Wait + InputPin> {
    input: T,
    debounce_time: Duration,
}

impl<T: Wait + InputPin> Debouncer<T> {
    pub fn new(input: T, debounce_time: Duration) -> Self {
        Self { input, debounce_time }
    }
}

impl<T: Wait + InputPin> ErrorType for Debouncer<T> {
    type Error = T::Error;
}

impl<T: Wait + InputPin> Wait for Debouncer<T> {
    async fn wait_for_high(&mut self) -> Result<(), T::Error> {
        if self.input.is_low()? {
            loop {
                self.input.wait_for_rising_edge().await?;

                Timer::after(self.debounce_time).await;

                if self.input.is_high()? {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), T::Error> {
        if self.input.is_high()? {
            loop {
                self.input.wait_for_falling_edge().await?;

                Timer::after(self.debounce_time).await;

                if self.input.is_low()? {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), T::Error> {
        loop {
            self.input.wait_for_rising_edge().await?;

            Timer::after(self.debounce_time).await;

            if self.input.is_high()? {
                break Ok(());
            }
        }
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), T::Error> {
        loop {
            self.input.wait_for_falling_edge().await?;

            Timer::after(self.debounce_time).await;

            if self.input.is_low()? {
                break Ok(());
            }
        }
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), T::Error> {
        loop {
            let l1: PinState = self.input.is_high()?.into();

            self.input.wait_for_any_edge().await?;

            Timer::after(self.debounce_time).await;

            let l2: PinState = self.input.is_high()?.into();
            if l1 != l2 {
                break Ok(());
            }
        }
    }

}
