use </home/dan/git/SCAD_Lib/that-box/thatbox.scad>;
use </home/dan/git/SCAD_Lib/wiznet-w55rp20-evb-pico/w55rp20-evb-pico.scad>;
use </home/dan/git/SCAD_Lib/sensirion-sdp8xx/sdp8xx.scad>;

box_inner = [100, 90, 38];
wall_thickness = 2;
base_thickness = 2;

pico_board_position = [-30, -44, 0];
sdp8xx_position = [49, 20, 0];

module Box() {
  color("red", 1.0) {
    difference() {
      union() {
        ThatBox_Box(
            inner = box_inner,
            wall_thickness = wall_thickness,
            base_thickness = base_thickness
        );

        // Pico board mounting hole support
        translate(pico_board_position) {
          W55RP20EVBPico_add();
        }

        // MOSFET driver board mounting hole support
        // TODO
      }

      // Pico board mounting holes and RJ45 cutout
      translate(pico_board_position) {
        W55RP20EVBPico_subtract(hole_extra_depth = base_thickness + 0.1);
      }

      // Pressure sensor mounting holes and cutout
      translate(sdp8xx_position) {
        rotate([0, 0, 90]) {
          SDP8xx_subtract(hole_extra_depth = base_thickness + 0.1);
        }
      }

      // OneWire header cutout
      // TODO

      // MOSFET driver board mounting holes and cutout
      // TODO
    }
  }
}

module Lid() {
  translate([0, 0, 30]) {
    color("blue", 0.5) {
      projection() {
        ThatBox_Lid(inner = box_inner);
      }
    }
  }
}

Box();
color("grey") {
  translate(pico_board_position) {
    W55RP20EVBPico_device();
  }
  translate(sdp8xx_position) {
    rotate([0, 0, 90]) {
      SDP8xx_device();
    }
  }
}
//Lid();
