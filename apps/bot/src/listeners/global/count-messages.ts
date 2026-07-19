import { bufferMessageCount } from '@/cron/messageCountFlush'
import { CreateListener } from '@/types'

export const countMessages: CreateListener = {
	id: 'global:count-messages',
	event: 'create',
	description: 'Counts messages',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async () => true,
	handle: async (ctx) => {
		if (!ctx.message.guild) return
		if (!ctx.message.author) return
		if (ctx.message.content.length <= 15) return
		if (ctx.message.channel.isThread()) return

		// Buffered in memory and flushed periodically in one batched upsert
		// (src/cron/messageCountFlush.ts) instead of one DB round-trip per
		// guild message — this fires on every qualifying message in every guild.
		bufferMessageCount(ctx.message.author.id)

		// const user = await db
		// 	.select({
		// 		id: users.id,
		// 		messagesSent: users.messagesSent,
		// 	})
		// 	.from(users)
		// 	.where(eq(users.id, ctx.message.author.id))
		// 	.limit(1)

		// if (user[0].messagesSent == 20) {
		// 	const guild = ctx.message.guild
		// 	const member = await guild.members.fetch(ctx.message.author.id)
		// 	const roleId = process.env.ACTIVE_ROLE_ID!
		// 	if (!roleId) return
		//
		// 	const alreadyHasRole = member.roles.cache.has(roleId)
		// 	if (alreadyHasRole) {
		// 		return
		// 	}
		//
		// 	await member.roles.add(roleId)
		// 	const embed = createDefaultEmbed(ACTIVE_ROLE_GRANTED_EMBED)
		// 	try {
		// 		await ctx.message.author.send({ embeds: [embed] })
		// 		info(
		// 			`:white_check_mark: User ${member.user} (\`${member.user.username}\`, ID: ${member.user.id}) has been automatically verified for 20 counted messages.`,
		// 		)
		// 	} catch {
		// 		// ignore DM failures
		// 	}
		// }
	},
}
