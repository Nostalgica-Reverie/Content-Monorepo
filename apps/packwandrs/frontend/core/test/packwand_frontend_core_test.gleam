import gleeunit
import gleeunit/should
import packwand_frontend_core

pub fn main() {
  gleeunit.main()
}

pub fn theme_validation_test() {
  packwand_frontend_core.validate_theme_id("user.nether-ember") |> should.be_true
  packwand_frontend_core.validate_theme_id("../escape") |> should.be_false
  packwand_frontend_core.validate_hex_colour("#aabbcc") |> should.be_true
  packwand_frontend_core.validate_hex_colour("red") |> should.be_false
}

pub fn reducer_describes_effects_test() {
  let model = packwand_frontend_core.init("builtin.packwand-dark")
  let #(next, effects) = packwand_frontend_core.update(
    model,
    packwand_frontend_core.SelectTheme("user.custom"),
  )
  packwand_frontend_core.theme_id(next) |> should.equal("user.custom")
  effects |> should.equal([packwand_frontend_core.PersistTheme("user.custom")])
}
