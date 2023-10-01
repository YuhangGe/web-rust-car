use esp_idf_hal::gpio::{PinDriver, Output, Gpio18};
use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

pub const TICK_MILLS: u16 = 50;
/// 转向马达的最大速度，0~255
pub const TURN_MAX_SPEED: u8 = 240;
/// 前进马达的最大速度，0~255
pub const RUN_MAX_SPEED: u8 = u8::MAX;
pub const TURN_ACCEL: u8 = 2;
pub const RUN_ACCEL: u8 = 4;
pub const TURN_MIN_SPEED: u8 = 230;
pub const RUN_MIN_SPEED: u8 = 190;

#[derive(Debug)]
struct Speed {
  positive: bool,
  value: u8,
}
impl Default for Speed {
  fn default() -> Self {
    Speed {
      positive: true,
      value: 0,
    }
  }
}
struct Motor<'a> {
  current_speed: Speed,
  target_speed: Speed,
  max_speed: u8,
  min_speed: u8,
  /// 加速度，每个 tick 增加或减少的速度的值
  acceleration: u8,
  positive_pwm_pin: LedcDriver<'a>,
  negitive_pwm_pin: LedcDriver<'a>,
}
impl<'a> Motor<'a> {
  fn new(
    max_speed: u8,
    min_speed: u8,
    accel: u8,
    positive_pwm_pin: LedcDriver<'a>,
    negitive_pwm_pin: LedcDriver<'a>,
  ) -> Self {
    Self {
      current_speed: Speed::default(),
      target_speed: Speed::default(),
      max_speed,
      min_speed,
      acceleration: accel,
      positive_pwm_pin,
      negitive_pwm_pin,
    }
  }
  /**
   * 设置马达目标速度，positive 代表是正向还是反向，speed 为 0~255 之间的数值
   */
  fn set_speed(&mut self, positive: bool, speed: u8) {
    let speed = if speed == 0 {
      0
    } else {
      let v = (self.max_speed - self.min_speed) as u32 * speed as u32 / u8::MAX as u32;
      self.min_speed + v as u8
      // speed.max(self.min_speed).min(self.max_speed)
    };
    if self.target_speed.positive == positive && self.target_speed.value == speed {
      return;
    }
    self.target_speed.positive = positive;
    self.target_speed.value = speed;
    // ::log::info!("Set {:?}", self.target_speed);
    if self.current_speed.value == 0 && speed > 0 {
      self.current_speed.positive = positive;
      self.current_speed.value = self.min_speed;
      self.adj_pin(positive, speed);
    }
  }
  fn adj_pin(&mut self, positive: bool, speed: u8) {
    let pin = if positive {
      &mut self.positive_pwm_pin
    } else {
      &mut self.negitive_pwm_pin
    };
    // ::log::info!("update pin duty {}", speed);
    if speed == 0 {
      match pin.set_duty(0) {
        Err(e) => ::log::error!("{:?}", e),
        Ok(_) => {}
      }
    } else if speed == u8::MAX {
      // ::log::info!("duty to max {}", max);
      match pin.set_duty(pin.get_max_duty()) {
        Err(e) => ::log::error!("{:?}", e),
        Ok(_) => {}
      }
    } else {
      // ::log::info!("duty to {}", max * (speed as u32) / (u16::MAX as u32));
      match pin.set_duty(pin.get_max_duty() * (speed as u32) / (u8::MAX as u32)) {
        Err(e) => ::log::error!("{:?}", e),
        Ok(_) => {}
      }
    }
  }
  fn tick(&mut self) {
    let cv = self.current_speed.value;
    let tv = self.target_speed.value;
    let cp = self.current_speed.positive;
    let tp = self.target_speed.positive;
    // ::log::info!("{} {} {} {}", cv, tv, cp, tp);
    if cv == tv && (cv == 0 || cp == tp) {
      return;
    }
    // ::log::info!("{} {} {} {}", cv, tv, cp, tp);

    // ::log::info!("{:?}, {:?}", self.current_speed, self.target_speed);

    if cp == tp {
      if cv > tv {
        if cv - tv <= self.acceleration {
          self.current_speed.value = tv;
        } else {
          let cv = cv - self.acceleration;
          if cv < self.min_speed && tv == 0 {
            self.current_speed.value = 0;
          } else {
            self.current_speed.value = cv;
          }
        }
      } else {
        if tv - cv <= self.acceleration {
          self.current_speed.value = tv;
        } else {
          self.current_speed.value = (cv + self.acceleration).max(self.min_speed);
        }
      }
    } else {
      if cv < self.acceleration {
        let new_value = self.acceleration - cv;
        self.current_speed.value = new_value.min(tv).max(self.min_speed);
        self.current_speed.positive = self.target_speed.positive;
      } else {
        let mut cv = cv - self.acceleration;
        if cv < self.min_speed {
          cv = 0;
        }
        if cv == 0 {
          self.current_speed.positive = self.target_speed.positive;
        }
        self.current_speed.value = cv;
      }
    }
    // ::log::info!("Adj {},{} {:?}", cp, cv, self.current_speed);

    if cp != self.current_speed.positive {
      // 如果方向发生了扭转，则将原来旧方向 pin 口的 pwm 调为 0
      self.adj_pin(cp, 0);
    }
    self.adj_pin(self.current_speed.positive, self.current_speed.value);
  }
}
pub struct Controller<'a> {
  turn_motor: Motor<'a>,
  run_motor: Motor<'a>,
  light_pin: PinDriver<'a, Gpio18, Output>
}

const TURN_LEFT: u8 = 1;
const TURN_RIGHT: u8 = 2;
const RUN_FRONT: u8 = 3;
const RUN_BACK: u8 = 4;
const STOP_ALL: u8 = 5;
const TOGGLE_LIGHT: u8 = 6;

impl<'a> Controller<'a> {
  pub fn new() -> Self {
    let peripherals = Peripherals::take().unwrap();
    let light_pin = PinDriver::output(peripherals.pins.gpio18).unwrap();

    let turn_left_pin = LedcDriver::new(
      peripherals.ledc.channel0,
      LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &config::TimerConfig::new().frequency(5.kHz().into()),
      )
      .unwrap(),
      peripherals.pins.gpio0,
    )
    .unwrap();
    let turn_right_pin = LedcDriver::new(
      peripherals.ledc.channel1,
      LedcTimerDriver::new(
        peripherals.ledc.timer1,
        &config::TimerConfig::new().frequency(5.kHz().into()),
      )
      .unwrap(),
      peripherals.pins.gpio1,
    )
    .unwrap();

    let run_front_pin = LedcDriver::new(
      peripherals.ledc.channel2,
      LedcTimerDriver::new(
        peripherals.ledc.timer2,
        &config::TimerConfig::new().frequency(5.kHz().into()),
      )
      .unwrap(),
      peripherals.pins.gpio2,
    )
    .unwrap();

    let run_back_pin = LedcDriver::new(
      peripherals.ledc.channel3,
      LedcTimerDriver::new(
        peripherals.ledc.timer3,
        &config::TimerConfig::new().frequency(5.kHz().into()),
      )
      .unwrap(),
      peripherals.pins.gpio3,
    )
    .unwrap();

    Self {
      light_pin,
      turn_motor: Motor::new(
        TURN_MAX_SPEED,
        TURN_MIN_SPEED,
        TURN_ACCEL,
        turn_left_pin,
        turn_right_pin,
      ),
      run_motor: Motor::new(
        RUN_MAX_SPEED,
        RUN_MIN_SPEED,
        RUN_ACCEL,
        run_front_pin,
        run_back_pin,
      ),
    }
  }
  pub fn handle(&mut self, recv_data: &[u8]) {
    if recv_data.len() < 2 {
      return;
    }
    // ::log::info!("{:?}", recv_data);
    match recv_data[0] {
      TURN_LEFT => self.turn_motor.set_speed(true, recv_data[1]),
      TURN_RIGHT => self.turn_motor.set_speed(false, recv_data[1]),
      RUN_FRONT => self.run_motor.set_speed(true, recv_data[1]),
      RUN_BACK => self.run_motor.set_speed(false, recv_data[1]),
      STOP_ALL => self.stop(),
      TOGGLE_LIGHT => self.toggle_light(recv_data[1]),
      _ => {}
    }
  }
  pub fn stop(&mut self) {
    self.turn_motor.set_speed(true, 0);
    self.run_motor.set_speed(true, 0);
  }
  pub fn tick(&mut self) {
    self.turn_motor.tick();
    self.run_motor.tick();
  }
  pub fn toggle_light(&mut self, light: u8) {
    match if light == 0 {
      self.light_pin.set_low()
    } else {
      self.light_pin.set_high()
    } {
      Ok(_) => {},
      Err(e) => ::log::error!("{:?}", e)
    }
  }
  pub fn flash(&mut self) {
    self.toggle_light(1);
    esp_idf_hal::delay::FreeRtos::delay_ms(400);
    self.toggle_light(0);
    esp_idf_hal::delay::FreeRtos::delay_ms(400);
    self.toggle_light(1);
    esp_idf_hal::delay::FreeRtos::delay_ms(400);
    self.toggle_light(0);
  }
}
