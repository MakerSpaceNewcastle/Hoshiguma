width = 55;
height = 35;

module PlaceAtCentres(cx) {
  dx = cx / 2;
  for (x = [-dx, dx]) {
    translate([x, 0]) {
      children();
    }
  }
}

module Main() {
  difference() {
    // Body
    linear_extrude(height) {
      square([width, 20], center = true);
    }

    // Pump body cutout
    translate([0, 0, height - (17.5 / 2) + 0.01]) {
      rotate([90, 0, 0]) {
        cube([17.5, 17.5, 22], center = true);
      }
    }

    // Case mounting holes
    translate([0, 0, -0.1]) {
      PlaceAtCentres(cx = 42) {
        cylinder(d = 3, h = 18, $fn = 5);
      }
    }

    // Clamp mounting holes
    translate([0, 0, height - 18 + 0.01]) {
      PlaceAtCentres(cx = 36) {
        cylinder(d = 3, h = 18, $fn = 5);
      }
    }
  }
}

module Clamp() {
  difference() {
    // Body
    linear_extrude(3) {
      square([width, 20], center = true);
    }

    // Clamp mounting holes
    translate([0, 0, 3 / 2]) {
      PlaceAtCentres(cx = 36) {
        cylinder(d = 3.2, h = 3.02, center = true, $fn = 24);
      }
    }
  }
}

Main();

translate([0, 0, 40]) {
  Clamp();
}
