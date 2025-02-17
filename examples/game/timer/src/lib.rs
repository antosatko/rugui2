#[derive(Debug, Clone, Copy, PartialEq)]
/// A simple time struct that can be used to keep track of time.
/// 
/// This struct is useful for games and simulations where you need to keep track of time.
/// It also can be used to sleep until the next frame.
pub struct Timer {
    /// The desired frame time.
    /// 
    /// This is the time that the program should sleep until the next frame.
    /// 
    /// The default value is 1/60 seconds.
    desired_frame_time: std::time::Duration,
    /// The desired frame rate.
    desired_frame_rate: f32,
    /// Instant when the last frame started.
    last_frame: std::time::Instant,
    /// Instant when the current frame started.
    current_frame: std::time::Instant,
    /// Instant when the timer started.
    start_time: std::time::Instant,
    /// The time that has passed since the last frame in seconds.
    delta: f32,
    /// The total number of frames that have passed since the start of the program.
    frame_count: u64,
    /// The time that has passed since the start of the program in seconds.
    elapsed: f32,
}

/// The default frame rate.
const DEFAULT_FRAME_RATE: f32 = 60.0;


impl Default for Timer {
    /// Creates a new `Time` with the desired frame rate of 60 frames per second.
    fn default() -> Self {
        Self {
            last_frame: std::time::Instant::now(),
            current_frame: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
            desired_frame_time: std::time::Duration::from_secs_f32(1.0 / DEFAULT_FRAME_RATE),
            desired_frame_rate: DEFAULT_FRAME_RATE,
            delta: 0.0,
            frame_count: 0,
            elapsed: 0.0,
        }
    }
}

impl Timer {
    /// Creates a new `Time` with the desired frame rate.
    /// 
    /// # Panics
    /// 
    /// Panics if the desired frame rate is less than or equal to 0 or if it is infinite.
    pub fn new(desired_frame_rate: f32) -> Self {
        assert!(Self::check_frame_rate(desired_frame_rate), "The desired frame rate must be greater than 0 and not infinite.");
        
        Self {
            desired_frame_time: std::time::Duration::from_secs_f32(1.0 / desired_frame_rate),
            ..Default::default()
        }
    }

    #[inline]
    /// Returns the time that has passed since the last frame in seconds.
    pub fn delta(&self) -> f32 {
        self.delta
    }

    #[inline]
    /// Returns the total number of frames that have passed since the start of the program.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    #[inline]
    /// Returns the desired frame rate.
    pub fn desired_frame_rate(&self) -> f32 {
        self.desired_frame_rate
    }

    #[inline]
    /// Sets the desired frame rate.
    ///
    /// # Panics
    /// 
    /// Panics if the desired frame rate is less than or equal to 0 or if it is infinite.
    pub fn set_desired_frame_rate(&mut self, desired_frame_rate: f32) -> bool {
        if !Self::check_frame_rate(desired_frame_rate) {
            return false;
        }
        self.desired_frame_time = std::time::Duration::from_secs_f32(1.0 / desired_frame_rate);
        self.desired_frame_rate = desired_frame_rate;
        true
    }

    #[inline]
    /// Returns Instant when the last frame started.
    pub fn last_frame(&self) -> &std::time::Instant {
        &self.last_frame
    }

    #[inline]
    /// Returns Instant when the current frame started.
    pub fn current_frame(&self) -> &std::time::Instant {
        &self.current_frame
    }

    #[inline]
    /// Returns Instant when the timer started.
    pub fn start_time(&self) -> &std::time::Instant {
        &self.start_time
    }

    /// Returns the remaining time for the current tick
    pub fn remaining_time(&self) -> Option<std::time::Duration> {
        self.desired_frame_time.checked_sub(self.current_frame.elapsed())
    }

    /// Ticks the time.
    /// 
    /// This should be called at the start of the frame.
    pub fn tick(&mut self) {
        self.last_frame = self.current_frame;
        self.current_frame = std::time::Instant::now();
        self.delta = (self.current_frame - self.last_frame).as_secs_f32();
        self.frame_count += 1;
        self.elapsed = self.current_frame.duration_since(self.start_time).as_secs_f32();
    }

    #[inline]
    /// Returns the frame rate.
    pub fn fps(&self) -> f32 {
        1.0 / self.delta
    }

    #[inline]
    /// Returns the time that has passed since the start of the program in seconds.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    #[inline]
    /// Returns true if the time has passed the interval.
    /// 
    /// This method may fail if your frame rate is too high.
    pub fn interval(&self, interval: f32) -> bool {
        let cum = self.elapsed % interval;
        cum < self.delta
    }

    #[inline]
    /// Returns the number of intervals that have passed since past frame.
    /// 
    /// Use this if your interval is smaller than your frame time.
    pub fn precise_interval(&self, interval: f32) -> f32 {
        (self.delta + (self.elapsed % interval)) / interval
    }

    #[inline]
    /// Returns true if the time has passed the timeout.
    pub fn timeout(&self, timeout: f32) -> bool {
        self.elapsed > timeout && self.elapsed - self.delta <= timeout
    }

    /// Sleeps until the next frame.
    /// 
    /// Returns the time slept.
    /// 
    /// If the frame is already late, it will return `None`.
    /// 
    /// Internaly, it uses `std::thread::sleep` to sleep. If precision is important, you should add the `spin_sleep` feature.
    /// This will use the `spin_sleep` crate to sleep.
    pub fn sleep(&self) -> Option<std::time::Duration> {
        let remaining = self.desired_frame_time.checked_sub(self.current_frame.elapsed());
        match remaining {
            Some(remaining) => {
                #[cfg(feature = "spin_sleep")]
                spin_sleep::sleep(remaining);
                #[cfg(not(feature = "spin_sleep"))]
                std::thread::sleep(remaining);
                Some(remaining)
            }
            None => {
                None
            }
        }
    }

    /// Sleeps until the next frame and ticks the time.
    /// 
    /// Use this if you can guarantee that no frame initialization is needed at the start of the loop.
    /// 
    /// Returns the time slept.
    /// 
    /// If the frame is already late, it will return `None`.
    /// 
    /// Internaly, it uses `std::thread::sleep` to sleep. If precision is important, you should add the `spin_sleep` feature.
    /// This will use the `spin_sleep` crate to sleep.
    pub fn sleep_tick(&mut self) -> Option<std::time::Duration> {
        let remaining = self.desired_frame_time.checked_sub(self.current_frame.elapsed());
        match remaining {
            Some(remaining) => {
                #[cfg(feature = "spin_sleep")]
                spin_sleep::sleep(remaining);
                #[cfg(not(feature = "spin_sleep"))]
                std::thread::sleep(remaining);
                self.tick();
                Some(remaining)
            }
            None => {
                self.tick();
                None
            }
        }
    }

    #[inline]
    /// Returns true if the frame rate is valid.
    fn check_frame_rate(frame_rate: f32) -> bool {
        frame_rate > 0.0 && !frame_rate.is_infinite()
    }
}