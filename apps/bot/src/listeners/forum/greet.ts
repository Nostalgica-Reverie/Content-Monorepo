import { ThreadChannel } from 'discord.js'

import { GREET_EMBED } from '@/data/forum'
import { CreateListener } from '@/types'
import { createDefaultEmbed } from '@/utils'
import { isInCommunitySupportThread, isThreadStarterMessage } from '@/utils'

export const greetCommunitySupport: CreateListener = {
	id: 'forum:community-support:greet',
	event: 'create',
	description: 'Greet new threads in #community-support with usage and support tips.',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async (ctx) =>
		isInCommunitySupportThread(ctx.message) && (await isThreadStarterMessage(ctx.message)),
	handle: async (ctx) => {
		const ch = ctx.message.channel as ThreadChannel
		const embed = createDefaultEmbed(GREET_EMBED)
		await ch.send({ embeds: [embed] })
	},
}
