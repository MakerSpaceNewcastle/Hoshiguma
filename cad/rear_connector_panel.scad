use </home/dan/git/SCAD_Lib/rj45_panel_mount.scad>;

module IecC14() {
  // Connector
  square([27, 19.3], center = true);

  // Connector mounting holes
  dx = 40 / 2;
  for(x = [-dx, dx]) {
    translate([x, 0]) {
      circle(d = 3.1, $fn = 16);
    }
  }
}

module DeathSocket() {
  square([37, 35], center = true);
}

difference() {
  // Panel
  minkowski() {
    square([160 - 10, 240 - 10], center = true);
    circle(d = 10, $fn = 16);
  }

  // Panel mounting holes
  centres = [140, 220];

  for(x = [-centres[0] / 2, centres[0] / 2]) {
    for(y = [-centres[1] / 2, centres[1] / 2]) {
      translate([x, y]) {
        circle(d = 4, $fn = 16);
      }
    }
  }

  // PC USB socket
  translate([0, 80]) {
    Rj45Connector();
  }

  // Network sockets
  for(x = [-30, 30]) {
    for(y = [-10, 30]) {
      translate([x, 10 + y]) {
        Rj45Connector();
      }
    }
  }

  // Mains in socket
  translate([-35, -70]) {
    IecC14();
  }

  // Fume extraction fan outlet
  translate([30, -70]) {
    DeathSocket();
  }
}
