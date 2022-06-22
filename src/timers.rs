use std::sync::{Mutex, Arc};
use timer::{Timer, Guard};
use chrono::Duration;

pub struct Timers {
    pub delay_timer: Arc<Mutex<u8>>,
    pub sound_timer: Arc<Mutex<u8>>,
    _timer: Timer,
    _guard: Guard,
}
impl Timers {
    pub fn new() -> Self {
        let delay_timer = Arc::new(Mutex::new(0));
        let sound_timer = Arc::new(Mutex::new(0));

        let _timer = timer::Timer::new();
        let _guard = {
            let delay_timer = Arc::clone(&delay_timer);
            let sound_timer = Arc::clone(&sound_timer);

            _timer.schedule_repeating(Duration::nanoseconds(16666667), move || {
                let mut delay_timer = delay_timer.lock().unwrap();
                if *delay_timer > 0 {
                    *delay_timer -= 1;
                }

                let mut sound_timer = sound_timer.lock().unwrap();
                if *sound_timer > 0 {
                    *sound_timer -= 1;
                }
            })
        };
        
        Self {
            delay_timer,
            sound_timer,
            _timer,
            _guard,
        }
    }
}