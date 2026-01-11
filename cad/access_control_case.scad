use <thatbox.scad>;

box_inner = [120, 65, 25];
wall_thickness = 3;
base_thickness = 2;

module Box() {
  color("red") {
    difference() {
      ThatBox_Box(
        inner = box_inner,
        wall_thickness = wall_thickness,
        base_thickness = base_thickness
      );

      // Cable hole
      translate([0, -(box_inner[1] / 2) + 0.05, box_inner[2] / 2]) {
        rotate([90, 0, 0]) {
          cylinder(h = wall_thickness + 0.1, d = 12, $fn = 16);
        }
      }

      // Moutning holes
      for(x = [-50, 50]) {
        translate([x, -(box_inner[1] / 2) + 0.05, box_inner[2] / 2]) {
          rotate([90, 0, 0]) {
            cylinder(h = wall_thickness + 0.1, d = 4.5, $fn = 16);
          }
        }
      }
    }
  }
}

module Lid() {
  translate([0, 0, box_inner[2] + 5]) {
    color("blue", 0.5) {
      projection() {
        difference() {
          ThatBox_Lid(inner = box_inner);

          // Reader mounting holes
          for(x = [-30, 30]) {
            for(y = [-25, 25]) {
              translate([x, y, -0.01]) {
                cylinder(h = 3, d = 3.5, $fn = 16);
              }
            }
          }
        }
      }
    }
  }
}

Box();
Lid();
