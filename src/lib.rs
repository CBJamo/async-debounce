#![cfg_attr(not(feature = "tokio"), no_std)]

#[cfg(all(feature = "embassy-time", feature = "tokio"))]
compile_error!("features `embassy-time` and `tokio` are mutually exclusive");

#[cfg(not(any(feature = "embassy-time", feature = "tokio")))]
compile_error!("one of `embassy-time` or `tokio` must be enabled");

#[cfg(feature = "embassy-time")]
use embassy_time::{Duration, with_timeout};

#[cfg(feature = "tokio")]
use core::time::Duration;
#[cfg(feature = "tokio")]
use tokio::time::timeout as with_timeout;

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
                if with_timeout(self.debounce_high_time, self.input.wait_for_low()).await.is_err() {
                    if self.input.is_high()? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn wait_for_low(&mut self) -> Result<(), T::Error> {
        if self.input.is_high()? {
            loop {
                self.input.wait_for_falling_edge().await?;
                if with_timeout(self.debounce_low_time, self.input.wait_for_high()).await.is_err() {
                    if self.input.is_low()? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), T::Error> {
        loop {
            self.input.wait_for_rising_edge().await?;
            if with_timeout(self.debounce_high_time, self.input.wait_for_low()).await.is_err() {
                if self.input.is_high()? {
                    break Ok(());
                }
            }
        }
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), T::Error> {
        loop {
            self.input.wait_for_falling_edge().await?;
            if with_timeout(self.debounce_low_time, self.input.wait_for_high()).await.is_err() {
                if self.input.is_low()? {
                    break Ok(());
                }
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
