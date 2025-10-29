probe_outer_diameter = 10;
probe_wall_thickness = 1;
probe_length = 120;

probe_hole_diameter = 2;
probe_hole_positions = [30, 70, 110];

flange_outer_diameter = 50;
flange_thickness = 5;

flange_mounting_hole_diameter = 3.5;
flange_mounting_hole_offset = 20;
flange_mounting_hole_count = 4;

color("blue") {
  linear_extrude(flange_thickness) {
    difference() {
      // Body
      circle(d = flange_outer_diameter, $fn = 64);

      // Place mounting holes
      for (i = [0:360 / flange_mounting_hole_count:360]) {
        rotate([0, 0, i]) {
          translate([flange_mounting_hole_offset, 0]) {
            circle(d = flange_mounting_hole_diameter, $fn = 16);
          }
        }
      }

      // Total/static vent holes
      for (x = [-3, 3]) {
        translate([x, 0]) {
          circle(d = 2, $fn = 16);
        }
      }
    }
  }
}

translate([0, 0, flange_thickness]) {
  difference() {
    union() {
      color("green") {
        linear_extrude(probe_length) {
          difference() {
            circle(d = probe_outer_diameter, $fn = 32);
            circle(d = probe_outer_diameter - (2 * probe_wall_thickness), $fn = 32);
          }

          square([probe_wall_thickness, probe_outer_diameter - 1], center = true);
        }
      }

      color("red") {
        translate([0, 0, probe_length]) {
          cylinder(h = probe_wall_thickness, d = probe_outer_diameter, $fn = 32);
        }
      }
    }

    for(z = probe_hole_positions) {
      for(a = [-probe_outer_diameter / 2, probe_outer_diameter / 2]) {
        translate([a, 0, z]) {
          rotate([0, 90, 0]) {
            cylinder(h = probe_wall_thickness + 2, d = probe_hole_diameter, center = true, $fn = 16);
          }
        }
      }
    }
  }
}
