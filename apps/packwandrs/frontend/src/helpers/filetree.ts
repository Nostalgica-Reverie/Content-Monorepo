import { editorFsReadDir, type EditorDirectoryEntry } from '@/helpers/invoke/editor'

/** VS Code's `FileType` bitmask, which `editor_fs_read_dir` mirrors. */
export const FILE_TYPE_DIRECTORY = 2

export interface FileNode {
	name: string
	/** Pack-relative and forward-slashed. The pack root is the empty string. */
	path: string
	directory: boolean
	expanded: boolean
	loading: boolean
	/** `null` until the directory has been read once. */
	children: FileNode[] | null
	error: string | null
}

export function toNode(entry: EditorDirectoryEntry, parent: string): FileNode {
	return {
		name: entry.name,
		path: parent ? `${parent}/${entry.name}` : entry.name,
		// Tested as a bitmask rather than compared: a symlinked directory sets both
		// the directory and the symlink bit.
		directory: (entry.fileType & FILE_TYPE_DIRECTORY) !== 0,
		expanded: false,
		loading: false,
		children: null,
		error: null,
	}
}

/** Directories first, then case-insensitive by name — the usual explorer order. */
export function ordered(nodes: FileNode[]): FileNode[] {
	return [...nodes].sort((left, right) => {
		if (left.directory !== right.directory) return left.directory ? -1 : 1
		return left.name.localeCompare(right.name, undefined, { sensitivity: 'base' })
	})
}

/** Read one directory level and return it in display order. */
export async function readLevel(packId: string, path: string): Promise<FileNode[]> {
	const entries = await editorFsReadDir(packId, path)
	return ordered(entries.map((entry) => toNode(entry, path)))
}
