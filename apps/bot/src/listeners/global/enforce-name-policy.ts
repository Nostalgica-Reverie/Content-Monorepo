import { GuildMember } from 'discord.js'

import { info } from '@/logging/logger'
import { CreateListener } from '@/types'
import { capitalizeFirstChar, isByActivatedUser } from '@/utils'

export const enforceNamePolicy: CreateListener = {
	id: 'global:enforce-name-policy',
	event: 'create',
	description: 'Enforces name policy',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async () => true,
	handle: async (ctx) => {
		if (!ctx.message.guild) return
		if (!ctx.message.author) return

		const member = ctx.message.guild.members.cache.get(ctx.message.author.id) as GuildMember
		if (!member) return

		// Skip enforcement for activated members
		if (isByActivatedUser(member)) return

		const displayName = member.displayName.toLowerCase()
		const compliantDisplayName = capitalizeFirstChar(member.user.username)

		const namePolicy = /^[\u0020-\u024F\-'. ]+$/u

		if (!namePolicy.test(displayName)) {
			// At this point, name violates the policy.
			try {
				await member.setNickname(compliantDisplayName, 'Name policy violation')
			} catch (err) {
				console.error(`Failed to update nickname for ${member.id}:`, err)
			}

			info(
				`:pencil2: Reset nickname for user ${member.user} (\`${member.user.username}\`, ID: ${member.user.id}) from \`${displayName}\` to \`${compliantDisplayName}\`.`,
			)
		}
	},
}
