import gleeunit/should
import packwand/collab

pub fn a_host_moves_from_idle_through_hosting_to_connected_test() {
  let hosting = collab.init() |> collab.update(collab.HostStarted)
  hosting.role |> should.equal(collab.Host)
  hosting.connection |> should.equal(collab.Hosting)

  hosting
  |> collab.update(collab.ConnectionEstablished)
  |> fn(model) { model.connection }
  |> should.equal(collab.Connected)
}

pub fn a_guest_moves_from_idle_through_joining_to_connected_test() {
  let joining = collab.init() |> collab.update(collab.JoinStarted)
  joining.role |> should.equal(collab.Guest)
  joining.connection |> should.equal(collab.Joining)

  joining
  |> collab.update(collab.ConnectionEstablished)
  |> fn(model) { model.connection }
  |> should.equal(collab.Connected)
}

pub fn disconnect_resets_every_active_state_test() {
  let host = collab.init() |> collab.update(collab.HostStarted)
  let guest = collab.init() |> collab.update(collab.JoinStarted)

  host |> collab.update(collab.ConnectionLost) |> should.equal(collab.init())
  guest
  |> collab.update(collab.ConnectionEstablished)
  |> collab.update(collab.ConnectionLost)
  |> should.equal(collab.init())
}

pub fn an_active_session_cannot_change_roles_test() {
  let hosting = collab.init() |> collab.update(collab.HostStarted)
  hosting |> collab.update(collab.JoinStarted) |> should.equal(hosting)
}

/// Guards the production-minification boundary used by the Vue store.
pub fn role_and_connection_keys_are_stable_test() {
  collab.role_key(collab.NoRole) |> should.equal("none")
  collab.role_key(collab.Host) |> should.equal("host")
  collab.role_key(collab.Guest) |> should.equal("guest")
  collab.connection_key(collab.Disconnected) |> should.equal("disconnected")
  collab.connection_key(collab.Hosting) |> should.equal("hosting")
  collab.connection_key(collab.Joining) |> should.equal("joining")
  collab.connection_key(collab.Connected) |> should.equal("connected")
}
