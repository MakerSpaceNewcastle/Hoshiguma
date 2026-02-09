difference() {
	union() {
		translate([0, 0, 8 / 2]) {
			cube([10, 10, 8], center = true);
		}

		for(a = [0, 90]) {
			rotate([0, 0, a]) {
				translate([0, 0, 2 / 2]) {
					cube([5, 18, 2], center = true);
				}
			}
		}
	}

	translate([0, 0, -0.1]) {
		cylinder(h = 10.2, d = 4.2, $fn = 12);
	}
}

