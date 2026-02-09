base_thickness = 12;
base_width = 20;
base_height = 40;

probe_width = 6;
probe_l1 = 100;
probe_l2 = 50;

probe_inner_width = 2.5;

difference() {
  // Main body
  union() {
    translate([-base_width, -base_height / 2, 0]) {
      cube([base_width, base_height, base_thickness]);
    }
    translate([0, -probe_width / 2, 0]) {
      cube([probe_l1, probe_width, probe_width]);
    }
    translate([probe_l1 - probe_width, -probe_width / 2, 0]) {
      cube([probe_width, probe_width, probe_l2]);
    }
  }

  // Pipe hole
  translate([-8, 0, base_thickness / 2]) {
    rotate([0, -90, 0]) {
      cylinder(d = 6.5, h = 20, $fn = 12);
    }
  }

  // Base airflow path
  hull() {
    translate([0, 0, probe_width / 2]) {
      rotate([0, 90, 0]) {
        linear_extrude(1) {
          square([probe_inner_width, probe_inner_width], center = true);
        }
      }
    }
    translate([-8.1, 0, base_thickness / 2]) {
      rotate([0, 90, 0]) {
        linear_extrude(1) {
          circle(d = 5.5, $fn = 24);
        }
      }
    }
  }

  // Probe airflow path
  translate([-0.1, -probe_inner_width / 2, (probe_width - probe_inner_width) / 2]) {
    cube([probe_l1 + 0.1 - (probe_width - probe_inner_width) / 2, probe_inner_width, probe_inner_width]);
  }
  translate([probe_l1 - (probe_width / 2) - (probe_inner_width / 2), -probe_inner_width / 2, (probe_width - probe_inner_width) / 2]) {
    cube([probe_inner_width, probe_inner_width, probe_l2 + 0.1 - (probe_width - probe_inner_width)]);
  }

  // Metering metering hole
  translate([probe_l1 - (probe_width / 2), 0, probe_l2 - (probe_width / 2)]) {
    rotate([90, 0, 0]) {
      cylinder(d = 1.5, h = (probe_width / 2) + 0.1, $fn = 16);
    }
  }

  // Mounting holes (for M3 brass inserts)
  for(y = [-15, 15]) {
    translate([-base_width / 2, y, -0.1]) {
      cylinder(d = 4.5, h = base_height + 0.2, $fn = 7);
    }
  }

  // translate([-50, 0, -1]) cube([150, 50, 80]);
}
