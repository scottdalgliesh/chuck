# chuck

An unnecessarily complicated robot to chuck balls for my dog.

## Description
A small robot built with 3D-printed parts, a NEMA 17 stepper motor, and an ESP32C3 which throws tennis balls.

## Project Structure
Project is split into two crates:
* `chuck_core` contains hardware-agnostic business logic, primarily related to driving the stepper motor.  
* `chuck` contains the hardware-specific binary which runs chuck.   

This structure is used to facilitate testing of the core logic on the host PC without requiring physical access to the MCU. It also conveniently allows use of std within tests.  

## To Do
* add image or GIF
* add STEP files for the 3D printed components
* add hardware list
* add electrical schematic

## Reference Links
 - [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
 - [XIAO-ESP32C3 dev board pinout](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/#pinout-diagram)
