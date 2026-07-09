import { MOD_ROLE_PING_REPORT_INSTEAD } from '@/data/'
import { MessageReactListener } from '@/types/'

// When a moderator reacts with ⚠️ to a message that pinged the moderator role,
// reply reminding the user to use the report feature instead of pinging mods directly.
export const moderatorPingReminder: MessageReactListener = {
	id: 'reaction.mod-ping-reminder',
	event: 'reactAdd',
	description: 'Reminds users not to ping the moderator role; use the report feature instead',
	priority: 10,
	async match({ reaction }): Promise<boolean> {
		return reaction.emoji.name === '⚠️'
	},
	async handle({ reaction, user }): Promise<void> {
		if ((user as any).bot) return
		const modRoleId = process.env.DISCORD_MODERATOR_ROLE_ID
		if (!modRoleId) return

		const message = reaction.message
		if (!message || !message.guild) return

		const member = await message.guild.members.fetch(user.id).catch(() => null)
		if (!member || !member.roles.cache.has(modRoleId)) return

		const mentioned = (message as any).mentions?.roles?.has(modRoleId)
		if (!mentioned) return

		const recent = await message.channel.messages.fetch({ limit: 15 }).catch(() => null)
		const duplicate = recent?.some(
			(m: any) =>
				m.reference?.messageId === message.id &&
				/avoid pinging the (discord )?moderator role/i.test(m.content),
		)
		if (duplicate) return

		await reaction.remove()

		await message.reply(MOD_ROLE_PING_REPORT_INSTEAD)
	},
}

export default moderatorPingReminder
