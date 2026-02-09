linear_extrude(3) {
  difference() {
    square([30, 85], center = true);

    translate([0, (85 / 2) - 6]) {
      circle(d = 3, $fn = 5);
      translate([0, -30]) {
        circle(d = 3, $fn = 5);
      }
    }
  }
}
