import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/string

pub type Document {
  Document(path: String, dirty: Bool)
}

pub type Model {
  Model(
    theme_id: String,
    documents: List(Document),
    active_document: Option(String),
  )
}

pub type Message {
  SelectTheme(String)
  RequestDocument(String)
  DocumentOpened(String)
  DocumentChanged(String)
  DocumentSaved(String)
  DocumentClosed(String)
}

pub type Effect {
  PersistTheme(String)
  LoadDocument(String)
}

pub fn init(theme_id: String) -> Model {
  Model(theme_id: theme_id, documents: [], active_document: None)
}

pub fn update(model: Model, message: Message) -> #(Model, List(Effect)) {
  case model, message {
    Model(_, documents, active), SelectTheme(id) ->
      #(Model(id, documents, active), [PersistTheme(id)])
    Model(theme, documents, _), RequestDocument(path) ->
      case has_document(documents, path) {
        True -> #(Model(theme, documents, Some(path)), [])
        False -> #(model, [LoadDocument(path)])
      }
    Model(theme, documents, _), DocumentOpened(path) ->
      #(Model(theme, [Document(path, False), ..documents], Some(path)), [])
    Model(theme, documents, active), DocumentChanged(path) ->
      #(Model(theme, set_dirty(documents, path, True), active), [])
    Model(theme, documents, active), DocumentSaved(path) ->
      #(Model(theme, set_dirty(documents, path, False), active), [])
    Model(theme, documents, active), DocumentClosed(path) -> {
      let remaining = list.filter(documents, fn(document) { document_path(document) != path })
      let next = case active == Some(path) {
        True -> remaining |> list.first |> option_map(document_path)
        False -> active
      }
      #(Model(theme, remaining, next), [])
    }
  }
}

pub fn theme_id(model: Model) -> String {
  let Model(id, _, _) = model
  id
}

pub fn document_path(document: Document) -> String {
  let Document(path, _) = document
  path
}

pub fn document_dirty(document: Document) -> Bool {
  let Document(_, dirty) = document
  dirty
}

pub fn validate_theme_id(value: String) -> Bool {
  let valid_prefix = string.starts_with(value, "user.") || string.starts_with(value, "builtin.")
  let tail = value |> string.split(".") |> list.drop(1) |> string.join(".")
  valid_prefix && tail != "" && string.to_graphemes(tail) |> list.all(valid_slug_character)
}

pub fn validate_hex_colour(value: String) -> Bool {
  let graphemes = string.to_graphemes(value)
  let length = list.length(graphemes)
  let body = list.drop(graphemes, 1)
  list.first(graphemes) == Ok("#")
  && { length == 7 || length == 9 }
  && list.all(body, valid_hex_character)
}

fn has_document(documents: List(Document), path: String) -> Bool {
  list.any(documents, fn(document) { document_path(document) == path })
}

fn set_dirty(documents: List(Document), path: String, dirty: Bool) -> List(Document) {
  list.map(documents, fn(document) {
    case document {
      Document(current, _) if current == path -> Document(current, dirty)
      other -> other
    }
  })
}

fn option_map(value: Result(Document, Nil), mapper: fn(Document) -> String) -> Option(String) {
  case value {
    Ok(document) -> Some(mapper(document))
    Error(_) -> None
  }
}

fn valid_slug_character(value: String) -> Bool {
  string.contains("abcdefghijklmnopqrstuvwxyz0123456789.-", value)
}

fn valid_hex_character(value: String) -> Bool {
  string.contains("0123456789abcdefABCDEF", value)
}
