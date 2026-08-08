import gleeunit/should
import packwand/instancing

pub fn version_label_test() {
  instancing.version_label("fabric", "1.21.1")
  |> should.equal("Fabric 1.21.1")
  instancing.version_label("vanilla", "1.20.4")
  |> should.equal("Vanilla 1.20.4")
}

pub fn inherited_placeholder_test() {
  instancing.inherited_placeholder("", "4096")
  |> should.equal("4096 (inherited)")
  instancing.inherited_placeholder("8192", "4096")
  |> should.equal("8192")
}
