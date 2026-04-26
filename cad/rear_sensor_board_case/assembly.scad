use </home/dan/git/SCAD_Lib/that-box/thatbox.scad>;
use </home/dan/git/SCAD_Lib/wiznet-w55rp20-evb-pico/w55rp20-evb-pico.scad>;
use </home/dan/git/SCAD_Lib/sensirion-sdp8xx/sdp8xx.scad>;
use </home/dan/git/SCAD_Lib/yynmos-4.scad>;

box_inner = [120, 90, 38];
wall_thickness = 2;
base_thickness = 2.5;

pico_board_position = [-40, -44];
mosfet_board_position = [16, -15];
sdp8xx_position = [10, 44];

module Box() {
  color("red") {
    difference() {
      union() {
        ThatBox_Box(
            inner = box_inner,
            wall_thickness = wall_thickness,
            base_thickness = base_thickness
        );

        // Mounting tabs
        difference() {
          hull() {
            for(x = [-70, 70]) {
              translate([x, 0, -base_thickness]) {
                cylinder(d = 30, h = base_thickness);
              }
            }
          }
          for(x = [-72, 72]) {
            translate([x, 0, -base_thickness - 0.1]) {
              cylinder(d = 4.5, h = base_thickness + 0.2, $fn = 16);
            }
          }
        }

        // Pico board mounting hole support
        translate(pico_board_position) {
          W55RP20EVBPico_add();
        }

        // MOSFET driver board mounting hole support
        translate(mosfet_board_position) {
          YYNMOS4_add();
        }
      }

      // Pico board mounting holes and RJ45 cutout
      translate(pico_board_position) {
        W55RP20EVBPico_subtract(hole_extra_depth = base_thickness + 0.1);
      }

      // Pressure sensor mounting holes and cutout
      translate(sdp8xx_position) {
        rotate([0, 0, 180]) {
          SDP8xx_subtract(hole_extra_depth = base_thickness + 0.1);
        }
      }

      // MOSFET driver board mounting holes and cutout
      translate(mosfet_board_position) {
        YYNMOS4_subtract(hole_extra_depth = base_thickness + 0.1);
      }

      // Cable access cutout
      translate([10, -45, 34]) {
        cube([10, 5, 10], center = true);
      }
    }
  }
}

module Lid() {
  color("blue", 0.5) {
    projection() {
      ThatBox_Lid(inner = box_inner);
    }
  }
}

color("grey") {
  translate(pico_board_position) {
    W55RP20EVBPico_device();
  }

  translate(mosfet_board_position) {
    YYNMOS4_device();
  }

  translate(sdp8xx_position) {
    rotate([0, 0, 180]) {
      SDP8xx_device();
    }
  }
}

Box();

translate([0, 0, 50]) {
  Lid();
}
