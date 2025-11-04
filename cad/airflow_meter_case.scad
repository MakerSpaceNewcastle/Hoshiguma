use <thatbox.scad>;

box_inner = [100, 60, 25];
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

      // Probe mounting holes
      translate([-20, 0, -base_thickness - 0.05]) {
        cylinder(h = base_thickness + 0.1, d = 22, $fn = 32);

        for (i = [0:90:360]) {
          rotate([0, 0, i]) {
            translate([20, 0]) {
              cylinder(h = base_thickness + 0.1, d = 3.5, $fn = 16);
            }
          }
        }
      }

      // Sensor mounting holes
      translate([30, -10, -base_thickness - 0.05]) {
        for (x = [-12, 12]) {
          translate([x, 0]) {
            cylinder(h = base_thickness + 0.1, d = 3, $fn = 5);
          }
        }
      }

      // Cable hole
      translate([(box_inner[0] / 2) - 0.05, -20, 15]) {
        rotate([0, 90, 0]) {
          cylinder(h = wall_thickness + 0.1, d = 3.5, $fn = 16);
        }
      }

      // Vent hole
      translate([(box_inner[0] / 2) - 0.05, 15, 10]) {
        rotate([0, 90, 0]) {
          cylinder(h = wall_thickness + 0.1, d = 3, $fn = 8);
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
Lid();
