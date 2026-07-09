import { ThreadChannel } from 'discord.js'

import { OP_DELETE_LOCK_EMBED } from '@/data'
import { MessageListener } from '@/types'
import { createDefaultEmbed, isInCommunitySupportThread } from '@/utils'

export const lockOnOpDeletesStarter: MessageListener = {
	id: 'forum:community-support:lock-on-op-delete-starter',
	event: 'delete',
	description: 'Auto lock/archive the thread when the starter message is deleted by the OP.',
	priority: 10,
	filter: { allowBots: true, allowDMs: false },
	match: async (ctx) => {
		try {
			if (!('message' in ctx)) return false
			const { message } = ctx
			if (!('channel' in message)) return false
			if (!isInCommunitySupportThread(message as any)) return false
			const ch = (message as any).channel as ThreadChannel
			return ch.id === (message as any).id
		} catch (err) {
			console.error('[lockOnOpDeletesStarter] Failed to match starter message')
			console.error(err)
			return false
		}
	},
	handle: async (ctx) => {
		if (!('message' in ctx)) return
		const ch = (ctx.message as any).channel as ThreadChannel
		try {
			const embed = createDefaultEmbed(OP_DELETE_LOCK_EMBED).setColor(0xff496e)
			await ch.send({ embeds: [embed] }).catch(() => {})

			if (!ch.locked) await ch.setLocked(true, 'OP deleted starter message')
			if (!ch.archived) await ch.setArchived(true, 'OP deleted starter message')
		} catch (err) {
			console.error('[lockOnOpDeletesStarter] Failed to lock/archive thread')
			console.error(err)
		}
	},
}
