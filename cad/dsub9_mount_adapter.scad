difference() {
  // Panel
  minkowski() {
    square([44, 44], center = true);
    circle(d = 10, $fn = 16);
  }

  // Connector
  square([18, 9.8], center = true);

  // Connector mounting holes
  dx = 25 / 2;
  for(x = [-dx, dx]) {
    translate([x, 0]) {
      circle(d = 3.1, $fn = 16);
    }
  }

  // Panel mounting holes
  dd = 40 / 2;
  for(d = [-dd, dd]) {
    translate([d, d]) {
      circle(d = 3.1, $fn = 16);
    }
  }
}
