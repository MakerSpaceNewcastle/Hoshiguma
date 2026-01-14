machine_panel_mounting_hole_size = 5;
machine_panel_mounting_hole_centres = [200, 253];

panel_size = machine_panel_mounting_hole_centres + [10, 10];
panel_edge_radius = 4;

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

module panel_main_section() {
	o = panel_edge_radius * 2;

	minkowski() {
		square([panel_size[0] - o, panel_size[1] - o], center = true);
		circle(r = panel_edge_radius, $fn = 32);
	}
}

module peek_o_display() {
	// Display cutout
	square([45, 59], center = true);

	// Mounting holes
	translate([0, -5]) { // TODO
		place_at_centres([44.45, 93.98]) {
			circle(d = 3.2, $fn = 16);
		}
	}
}

module full_panel() {
	difference() {
		panel_main_section();

		// Panel mounting holes
		place_at_centres(machine_panel_mounting_hole_centres) {
			circle(d = machine_panel_mounting_hole_size, $fn = 16);
		}

		// Ruida HMI
		translate([0, -52]) {
			square([147, 96 + 5], center = true);
		}

		// Emergency stop button
		translate([60, 70]) {
			circle(d = 22);
		}

		// Access controller mounting holes
		translate([0, 70]) {
			square([8, 8], center = true); // For graphics alignment in LightBurn, don't actually cut this

			place_at_centres([50, 60]) {
				circle(d = 3.2, $fn = 16);
			}
		}

		// Machine HMI
		translate([-62, 70]) {
			peek_o_display();
		}
	}
}

full_panel();
