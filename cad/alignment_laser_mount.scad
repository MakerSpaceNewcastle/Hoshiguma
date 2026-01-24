beam_tube_diameter = 24;
laser_diameter = 9;
laser_offset = 19;

linear_extrude(4) {
	difference() {
		union() {
			circle(d = beam_tube_diameter + 5);

			for (a = [0, -90]) {
				hull() {
					circle(d = laser_diameter + 3);

					rotate([0, 0, a]) {
						translate([0, -laser_offset]) {
							circle(d = laser_diameter + 3);
						}
					}
				}
			}
		}

		circle(d = beam_tube_diameter - 1);

		rotate([0, 0, -45]) {
			translate([0, 13]) {
				square([3, 5], center = true);
			}
		}

		for (a = [0, -90]) {
			rotate([0, 0, a]) {
				translate([0, -laser_offset]) {
					circle(d = laser_diameter - 0.5);

					translate([0, -5]) {
						square([2, 3], center = true);
					}
				}
			}
		}
	}
}
