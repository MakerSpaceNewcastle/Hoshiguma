width = 55;

difference() {
  // Body
  linear_extrude(40) {
    square([width, 20], center = true);
  }

  // Pump body cutout
  translate([0, 0, 42]) {
    rotate([90, 0, 0]) {
      cylinder(d = 45, h = 22, center = true);
    }
  }

  // Pump retention cable tie hole
  translate([0, 0, 18]) {
    cube([width + 1, 5, 2], center = true);
  }

  // Case mounting holes
  dx = 42 / 2;
  for (x = [-dx, dx]) {
    translate([x, 0, -0.1]) {
      cylinder(d = 3, h = 18, $fn = 5);
    }
  }
}
