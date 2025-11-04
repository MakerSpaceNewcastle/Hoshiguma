flange_outer_diameter = 50;
flange_thickness = 2;

flange_mounting_hole_diameter = 3.5;
flange_mounting_hole_offset = 20;
flange_mounting_hole_count = 4;

probe_height = 30;
probe_diameter = 20;
probe_inner_diameter = 5; 

hose_outer_diameter = 10;
hose_insert_length = 20;

metering_hole_z_position = 25;
metering_hole_diameter = 1;

difference() {
  union() {
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
        }
      }
    }

    translate([0, 0, flange_thickness]) {
      color("green") {
        difference() {
          cylinder(h = probe_height, d = probe_diameter, $fn = 32);

          cylinder(h = probe_height - 1, d = probe_inner_diameter, $fn = 16);

          translate([0, 0, metering_hole_z_position]) {
            rotate([0, 90, 0]) {
              cylinder(h = probe_diameter / 2, d = metering_hole_diameter, $fn = 16);
            }
          }
        }
      }
    }
  }

  translate([0, 0, -0.1]) {
    cylinder(h = hose_insert_length, d = hose_outer_diameter);
  }
}
