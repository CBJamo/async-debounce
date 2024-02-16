#![no_std]

use embassy_time::{Duration, Timer};
use embedded_hal::digital::{ErrorType, InputPin};
use embedded_hal_async::digital::Wait;

pub struct Debouncer<T> {
    input: T,
    debounce_high_time: Duration,
    debounce_low_time: Duration,
}

impl<T> Debouncer<T> {
    pub fn new(input: T, debounce_time: Duration) -> Self {
        Self {
            input,
            debounce_high_time: debounce_time,
            debounce_low_time: debounce_time,
        }
    }

    pub fn new_asymmetric(
        input: T,
        debounce_high_time: Duration,
        debounce_low_time: Duration,
    ) -> Self {
        Self {
            input,
            debounce_high_time,
            debounce_low_time,
        }
    }
}

impl<T: InputPin> ErrorType for Debouncer<T> {
    type Error = T::Error;
}

impl<T: Wait + InputPin> Wait for Debouncer<T> {
    async fn wait_for_high(&mut self) -> Result<(), T::Error> {
        if self.input.is_low()? {
            loop {
                self.input.wait_for_rising_edge().await?;

                Timer::after(self.debounce_high_time).await;

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

                Timer::after(self.debounce_low_time).await;

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

            Timer::after(self.debounce_high_time).await;

            if self.input.is_high()? {
                break Ok(());
            }
        }
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), T::Error> {
        loop {
            self.input.wait_for_falling_edge().await?;

            Timer::after(self.debounce_low_time).await;

            if self.input.is_low()? {
                break Ok(());
            }
        }
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), T::Error> {
        if self.input.is_low()? {
            self.wait_for_rising_edge().await
        } else {
            self.wait_for_falling_edge().await
        }
    }
}

impl<T: InputPin> InputPin for Debouncer<T> {
    #[inline]
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.input.is_high()
    }

    #[inline]
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.input.is_low()
    }
}
