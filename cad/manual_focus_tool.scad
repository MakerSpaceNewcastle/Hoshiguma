module focus_tool(d) {
	points = [
		[-15, 8],
		[5, 8],
		[5, 0],
		[0, 0],
		[0, -d],
		[-15, -d],
	];
	polygon(points, [[0, 1, 2, 3, 4, 5]]);
}

// focus_tool(37.6); // F=50.8mm
focus_tool(37.6 + 4); // F=50.8mm with alignment laser mount bracket
