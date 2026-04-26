use </home/dan/git/SCAD_Lib/rj45_panel_mount.scad>;
use <cooler_connector_blank.scad>;

difference() {
  BlankPanel();
  Rj45Connector();
}
