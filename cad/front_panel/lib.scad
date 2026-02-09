machine_panel_mounting_hole_size = 5.5;
machine_panel_mounting_hole_centres = [200, 253];

split_point_y = 10;
split_overlap = 10;

panel_size = machine_panel_mounting_hole_centres + [10, 10];
panel_thickness = 6;

panel_lip_size = [185, 240];
panel_lip_thickness = 2.5;

module place_at_centres(centres) {
	dx = centres[0] / 2;
	dy = centres[1] / 2;

	for(x = [-dx, dx]) {
		for(y = [-dy, dy]) {
			translate([x, y]) {
				children();
			}
		}
	}
}

module through_panel() {
	panel_depth = panel_lip_thickness + panel_thickness;

	translate([0, 0, -panel_depth - 0.1]) {
		linear_extrude(panel_depth + 0.2) {
			children();
		}
	}
}

module solid_panel() {
	// Top lip (to make the surface flush with the top of the metal body of the machine)
	translate([0, 0, -panel_lip_thickness / 2]) {
		cube([panel_lip_size[0], panel_lip_size[1], panel_lip_thickness], center = true);
	}

	// Main section of the panel
	translate([0, 0, -panel_lip_thickness - (panel_thickness / 2)]) {
		cube([panel_size[0], panel_size[1], panel_thickness], center = true);
	}
}

module full_panel() {
	difference() {
		solid_panel();
		
		// Panel mounting holes
		dx = machine_panel_mounting_hole_centres[0] / 2;
		dy = machine_panel_mounting_hole_centres[1] / 2;
		for(x = [-dx, dx]) {
			for(y = [-dy, split_point_y, dy]) {
				translate([x, y]) {
					through_panel() {
						circle(d = machine_panel_mounting_hole_size, $fn = 16);
					}
				}
			}
		}

		// Ruida HMI
		translate([0, -55]) {
			// Panel cutout
			through_panel() {
				square([147.5, 97], center = true);
			}

			// Guide/retention holes for panel mount bracket screws
			translate([-15, 0, -panel_thickness - panel_lip_thickness - 0.1]) {
				place_at_centres([68, 109]) {
					cylinder(h = 3, d = 6, $fn = 24);
				}
			}
		}

		// Emergency stop button
		translate([-2, 80]) {
			// Cutout
			through_panel() {
				circle(d = 23);
			}

			// Recess for retention nut
			translate([0, 0, -panel_thickness - panel_lip_thickness - 0.1]) {
				cylinder(h = panel_thickness + panel_lip_thickness - 2, d = 45);
			}
		}

		// Access controller
		translate([58, 80, -panel_thickness - panel_lip_thickness - 0.1]) {
			// Cutout
			d = panel_thickness + panel_lip_thickness - 1;
			translate([0, 0, d / 2]) {
				cube([64, 54, d], center = true);
				cube([43, 75, d], center = true);
			}
		
			// Mounting holes
			place_at_centres([50, 60]) {
				cylinder(h = panel_thickness + panel_lip_thickness - 1, d = 4.2, $fn = 12);
			}
		}

		// Machine HMI
		translate([-62, 72]) {
			// Display cutout
			through_panel() {
				square([45, 59], center = true);
			}

			// Mounting holes
			translate([0, -5, -panel_thickness - panel_lip_thickness - 0.1]) {
				d = panel_thickness + panel_lip_thickness - 0.5;
				translate([0, 0, d / 2]) {
					cube([60, 88, d], center = true);
				}
			
				place_at_centres([44.45, 93.98]) {
					cylinder(h = panel_thickness + panel_lip_thickness - 1, d = 4.2, $fn = 12);
				}
			}
		}
	}
}

module panel_split_lower() {
	y0 = split_point_y + (split_overlap / 2);
	y1 = split_point_y - (split_overlap / 2);
	y2 = -(panel_size[1] / 2) - 0.1;

	z0 = 0.1;
	z1 = -panel_lip_thickness - (panel_thickness / 2) + 0.4;
	z2 = -panel_lip_thickness - panel_thickness - 0.1;

	rotate([90, 0, 90]) {
		translate([0, 0, -(panel_size[0] / 2) - 0.1]) {
			linear_extrude(panel_size[0] + 0.2) {
				polygon(points = [
					[y0, z0],
					[y0, z1],
					[y1, z1],
					[y1, z2],
					[y2, z2],
					[y2, z0],
				]);
			}
		}
	}
}

module panel_split_upper() {
	y0 = split_point_y - (split_overlap / 2);
	y1 = split_point_y + (split_overlap / 2);
	y2 = (panel_size[1] / 2) - 0.1;

	z0 = 0.1;
	z1 = -panel_lip_thickness - (panel_thickness / 2);
	z2 = -panel_lip_thickness - panel_thickness - 0.1;

	rotate([90, 0, 90]) {
		translate([0, 0, -(panel_size[0] / 2) - 0.1]) {
			linear_extrude(panel_size[0] + 0.2) {
				polygon(points = [
					[y1, z0],
					[y2, z0],
					[y2, z2],
					[y0, z2],
					[y0, z1],
					[y1, z1],
				]);
			}
		}
	}
}

module panel_lower() {
	intersection() {
		full_panel();
		panel_split_lower();
	}
}

module panel_upper() {
	intersection() {
		full_panel();
		panel_split_upper();
	}
}

// full_panel();
color("red") {
	panel_lower();
}
color("green") {
	panel_upper();
}
