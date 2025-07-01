# chuck

An unnecessarily complicated robot to chuck balls for my dog.

## Description
A small robot built with 3D-printed parts, a NEMA 17 stepper motor, and an ESP32C3 which throws tennis balls.

<img src="https://github.com/scottdalgliesh/chuck_models/blob/main/images/chuck%20assembly.PNG?raw=true" alt="CAD model" width=50% />

## Project Structure
Project is split into two crates:
* `chuck_core` contains hardware-agnostic business logic, primarily related to driving the stepper motor.  
* `chuck` contains the hardware-specific binary which runs chuck.   

This structure is used to facilitate testing of the core logic on the host PC without requiring physical access to the MCU. It also conveniently allows use of std within tests.  

## To Do
* add hardware list
* add electrical schematic

## License
[CC-BY](https://creativecommons.org/licenses/by-sa/4.0/)

## Reference 
 - [CAD files](https://github.com/scottdalgliesh/chuck_models)
 - [ESP32-C3 Datasheet](https://www.espressif.com/sites/default/files/documentation/esp32-c3_datasheet_en.pdf)
 - [XIAO-ESP32C3 dev board pinout](https://wiki.seeedstudio.com/XIAO_ESP32C3_Getting_Started/#pinout-diagram)
