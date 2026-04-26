use <cooler_connector_blank.scad>;

module Rj45Connector() {
  // Connector
  square([15.9, 13.2], center = true);

  // Connector mounting holes
  dx = 28 / 2;
  for(x = [-dx, dx]) {
    translate([x, 1.5]) {
      circle(d = 3.1, $fn = 16);
    }
  }
}

difference() {
  BlankPanel();
  Rj45Connector();
}
