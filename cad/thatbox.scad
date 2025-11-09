module ThatBox_Layout(dims, corner_radius) {
  dx = (dims[0] / 2) - corner_radius;
  dy = (dims[1] / 2) - corner_radius;

  for(x = [-dx, dx]) {
    for(y = [-dy, dy]) {
      translate([x, y]) {
        children();
      }
    }
  }
}

module ThatBox_Plate(dims, thickness, corner_radius) {
  hull() {
    ThatBox_Layout(dims, corner_radius) {
      cylinder(h = thickness, r = corner_radius, $fn = 24);
    }
  }
}

function PlateSize(inner, wall_thickness) = [inner[0] + (wall_thickness * 2), inner[1] + (wall_thickness * 2)];

module ThatBox_Box(
  inner = [120, 80, 30],
  wall_thickness = 2, 
  corner_radius = 4,
  mounting_hole_diameter = 3,
  mounting_hole_depth = 20,
  base_thickness = 3, 
) {
  plate_size = PlateSize(inner, wall_thickness);

  difference() {
    // Outer volume
    translate([0, 0, -base_thickness]) {
      ThatBox_Plate(dims = plate_size, thickness = inner[2] + base_thickness, corner_radius = corner_radius);
    }

    // Inner volume
    translate([0, 0, (inner[2] / 2) + 0.01]) {
      cube(inner, center = true);
    }
  }

  difference() {
    // Mounting hole supports
    ThatBox_Layout(dims = plate_size, corner_radius = corner_radius) {
      cylinder(h = inner[2], r = corner_radius, $fn = 16);
    }

    // Mounting holes
    ThatBox_Layout(dims = plate_size, corner_radius = corner_radius) {
      translate([0, 0, inner[2] - mounting_hole_depth + 0.01]) {
        cylinder(h = mounting_hole_depth, d = mounting_hole_diameter, $fn = 5);
      }
    }
  }
}

module ThatBox_Lid(
  inner = [120, 80, 30],
  wall_thickness = 2, 
  corner_radius = 4,
  mounting_hole_diameter = 3.1,
  lid_thickness = 2,
) {
  plate_size = PlateSize(inner, wall_thickness);

  difference() {
    // Panel
    ThatBox_Plate(dims = plate_size, thickness = lid_thickness, corner_radius = corner_radius);

    // Mounting holes
    ThatBox_Layout(dims = plate_size, corner_radius = corner_radius) {
      translate([0, 0, -1]) {
        cylinder(h = lid_thickness + 2, d = mounting_hole_diameter, $fn = 16);
      }
    }
  }
}

// Demo
color("red") {
  ThatBox_Box();
}
translate([0, 0, 30]) {
  color("blue", 0.5) {
    ThatBox_Lid();
  }
}
