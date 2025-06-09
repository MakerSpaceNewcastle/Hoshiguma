bottom_height = 43;

// An approximation of the head/lens assembly that the mount must attach to.
module LensAssembly() {
  color("gray") {
    cylinder(h = bottom_height, d = 28);

    translate([0, 0, bottom_height]) {
      cylinder(h = 50, d = 21);
    }
  }
}

module MountScrews() {
  for(x = [-15, 15]) {
    translate([x, 0, 5]) {
      rotate([90, 0, 0]) {
        children();
      }
    }
  }
}

mount_width = 42;

module MainMount() {
  color("blue") {
    difference() {
      translate([0, 0, bottom_height + 0.01]) {
        difference() {
          union() {
            translate([-mount_width/2, -3, 0]) {
              cube([mount_width, 22, 10]);
            }

            translate([0, 19, -45/2]) {
              rotate([90, 0, 0]) {
                linear_extrude(3) {
                  difference() {
                    union() {
                      polygon(points=[
                        [-mount_width/2, 45/2],
                        [mount_width/2, 45/2],
                        [10, -45/2],
                        [-10, -45/2],
                      ]);
                    }

                    for(x = [-3, 3]) {
                      translate([x, -(45/2)+3, 0]) {
                        circle(d = 1.8, $fn = 5);
                      }
                    }
                  }
                }
              }
            }
          }

          MountScrews() {
            cylinder(d = 3, h = 100, center = true, $fn = 5);
          }
        }
      }

      LensAssembly();
    }
  }
}

module MountClamp() {
  color("green") {
    difference() {
      translate([0, 0, bottom_height + 0.01]) {
        difference() {
          translate([-mount_width/2, -14, 0]) {
            cube([mount_width, 10, 10]);
          }

          MountScrews() {
            cylinder(d = 3.2, h = 100, center = true, $fn = 16);
          }
        }
      }

      LensAssembly();
    }
  }
}

#LensAssembly();
MainMount();
MountClamp();
