use <third-party/SCAD_Lib/wago_222_mount/mount.scad>;

pcb_thickness = 1.6;

module Case() {
  board_size = [181.5, 72] + [1, 1];

  height = 8;
  tray_depth = 4;

  screw_centres = [172.5, 63];
  screw_positions = [
    [screw_centres[0] / 2, screw_centres[1] / 2],
    [-screw_centres[0] / 2, screw_centres[1] / 2],
    [screw_centres[0] / 2, -screw_centres[1] / 2],
    [-screw_centres[0] / 2, -screw_centres[1] / 2],
  ];

  difference() {
    union() {
      difference() {
        translate([0, 0, -height]) {
          linear_extrude(height) {
            square(board_size + [5, 5], center = true);
          }
        }

        translate([0, 0, 0.01 - tray_depth]) {
          linear_extrude(tray_depth) {
            square(board_size, center = true);
          }
        }
      }

      translate([0, 0, -tray_depth]) {
        for(p = screw_positions) {
          translate(p) {
            cylinder(d = 10, h = abs(tray_depth) - pcb_thickness);
          }
        }
      }

      translate([(board_size[0] + 5) / 2, -(board_size[1] + 5) / 2, -height]) {
        cube([50, board_size[1] + 5, height]);

        translate([24, (board_size[1] + 5) / 2, height]) {
          for(p = [[0, -8.5], [180, 8.5]]) {
          translate([0, p[1], 0]) {
              rotate([0, 0, p[0]]) {
                color("red") {
                  Wago222Mount_5pin();
                }
              }
            }
          }
        }
      }
    }

    translate([0, 0, -height - 1]) {
      for(p = screw_positions) {
        translate(p) {
          cylinder(d = 4, h = height + 2);
        }
      }
    }
  }
}

Case();
