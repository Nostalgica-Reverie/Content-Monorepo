import { AnyThreadChannel, Message, ThreadChannel } from 'discord.js'

/**
 * Attempts to join a thread, ignoring errors and no-op if already joined.
 */
export async function tryJoinThread(thread: ThreadChannel | AnyThreadChannel): Promise<void> {
	try {
		// Avoid joining if already in the thread (best-effort using available flags)
		const joined = (thread as ThreadChannel).joined ?? false
		// Some environments expose `joinable`; if present and false, skip
		const joinable: boolean | undefined = (thread as any).joinable
		if (joined) return
		if (joinable === false) return
		await thread.join()
	} catch {
		// Silently ignore permission/api errors; thread events may still flow if already joined
	}
}

/** Returns true if the message is in a thread under the #community-support forum. */
export function isInCommunitySupportThread(
	message: Message,
): message is Message & { channel: ThreadChannel } {
	if (!message.guildId) return false
	const ch = message.channel
	if (!('isThread' in ch) || !ch.isThread()) return false
	return ch.parentId === process.env.COMMUNITY_SUPPORT_FORUM_ID
}

/** Returns true if the message is the starter message of its thread. */
export async function isThreadStarterMessage(message: Message): Promise<boolean> {
	const ch = message.channel
	if (!('isThread' in ch) || !ch.isThread()) return false
	try {
		// Ensure we're in the thread so we can fetch the starter reliably
		await tryJoinThread(ch as ThreadChannel)
		const starter = await ch.fetchStarterMessage()
		return !!starter && starter.id === message.id
	} catch {
		return false
	}
}

/** Returns true if the given message was authored by the thread starter (OP). */
export async function isByThreadOP(message: Message): Promise<boolean> {
	const ch = message.channel
	if (!('isThread' in ch) || !ch.isThread()) return false
	try {
		// Ensure we're in the thread so we can fetch the starter reliably
		await tryJoinThread(ch as ThreadChannel)
		const starter = await ch.fetchStarterMessage()
		return !!starter && starter.author?.id === message.author.id
	} catch {
		return false
	}
}

/** Returns true if the given message was in a thead that is marked as solved */
export async function isThreadSolved(message: Message): Promise<boolean> {
	const ch = message.channel
	if (!('isThread' in ch) || !ch.isThread()) return false
	try {
		// Ensure we're in the thread so we can fetch the starter reliably
		await tryJoinThread(ch as ThreadChannel)

		return ch.appliedTags?.includes(process.env.COMMUNITY_SUPPORT_FORUM_SOLVED_TAG_ID!)
	} catch {
		return false
	}
}
