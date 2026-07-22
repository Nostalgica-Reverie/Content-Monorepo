import type { TreeEntry } from '../types'
import { call } from './core'

export const editorTree = (id: string) => call<TreeEntry[]>('editor_tree', { id })
export const fileRead = (id: string, path: string) => call<string>('editor_file_read', { id, path })
export const fileWrite = (id: string, path: string, content: string) => call<void>('editor_file_write', { id, path, content })
export const editorCreate = (id: string, path: string, directory: boolean) => call<void>('editor_create', { id, path, directory })

export interface EditorFileStat { fileType: number; size: number; ctime: number; mtime: number }
export interface EditorDirectoryEntry { name: string; fileType: number }

export const editorFsStat = (id: string, path: string) => call<EditorFileStat>('editor_fs_stat', { id, path })
export const editorFsReadDir = (id: string, path: string) => call<EditorDirectoryEntry[]>('editor_fs_read_dir', { id, path })
export const editorFsReadFile = (id: string, path: string) => call<number[]>('editor_fs_read_file', { id, path })
export const editorFsWriteFile = (id: string, path: string, content: number[], create: boolean, overwrite: boolean) => call<void>('editor_fs_write_file', { id, path, content, create, overwrite })
export const editorFsCreateDir = (id: string, path: string) => call<void>('editor_fs_create_dir', { id, path })
export const editorFsDelete = (id: string, path: string, recursive: boolean) => call<void>('editor_fs_delete', { id, path, recursive })
export const editorFsRename = (id: string, from: string, to: string, overwrite: boolean) => call<void>('editor_fs_rename', { id, from, to, overwrite })
