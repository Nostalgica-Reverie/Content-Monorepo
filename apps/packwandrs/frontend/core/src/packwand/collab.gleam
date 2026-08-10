//// Live-session lifecycle.
////
//// Rust owns sockets and authentication; this reducer owns the states the UI
//// may present while those operations are in flight. Keeping the transition
//// table here prevents a failed join or a late disconnect event from leaving
//// Pinia with a role and connection state that contradict each other.

pub type Role {
  NoRole
  Host
  Guest
}

pub type Connection {
  Disconnected
  Hosting
  Joining
  Connected
}

pub type Model {
  Model(role: Role, connection: Connection)
}

pub type Message {
  HostStarted
  JoinStarted
  ConnectionEstablished
  ConnectionLost
}

pub fn init() -> Model {
  Model(role: NoRole, connection: Disconnected)
}

/// A stable role name for TypeScript and Vue.
///
/// Gleam variants compile to ES classes whose constructor names are mangled by
/// Vite. Literal keys remain stable in production builds.
pub fn role_key(role: Role) -> String {
  case role {
    NoRole -> "none"
    Host -> "host"
    Guest -> "guest"
  }
}

/// A stable connection name for TypeScript and Vue.
pub fn connection_key(connection: Connection) -> String {
  case connection {
    Disconnected -> "disconnected"
    Hosting -> "hosting"
    Joining -> "joining"
    Connected -> "connected"
  }
}

pub fn update(model: Model, message: Message) -> Model {
  case message {
    HostStarted ->
      case model.connection {
        Disconnected -> Model(role: Host, connection: Hosting)
        _ -> model
      }

    JoinStarted ->
      case model.connection {
        Disconnected -> Model(role: Guest, connection: Joining)
        _ -> model
      }

    ConnectionEstablished ->
      case model.connection {
        Hosting | Joining -> Model(..model, connection: Connected)
        _ -> model
      }

    ConnectionLost -> init()
  }
}
