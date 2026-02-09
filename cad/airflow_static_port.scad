use <third-party/threads.scad>;

thread_diameter = 24;
height = 20;

// Port
difference() {
  union() {
    ScrewThread(thread_diameter, height = height, tolerance = 0.4);
    cylinder(d = 50, h = 2, $fn = 64);
  }

  // Port hole
  translate([0, 0, -0.1]) {
    cylinder(d = 4, h = 5 + 0.2, $fn = 16);
  }

  // Pipe hole
  translate([0, 0, 5 - 0.1]) {
    cylinder(d = 7.55, h = height + 0.2 - 5, $fn = 16);
  }
}

// Nut
translate([50, 0, 0]) {
  MetricNut(thread_diameter, thickness=10, tolerance=0.4);
}
