$fn = 64;

h1 = 3;
h2 = 10;

d1 = 20;
d2 = 15;

cylinder(d = d1, h = h1);

translate([0, 0, h1]) {
  cylinder(d1 = d1, d2 = d2, h = h2);
}
