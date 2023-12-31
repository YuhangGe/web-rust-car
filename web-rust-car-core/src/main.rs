extern crate alloc;
mod controller;

use std::sync::{Arc, Mutex};

// use alloc::format;
use controller::{Controller, TICK_MILLS};
use esp32_nimble::{uuid128, BLEDevice, NimbleProperties};
use esp_idf_sys as _;


fn main() {
  esp_idf_sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let ble_device = BLEDevice::take();

  let server = ble_device.get_server();
  let controller = Arc::new(Mutex::new(Controller::new()));

  let controller4 = controller.clone();
  server.on_connect(move |server, desc| {
    ::log::info!("Client connected");
    // 蓝牙连上后也闪速下大灯
    match controller4.lock() {
      Ok(mut ctrl) => {
        ctrl.flash();
      }
      Err(e) => {
        ::log::error!("{:?}", e)
      }
    };
    server
      .update_conn_params(desc.conn_handle, 24, 48, 0, 60)
      .unwrap();

    // ::log::info!("Multi-connect support: start advertising");
    // ble_device.get_advertising().start().unwrap();
  });
  let controller3 = controller.clone();
  server.on_disconnect(move |_| {
    match controller3.lock() {
      Ok(mut ctrl) => ctrl.stop(),
      Err(e) => ::log::error!("{}", e),
    }
    ::log::info!("Client disconected");
  });

  let service = server.create_service(uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"));

  // A static characteristic.
  // let static_characteristic = service.lock().create_characteristic(
  //   uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
  //   NimbleProperties::READ,
  // );
  // static_characteristic
  //   .lock()
  //   .set_value("Hello, world!".as_bytes());

  // A characteristic that notifies every second.
  // let notifying_characteristic = service.lock().create_characteristic(
  //   uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
  //   NimbleProperties::READ | NimbleProperties::NOTIFY,
  // );
  // notifying_characteristic.lock().set_value(b"Initial value.");

  // A writable characteristic.
  let writable_characteristic = service.lock().create_characteristic(
    uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
    NimbleProperties::READ | NimbleProperties::WRITE,
  );

  let controller2 = controller.clone();
  writable_characteristic
    .lock()
    .on_read(move |_, _| {
      ::log::info!("Read from writable characteristic.");
    })
    .on_write(move |args| {
      match controller2.lock() {
        Ok(mut ctrl) => ctrl.handle(args.recv_data),
        Err(e) => {
          ::log::error!("{:?}", e)
        }
      }

      // ::log::info!("Wrote to writable characteristic: {:?}", args.recv_data);
    });

  let ble_advertising = ble_device.get_advertising();
  ble_advertising
    .name("Web-Rust-Car")
    .add_service_uuid(uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"));

  ble_advertising.start().unwrap();

  // let mut counter = 0;
  // loop {
  //   esp_idf_hal::delay::FreeRtos::delay_ms(1000);
  //   notifying_characteristic
  //     .lock()
  //     .set_value(format!("Counter: {counter}").as_bytes())
  //     .notify();

  //   counter += 1;
  // }

  match controller.lock() {
    Ok(mut ctrl) => {
      ctrl.flash();
    }
    Err(e) => {
      ::log::error!("{:?}", e)
    }
  }

  loop {
    esp_idf_hal::delay::FreeRtos::delay_ms(TICK_MILLS as u32);
    match controller.lock() {
      Ok(mut ctrl) => {
        ctrl.tick();
      }
      Err(e) => {
        ::log::error!("{:?}", e)
      }
    }
  }
}
