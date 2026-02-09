width = 55;
height = 40;
thickness = 25;
pipe_height = 28;

module PlaceAtCentres(cx) {
  dx = cx / 2;
  for (x = [-dx, dx]) {
    translate([x, 0]) {
      children();
    }
  }
}

module Full() {
  difference() {
    // Body
    linear_extrude(height) {
      square([width, thickness], center = true);
    }

    // Case mounting holes (for M3 brass insert)
    translate([0, 0, -0.1]) {
      PlaceAtCentres(cx = 42) {
        cylinder(d = 4.5, h = 20, $fn = 16);
      }
    }

    // Pipe cutouts
    translate([0, 0, pipe_height]) {
      PlaceAtCentres(cx = 25) {
        rotate([90, 0, 0]) {
            cylinder(d = 16.5, h = thickness + 2, center = true);
        }
      }
    }
  }
}

module Base() {
  color("blue") {
    intersection() {
      difference() {
        Full();

        // Clamp mounting hole (for M3 brass insert)
        translate([0, 0, pipe_height - 20 + 0.01]) {
          cylinder(d = 4.5, h = 20, $fn = 16);
        }
      }

      translate([-width / 2, -thickness / 2, 0]) {
        cube([width, thickness, pipe_height]);
      }
    }
  }
}

module Clamp() {
  color("green") {
    intersection() {
      difference() {
        Full();

        // Mounting clearance hole
        translate([0, 0, pipe_height - 0.1]) {
          cylinder(d = 3.2, h = pipe_height + 0.2, $fn = 24);
        }
      }

      translate([-width / 2, -thickness / 2, pipe_height]) {
        cube([width, thickness, height - pipe_height]);
      }
    }
  }
}

Base();
translate([0, 0, 5]) Clamp();
