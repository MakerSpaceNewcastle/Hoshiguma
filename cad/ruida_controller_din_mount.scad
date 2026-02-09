use <third-party/SCAD_Lib/din_rail/clip.scad>

height = 95;
width = 12;
thickness = 8;
screw_centres = 80;

difference() {
  translate([0, width / 2, -thickness]) {
    rotate([90, 0, 0]) {
      linear_extrude(width) {
        union() {
          DinRailClip();

          translate([-height / 2, 0]) {
            square([height, thickness]);
          }
        }
      }
    }
  }

  for(x = [-screw_centres / 2, screw_centres / 2]) {
    translate([x, 0, -thickness - 0.1]) {
      cylinder(h = thickness + 0.2, d = 4.2); // For M3 threaded brass insert
    }
  }
}
