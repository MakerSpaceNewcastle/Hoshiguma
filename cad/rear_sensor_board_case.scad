use <thatbox.scad>;

// https://docs.wiznet.io/Product/Chip/MCU/W55RP20/w55rp20-evb-pico#dimension-unit--mm

box_inner = [100, 60, 38];
wall_thickness = 2;
base_thickness = 2;

module Box() {
  color("red") {
    difference() {
      ThatBox_Box(
        inner = box_inner,
        wall_thickness = wall_thickness,
        base_thickness = base_thickness
      );

      // Pressure sensor mounting holes
      translate([30, -10, -base_thickness - 0.05]) {
        for (x = [-12, 12]) {
          translate([x, 0]) {
            cylinder(h = base_thickness + 0.1, d = 3, $fn = 5);
          }
        }
      }
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
//Lid();
