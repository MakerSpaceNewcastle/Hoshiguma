linear_extrude(3) {
  difference() {
    square([85, 52], center = true);

    x = 70 / 2;
    y = 35 / 2;

    for (p = [[x, -y], [-x, y]]) {
      translate(p) {
        circle(d = 4, $fn = 5);
      }
    }
  }
}
